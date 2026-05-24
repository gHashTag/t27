use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

impl std::fmt::Display for ErrSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrSeverity::Info => write!(f, "info"),
            ErrSeverity::Warning => write!(f, "warning"),
            ErrSeverity::Error => write!(f, "error"),
            ErrSeverity::Fatal => write!(f, "fatal"),
        }
    }
}

pub type ErrCode = u32;

#[derive(Debug, Clone)]
pub struct ErrLink {
    pub code: ErrCode,
    pub message: String,
    pub severity: ErrSeverity,
    pub context: BTreeMap<String, String>,
}

impl ErrLink {
    pub fn new(code: ErrCode, message: &str, severity: ErrSeverity) -> Self {
        Self {
            code,
            message: message.to_string(),
            severity,
            context: BTreeMap::new(),
        }
    }

    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ErrorChain {
    links: Vec<ErrLink>,
    timestamp_us: u64,
}

impl ErrorChain {
    pub fn new(link: ErrLink, timestamp_us: u64) -> Self {
        Self {
            links: vec![link],
            timestamp_us,
        }
    }

    pub fn push(&mut self, link: ErrLink) {
        self.links.push(link);
    }

    pub fn root(&self) -> &ErrLink {
        self.links.first().unwrap()
    }

    pub fn tip(&self) -> &ErrLink {
        self.links.last().unwrap()
    }

    pub fn depth(&self) -> usize {
        self.links.len()
    }

    pub fn links(&self) -> &[ErrLink] {
        &self.links
    }

    pub fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }

    pub fn worst_severity(&self) -> ErrSeverity {
        self.links.iter().map(|l| l.severity).max().unwrap_or(ErrSeverity::Info)
    }

    pub fn contains_code(&self, code: ErrCode) -> bool {
        self.links.iter().any(|l| l.code == code)
    }
}

#[derive(Debug, Clone)]
pub struct ErrorChainBuilder {
    chain: ErrorChain,
}

impl ErrorChainBuilder {
    pub fn new(code: ErrCode, message: &str, severity: ErrSeverity, timestamp_us: u64) -> Self {
        Self {
            chain: ErrorChain::new(ErrLink::new(code, message, severity), timestamp_us),
        }
    }

    pub fn chain(mut self, code: ErrCode, message: &str, severity: ErrSeverity) -> Self {
        self.chain.push(ErrLink::new(code, message, severity));
        self
    }

    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        if let Some(tip) = self.chain.links.last_mut() {
            tip.context.insert(key.to_string(), value.to_string());
        }
        self
    }

    pub fn build(self) -> ErrorChain {
        self.chain
    }
}

#[derive(Debug, Clone)]
pub struct ErrorChainCollector {
    chains: Vec<ErrorChain>,
    max_chains: usize,
    total: u64,
    by_severity: BTreeMap<ErrSeverity, u64>,
}

impl ErrorChainCollector {
    pub fn new(max_chains: usize) -> Self {
        Self {
            chains: Vec::new(),
            max_chains,
            total: 0,
            by_severity: BTreeMap::new(),
        }
    }

    pub fn record(&mut self, chain: ErrorChain) {
        self.total += 1;
        let sev = chain.worst_severity();
        *self.by_severity.entry(sev).or_insert(0) += 1;
        if self.chains.len() >= self.max_chains {
            self.chains.remove(0);
        }
        self.chains.push(chain);
    }

    pub fn chains(&self) -> &[ErrorChain] {
        &self.chains
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn count_by_severity(&self, sev: ErrSeverity) -> u64 {
        self.by_severity.get(&sev).copied().unwrap_or(0)
    }

    pub fn clear(&mut self) {
        self.chains.clear();
        self.total = 0;
        self.by_severity.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn err_link_new() {
        let l = ErrLink::new(1, "test", ErrSeverity::Error);
        assert_eq!(l.code, 1);
        assert_eq!(l.message, "test");
    }

    #[test]
    fn err_link_with_context() {
        let l = ErrLink::new(1, "x", ErrSeverity::Error).with_context("addr", "0x100");
        assert_eq!(l.context.get("addr").unwrap(), "0x100");
    }

    #[test]
    fn severity_display() {
        assert_eq!(ErrSeverity::Fatal.to_string(), "fatal");
    }

    #[test]
    fn chain_root_tip() {
        let mut chain = ErrorChain::new(ErrLink::new(1, "root", ErrSeverity::Error), 0);
        chain.push(ErrLink::new(2, "tip", ErrSeverity::Fatal));
        assert_eq!(chain.root().code, 1);
        assert_eq!(chain.tip().code, 2);
        assert_eq!(chain.depth(), 2);
    }

    #[test]
    fn chain_worst_severity() {
        let mut chain = ErrorChain::new(ErrLink::new(1, "a", ErrSeverity::Info), 0);
        chain.push(ErrLink::new(2, "b", ErrSeverity::Error));
        assert_eq!(chain.worst_severity(), ErrSeverity::Error);
    }

    #[test]
    fn chain_contains_code() {
        let mut chain = ErrorChain::new(ErrLink::new(42, "a", ErrSeverity::Error), 0);
        chain.push(ErrLink::new(99, "b", ErrSeverity::Error));
        assert!(chain.contains_code(42));
        assert!(!chain.contains_code(7));
    }

    #[test]
    fn builder_basic() {
        let chain = ErrorChainBuilder::new(1, "root", ErrSeverity::Error, 100)
            .chain(2, "mid", ErrSeverity::Warning)
            .chain(3, "tip", ErrSeverity::Fatal)
            .build();
        assert_eq!(chain.depth(), 3);
        assert_eq!(chain.tip().code, 3);
    }

    #[test]
    fn builder_with_context() {
        let chain = ErrorChainBuilder::new(1, "x", ErrSeverity::Error, 0)
            .with_context("key", "val")
            .build();
        assert_eq!(chain.tip().context.get("key").unwrap(), "val");
    }

    #[test]
    fn collector_record() {
        let mut c = ErrorChainCollector::new(10);
        let chain = ErrorChain::new(ErrLink::new(1, "a", ErrSeverity::Error), 0);
        c.record(chain);
        assert_eq!(c.total(), 1);
        assert_eq!(c.count_by_severity(ErrSeverity::Error), 1);
    }

    #[test]
    fn collector_evicts() {
        let mut c = ErrorChainCollector::new(2);
        c.record(ErrorChain::new(ErrLink::new(1, "a", ErrSeverity::Error), 0));
        c.record(ErrorChain::new(ErrLink::new(2, "b", ErrSeverity::Error), 1));
        c.record(ErrorChain::new(ErrLink::new(3, "c", ErrSeverity::Error), 2));
        assert_eq!(c.chains().len(), 2);
        assert_eq!(c.chains()[0].root().code, 2);
        assert_eq!(c.total(), 3);
    }

    #[test]
    fn collector_clear() {
        let mut c = ErrorChainCollector::new(10);
        c.record(ErrorChain::new(ErrLink::new(1, "a", ErrSeverity::Error), 0));
        c.clear();
        assert_eq!(c.total(), 0);
        assert!(c.chains().is_empty());
    }

    #[test]
    fn chain_timestamp() {
        let chain = ErrorChain::new(ErrLink::new(1, "a", ErrSeverity::Error), 42);
        assert_eq!(chain.timestamp_us(), 42);
    }
}

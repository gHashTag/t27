#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    Accept,
    Deny,
}

impl std::fmt::Display for FilterAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterAction::Accept => write!(f, "accept"),
            FilterAction::Deny => write!(f, "deny"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterRule {
    pub name: String,
    pub match_fn: fn(&[u8]) -> bool,
    pub action: FilterAction,
    pub priority: u32,
}

impl FilterRule {
    pub fn new(name: &str, match_fn: fn(&[u8]) -> bool, action: FilterAction, priority: u32) -> Self {
        Self {
            name: name.to_string(),
            match_fn,
            action,
            priority,
        }
    }

    pub fn matches(&self, packet: &[u8]) -> bool {
        (self.match_fn)(packet)
    }
}

#[derive(Debug, Clone)]
pub struct PacketFilter {
    rules: Vec<FilterRule>,
    default_action: FilterAction,
    accepted: u64,
    denied: u64,
}

impl PacketFilter {
    pub fn new(default_action: FilterAction) -> Self {
        Self {
            rules: Vec::new(),
            default_action,
            accepted: 0,
            denied: 0,
        }
    }

    pub fn add_rule(&mut self, rule: FilterRule) {
        let pos = self.rules.iter().position(|r| r.priority > rule.priority).unwrap_or(self.rules.len());
        self.rules.insert(pos, rule);
    }

    pub fn remove_rule(&mut self, name: &str) -> bool {
        let len_before = self.rules.len();
        self.rules.retain(|r| r.name != name);
        self.rules.len() != len_before
    }

    pub fn apply(&mut self, packet: &[u8]) -> FilterAction {
        for rule in &self.rules {
            if rule.matches(packet) {
                match rule.action {
                    FilterAction::Accept => self.accepted += 1,
                    FilterAction::Deny => self.denied += 1,
                }
                return rule.action;
            }
        }
        match self.default_action {
            FilterAction::Accept => self.accepted += 1,
            FilterAction::Deny => self.denied += 1,
        }
        self.default_action
    }

    pub fn apply_batch(&mut self, packets: &[&[u8]]) -> Vec<FilterAction> {
        packets.iter().map(|p| self.apply(p)).collect()
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn accepted(&self) -> u64 {
        self.accepted
    }

    pub fn denied(&self) -> u64 {
        self.denied
    }

    pub fn total(&self) -> u64 {
        self.accepted + self.denied
    }

    pub fn deny_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            self.denied as f64 / self.total() as f64
        }
    }

    pub fn reset_stats(&mut self) {
        self.accepted = 0;
        self.denied = 0;
    }

    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }
}

impl Default for PacketFilter {
    fn default() -> Self {
        Self::new(FilterAction::Accept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn always_true(_pkt: &[u8]) -> bool {
        true
    }

    fn always_false(_pkt: &[u8]) -> bool {
        false
    }

    fn is_short(pkt: &[u8]) -> bool {
        pkt.len() < 4
    }

    fn starts_with_ff(pkt: &[u8]) -> bool {
        pkt.first() == Some(&0xFF)
    }

    #[test]
    fn action_display() {
        assert_eq!(FilterAction::Accept.to_string(), "accept");
        assert_eq!(FilterAction::Deny.to_string(), "deny");
    }

    #[test]
    fn rule_matches() {
        let r = FilterRule::new("short", is_short, FilterAction::Deny, 10);
        assert!(r.matches(b"abc"));
        assert!(!r.matches(b"hello"));
    }

    #[test]
    fn default_accept() {
        let mut pf = PacketFilter::new(FilterAction::Accept);
        assert_eq!(pf.apply(b"anything"), FilterAction::Accept);
        assert_eq!(pf.accepted(), 1);
    }

    #[test]
    fn default_deny() {
        let mut pf = PacketFilter::new(FilterAction::Deny);
        assert_eq!(pf.apply(b"anything"), FilterAction::Deny);
        assert_eq!(pf.denied(), 1);
    }

    #[test]
    fn add_rule_priority_order() {
        let mut pf = PacketFilter::new(FilterAction::Accept);
        pf.add_rule(FilterRule::new("low", always_true, FilterAction::Deny, 100));
        pf.add_rule(FilterRule::new("high", always_true, FilterAction::Accept, 1));
        assert_eq!(pf.rules[0].name, "high");
        assert_eq!(pf.rules[1].name, "low");
    }

    #[test]
    fn first_match_wins() {
        let mut pf = PacketFilter::new(FilterAction::Deny);
        pf.add_rule(FilterRule::new("deny_short", is_short, FilterAction::Deny, 10));
        pf.add_rule(FilterRule::new("accept_all", always_true, FilterAction::Accept, 20));
        assert_eq!(pf.apply(b"abc"), FilterAction::Deny);
        assert_eq!(pf.apply(b"hello world"), FilterAction::Accept);
    }

    #[test]
    fn remove_rule() {
        let mut pf = PacketFilter::new(FilterAction::Deny);
        pf.add_rule(FilterRule::new("r1", always_true, FilterAction::Accept, 10));
        assert!(pf.remove_rule("r1"));
        assert!(!pf.remove_rule("r1"));
        assert_eq!(pf.rule_count(), 0);
    }

    #[test]
    fn apply_batch() {
        let mut pf = PacketFilter::new(FilterAction::Accept);
        pf.add_rule(FilterRule::new("deny_ff", starts_with_ff, FilterAction::Deny, 10));
        let results = pf.apply_batch(&[b"normal", b"\xff\x00", b"ok"]);
        assert_eq!(results, vec![FilterAction::Accept, FilterAction::Deny, FilterAction::Accept]);
    }

    #[test]
    fn deny_rate() {
        let mut pf = PacketFilter::new(FilterAction::Accept);
        pf.add_rule(FilterRule::new("deny_short", is_short, FilterAction::Deny, 10));
        pf.apply(b"abc");
        pf.apply(b"hello world");
        assert!((pf.deny_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn reset_stats() {
        let mut pf = PacketFilter::new(FilterAction::Accept);
        pf.apply(b"x");
        pf.reset_stats();
        assert_eq!(pf.total(), 0);
    }

    #[test]
    fn clear_rules() {
        let mut pf = PacketFilter::new(FilterAction::Accept);
        pf.add_rule(FilterRule::new("r", always_true, FilterAction::Deny, 10));
        pf.clear_rules();
        assert_eq!(pf.rule_count(), 0);
    }

    #[test]
    fn no_match_uses_default() {
        let mut pf = PacketFilter::new(FilterAction::Accept);
        pf.add_rule(FilterRule::new("never", always_false, FilterAction::Deny, 10));
        assert_eq!(pf.apply(b"anything"), FilterAction::Accept);
    }
}

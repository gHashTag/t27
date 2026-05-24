use std::collections::BTreeMap;

pub type IrqHandlerFn = fn(IrqSource, u32) -> IrqAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IrqSource {
    InferenceDone = 0,
    DmaDone = 1,
    Error = 2,
}

impl std::fmt::Display for IrqSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrqSource::InferenceDone => write!(f, "inference_done"),
            IrqSource::DmaDone => write!(f, "dma_done"),
            IrqSource::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqAction {
    Handled,
    WakePoll,
    ResetDevice,
    LogAndContinue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteError {
    NoHandler { source: IrqSource },
    SourceAlreadyRouted { source: IrqSource },
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteError::NoHandler { source } => write!(f, "no handler for {source}"),
            RouteError::SourceAlreadyRouted { source } => {
                write!(f, "{source} already has a handler")
            }
        }
    }
}

impl std::error::Error for RouteError {}

#[derive(Debug, Clone, Copy)]
pub struct IrqEvent {
    pub source: IrqSource,
    pub timestamp_us: u64,
    pub data: u32,
}

#[derive(Debug, Clone)]
pub struct IrqRouter {
    handlers: BTreeMap<IrqSource, IrqHandlerFn>,
    mask: u8,
    total_dispatched: u64,
    total_dropped: u64,
    per_source_count: BTreeMap<IrqSource, u64>,
}

impl IrqRouter {
    pub fn new() -> Self {
        Self {
            handlers: BTreeMap::new(),
            mask: 0x07,
            total_dispatched: 0,
            total_dropped: 0,
            per_source_count: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, source: IrqSource, handler: IrqHandlerFn) -> Result<(), RouteError> {
        if self.handlers.contains_key(&source) {
            return Err(RouteError::SourceAlreadyRouted { source });
        }
        self.handlers.insert(source, handler);
        Ok(())
    }

    pub fn unregister(&mut self, source: IrqSource) -> bool {
        self.handlers.remove(&source).is_some()
    }

    pub fn set_mask(&mut self, mask: u8) {
        self.mask = mask;
    }

    pub fn mask(&self) -> u8 {
        self.mask
    }

    pub fn is_masked(&self, source: IrqSource) -> bool {
        let bit = source as u8;
        (self.mask & (1 << bit)) == 0
    }

    pub fn dispatch(&mut self, event: IrqEvent) -> Option<IrqAction> {
        if self.is_masked(event.source) {
            self.total_dropped += 1;
            return None;
        }
        let handler = match self.handlers.get(&event.source) {
            Some(h) => *h,
            None => {
                self.total_dropped += 1;
                return None;
            }
        };
        *self.per_source_count.entry(event.source).or_insert(0) += 1;
        self.total_dispatched += 1;
        Some(handler(event.source, event.data))
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    pub fn has_handler(&self, source: IrqSource) -> bool {
        self.handlers.contains_key(&source)
    }

    pub fn total_dispatched(&self) -> u64 {
        self.total_dispatched
    }

    pub fn total_dropped(&self) -> u64 {
        self.total_dropped
    }

    pub fn source_count(&self, source: IrqSource) -> u64 {
        *self.per_source_count.get(&source).unwrap_or(&0)
    }

    pub fn clear_stats(&mut self) {
        self.total_dispatched = 0;
        self.total_dropped = 0;
        self.per_source_count.clear();
    }
}

impl Default for IrqRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handler_done(_src: IrqSource, _data: u32) -> IrqAction {
        IrqAction::WakePoll
    }

    fn handler_dma(_src: IrqSource, data: u32) -> IrqAction {
        if data == 0 {
            IrqAction::Handled
        } else {
            IrqAction::LogAndContinue
        }
    }

    fn handler_error(_src: IrqSource, _data: u32) -> IrqAction {
        IrqAction::ResetDevice
    }

    #[test]
    fn register_and_dispatch() {
        let mut r = IrqRouter::new();
        r.register(IrqSource::InferenceDone, handler_done).unwrap();
        let action = r.dispatch(IrqEvent {
            source: IrqSource::InferenceDone,
            timestamp_us: 100,
            data: 0,
        });
        assert_eq!(action, Some(IrqAction::WakePoll));
        assert_eq!(r.total_dispatched(), 1);
    }

    #[test]
    fn register_duplicate_fails() {
        let mut r = IrqRouter::new();
        r.register(IrqSource::InferenceDone, handler_done).unwrap();
        let err = r.register(IrqSource::InferenceDone, handler_done).unwrap_err();
        assert!(matches!(err, RouteError::SourceAlreadyRouted { .. }));
    }

    #[test]
    fn unregister() {
        let mut r = IrqRouter::new();
        r.register(IrqSource::InferenceDone, handler_done).unwrap();
        assert!(r.unregister(IrqSource::InferenceDone));
        assert!(!r.has_handler(IrqSource::InferenceDone));
        assert!(!r.unregister(IrqSource::InferenceDone));
    }

    #[test]
    fn no_handler_drops() {
        let mut r = IrqRouter::new();
        let action = r.dispatch(IrqEvent {
            source: IrqSource::DmaDone,
            timestamp_us: 0,
            data: 0,
        });
        assert!(action.is_none());
        assert_eq!(r.total_dropped(), 1);
    }

    #[test]
    fn masked_drops() {
        let mut r = IrqRouter::new();
        r.register(IrqSource::InferenceDone, handler_done).unwrap();
        r.set_mask(0);
        let action = r.dispatch(IrqEvent {
            source: IrqSource::InferenceDone,
            timestamp_us: 0,
            data: 0,
        });
        assert!(action.is_none());
        assert_eq!(r.total_dropped(), 1);
    }

    #[test]
    fn per_source_count() {
        let mut r = IrqRouter::new();
        r.register(IrqSource::DmaDone, handler_dma).unwrap();
        r.dispatch(IrqEvent { source: IrqSource::DmaDone, timestamp_us: 0, data: 0 }).unwrap();
        r.dispatch(IrqEvent { source: IrqSource::DmaDone, timestamp_us: 0, data: 1 }).unwrap();
        assert_eq!(r.source_count(IrqSource::DmaDone), 2);
    }

    #[test]
    fn multiple_sources() {
        let mut r = IrqRouter::new();
        r.register(IrqSource::InferenceDone, handler_done).unwrap();
        r.register(IrqSource::DmaDone, handler_dma).unwrap();
        r.register(IrqSource::Error, handler_error).unwrap();
        assert_eq!(r.handler_count(), 3);
        let a1 = r.dispatch(IrqEvent { source: IrqSource::Error, timestamp_us: 0, data: 0 }).unwrap();
        assert_eq!(a1, IrqAction::ResetDevice);
        let a2 = r.dispatch(IrqEvent { source: IrqSource::DmaDone, timestamp_us: 0, data: 0 }).unwrap();
        assert_eq!(a2, IrqAction::Handled);
    }

    #[test]
    fn clear_stats() {
        let mut r = IrqRouter::new();
        r.register(IrqSource::InferenceDone, handler_done).unwrap();
        r.dispatch(IrqEvent { source: IrqSource::InferenceDone, timestamp_us: 0, data: 0 }).unwrap();
        r.clear_stats();
        assert_eq!(r.total_dispatched(), 0);
        assert_eq!(r.source_count(IrqSource::InferenceDone), 0);
    }

    #[test]
    fn is_masked() {
        let mut r = IrqRouter::new();
        r.set_mask(0x01);
        assert!(!r.is_masked(IrqSource::InferenceDone));
        assert!(r.is_masked(IrqSource::DmaDone));
    }

    #[test]
    fn source_display() {
        assert_eq!(IrqSource::InferenceDone.to_string(), "inference_done");
        assert_eq!(IrqSource::Error.to_string(), "error");
    }

    #[test]
    fn error_display() {
        let e = RouteError::NoHandler { source: IrqSource::DmaDone };
        assert!(e.to_string().contains("dma_done"));
    }

    #[test]
    fn default_is_empty() {
        let r = IrqRouter::default();
        assert_eq!(r.handler_count(), 0);
    }
}

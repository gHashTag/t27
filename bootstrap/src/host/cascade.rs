use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CascadeError {
    HandlerExists { id: u64 },
    HandlerNotFound { id: u64 },
    NoHandlers,
}

impl std::fmt::Display for CascadeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CascadeError::HandlerExists { id } => write!(f, "handler {id} exists"),
            CascadeError::HandlerNotFound { id } => write!(f, "handler {id} not found"),
            CascadeError::NoHandlers => write!(f, "no handlers"),
        }
    }
}

impl std::error::Error for CascadeError {}

#[derive(Debug, Clone)]
struct Handler {
    id: u64,
    priority: u32,
    enabled: bool,
    notify_count: u64,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub code: u32,
    pub payload: u64,
}

#[derive(Debug, Clone)]
pub struct DeliveryReport {
    pub notified: usize,
    pub skipped: usize,
    pub total_handlers: usize,
}

#[derive(Debug, Clone)]
pub struct CascadeNotifier {
    handlers: BTreeMap<u64, Handler>,
    order: Vec<u64>,
    total_notifications: u64,
    total_deliveries: u64,
    total_skips: u64,
}

impl CascadeNotifier {
    pub fn new() -> Self {
        Self {
            handlers: BTreeMap::new(),
            order: Vec::new(),
            total_notifications: 0,
            total_deliveries: 0,
            total_skips: 0,
        }
    }

    pub fn register(&mut self, id: u64, priority: u32) -> Result<(), CascadeError> {
        if self.handlers.contains_key(&id) {
            return Err(CascadeError::HandlerExists { id });
        }
        self.handlers.insert(id, Handler { id, priority, enabled: true, notify_count: 0 });
        self.rebuild_order();
        Ok(())
    }

    pub fn unregister(&mut self, id: u64) -> Result<(), CascadeError> {
        if self.handlers.remove(&id).is_none() {
            return Err(CascadeError::HandlerNotFound { id });
        }
        self.rebuild_order();
        Ok(())
    }

    pub fn enable(&mut self, id: u64) -> Result<bool, CascadeError> {
        let h = self.handlers.get_mut(&id).ok_or(CascadeError::HandlerNotFound { id })?;
        h.enabled = true;
        Ok(true)
    }

    pub fn disable(&mut self, id: u64) -> Result<bool, CascadeError> {
        let h = self.handlers.get_mut(&id).ok_or(CascadeError::HandlerNotFound { id })?;
        h.enabled = false;
        Ok(false)
    }

    pub fn is_enabled(&self, id: u64) -> bool {
        self.handlers.get(&id).map(|h| h.enabled).unwrap_or(false)
    }

    fn rebuild_order(&mut self) {
        let mut ids: Vec<(u64, u32)> = self.handlers.iter().map(|(&id, h)| (id, h.priority)).collect();
        ids.sort_by_key(|&(_, p)| std::cmp::Reverse(p));
        self.order = ids.into_iter().map(|(id, _)| id).collect();
    }

    pub fn notify(&mut self, notification: &Notification) -> DeliveryReport {
        self.total_notifications += 1;
        let mut notified = 0;
        let mut skipped = 0;
        for &id in &self.order {
            if let Some(h) = self.handlers.get_mut(&id) {
                if h.enabled {
                    h.notify_count += 1;
                    notified += 1;
                    self.total_deliveries += 1;
                } else {
                    skipped += 1;
                    self.total_skips += 1;
                }
            }
        }
        DeliveryReport { notified, skipped, total_handlers: self.order.len() }
    }

    pub fn notify_one(&mut self, id: u64, notification: &Notification) -> bool {
        if let Some(h) = self.handlers.get_mut(&id) {
            if h.enabled {
                h.notify_count += 1;
                self.total_deliveries += 1;
                self.total_notifications += 1;
                return true;
            }
        }
        false
    }

    pub fn order(&self) -> &[u64] {
        &self.order
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.handlers.values().filter(|h| h.enabled).count()
    }

    pub fn notify_count(&self, id: u64) -> u64 {
        self.handlers.get(&id).map(|h| h.notify_count).unwrap_or(0)
    }

    pub fn total_notifications(&self) -> u64 { self.total_notifications }
    pub fn total_deliveries(&self) -> u64 { self.total_deliveries }
    pub fn total_skips(&self) -> u64 { self.total_skips }

    pub fn clear(&mut self) {
        self.handlers.clear();
        self.order.clear();
    }
}

impl Default for CascadeNotifier {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_notifier() {
        let cn = CascadeNotifier::new();
        assert_eq!(cn.handler_count(), 0);
    }

    #[test]
    fn register_and_count() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.register(2, 20).unwrap();
        assert_eq!(cn.handler_count(), 2);
    }

    #[test]
    fn duplicate_handler() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        let err = cn.register(1, 20).unwrap_err();
        assert!(matches!(err, CascadeError::HandlerExists { .. }));
    }

    #[test]
    fn notify_all() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.register(2, 20).unwrap();
        let report = cn.notify(&Notification { code: 1, payload: 42 });
        assert_eq!(report.notified, 2);
        assert_eq!(cn.notify_count(1), 1);
        assert_eq!(cn.notify_count(2), 1);
    }

    #[test]
    fn priority_order() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.register(2, 30).unwrap();
        cn.register(3, 20).unwrap();
        assert_eq!(cn.order(), vec![2, 3, 1]);
    }

    #[test]
    fn disable_skips() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.register(2, 20).unwrap();
        cn.disable(2).unwrap();
        let report = cn.notify(&Notification { code: 1, payload: 0 });
        assert_eq!(report.notified, 1);
        assert_eq!(report.skipped, 1);
        assert_eq!(cn.notify_count(2), 0);
    }

    #[test]
    fn enable_re_enables() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.disable(1).unwrap();
        cn.enable(1).unwrap();
        assert!(cn.is_enabled(1));
        let report = cn.notify(&Notification { code: 1, payload: 0 });
        assert_eq!(report.notified, 1);
    }

    #[test]
    fn unregister() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.unregister(1).unwrap();
        assert_eq!(cn.handler_count(), 0);
        let err = cn.unregister(1).unwrap_err();
        assert!(matches!(err, CascadeError::HandlerNotFound { .. }));
    }

    #[test]
    fn notify_one() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.register(2, 20).unwrap();
        assert!(cn.notify_one(1, &Notification { code: 1, payload: 0 }));
        assert_eq!(cn.notify_count(1), 1);
        assert_eq!(cn.notify_count(2), 0);
    }

    #[test]
    fn stats() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.notify(&Notification { code: 1, payload: 0 });
        cn.notify(&Notification { code: 2, payload: 0 });
        assert_eq!(cn.total_notifications(), 2);
        assert_eq!(cn.total_deliveries(), 2);
    }

    #[test]
    fn clear() {
        let mut cn = CascadeNotifier::new();
        cn.register(1, 10).unwrap();
        cn.clear();
        assert_eq!(cn.handler_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(CascadeError::HandlerExists { id: 3 }.to_string().contains("3"));
    }
}

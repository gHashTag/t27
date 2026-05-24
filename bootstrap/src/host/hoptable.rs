use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HopError {
    DuplicateRoute { prefix: String },
    NotFound { prefix: String },
}

impl std::fmt::Display for HopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HopError::DuplicateRoute { prefix } => write!(f, "duplicate route {prefix}"),
            HopError::NotFound { prefix } => write!(f, "route {prefix} not found"),
        }
    }
}

impl std::error::Error for HopError {}

#[derive(Debug, Clone)]
pub struct Route {
    pub prefix: String,
    pub prefix_len: u8,
    pub next_hop: String,
    pub metric: u32,
    pub interface: u8,
}

#[derive(Debug, Clone)]
pub struct HopTable {
    routes: BTreeMap<String, Route>,
    total_lookups: u64,
    total_hits: u64,
    total_misses: u64,
}

impl HopTable {
    pub fn new() -> Self {
        Self { routes: BTreeMap::new(), total_lookups: 0, total_hits: 0, total_misses: 0 }
    }

    pub fn add(&mut self, route: Route) -> Result<(), HopError> {
        if self.routes.contains_key(&route.prefix) {
            return Err(HopError::DuplicateRoute { prefix: route.prefix.clone() });
        }
        self.routes.insert(route.prefix.clone(), route);
        Ok(())
    }

    pub fn remove(&mut self, prefix: &str) -> Result<Route, HopError> {
        self.routes.remove(prefix)
            .ok_or(HopError::NotFound { prefix: prefix.to_string() })
    }

    pub fn update_metric(&mut self, prefix: &str, metric: u32) -> Result<(), HopError> {
        let route = self.routes.get_mut(prefix)
            .ok_or(HopError::NotFound { prefix: prefix.to_string() })?;
        route.metric = metric;
        Ok(())
    }

    pub fn lookup(&mut self, dest: &str) -> Option<&Route> {
        self.total_lookups += 1;
        let mut best: Option<&Route> = None;
        for route in self.routes.values() {
            if dest.starts_with(&route.prefix) {
                if let Some(ref b) = best {
                    if route.prefix_len > b.prefix_len
                        || (route.prefix_len == b.prefix_len && route.metric < b.metric)
                    {
                        best = Some(route);
                    }
                } else {
                    best = Some(route);
                }
            }
        }
        if best.is_some() { self.total_hits += 1; } else { self.total_misses += 1; }
        best
    }

    pub fn get(&self, prefix: &str) -> Option<&Route> {
        self.routes.get(prefix)
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    pub fn routes(&self) -> Vec<&Route> {
        self.routes.values().collect()
    }

    pub fn total_lookups(&self) -> u64 {
        self.total_lookups
    }

    pub fn total_hits(&self) -> u64 {
        self.total_hits
    }

    pub fn total_misses(&self) -> u64 {
        self.total_misses
    }

    pub fn clear(&mut self) {
        self.routes.clear();
    }
}

impl Default for HopTable {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(prefix: &str, len: u8, hop: &str, metric: u32, iface: u8) -> Route {
        Route { prefix: prefix.to_string(), prefix_len: len, next_hop: hop.to_string(), metric, interface: iface }
    }

    #[test]
    fn new_table() {
        let ht = HopTable::new();
        assert_eq!(ht.route_count(), 0);
    }

    #[test]
    fn add_and_get() {
        let mut ht = HopTable::new();
        ht.add(route("10.0.", 3, "eth0", 1, 0)).unwrap();
        let r = ht.get("10.0.").unwrap();
        assert_eq!(r.next_hop, "eth0");
    }

    #[test]
    fn duplicate_route() {
        let mut ht = HopTable::new();
        ht.add(route("10.", 2, "a", 1, 0)).unwrap();
        let err = ht.add(route("10.", 2, "b", 1, 0)).unwrap_err();
        assert!(matches!(err, HopError::DuplicateRoute { .. }));
    }

    #[test]
    fn remove_route() {
        let mut ht = HopTable::new();
        ht.add(route("10.", 2, "a", 1, 0)).unwrap();
        let r = ht.remove("10.").unwrap();
        assert_eq!(r.next_hop, "a");
        assert_eq!(ht.route_count(), 0);
    }

    #[test]
    fn remove_not_found() {
        let mut ht = HopTable::new();
        let err = ht.remove("nope").unwrap_err();
        assert!(matches!(err, HopError::NotFound { .. }));
    }

    #[test]
    fn lookup_longest_prefix() {
        let mut ht = HopTable::new();
        ht.add(route("10.", 2, "default", 10, 0)).unwrap();
        ht.add(route("10.0.", 3, "specific", 1, 1)).unwrap();
        let r = ht.lookup("10.0.1.2").unwrap();
        assert_eq!(r.next_hop, "specific");
    }

    #[test]
    fn lookup_lower_metric() {
        let mut ht = HopTable::new();
        ht.add(route("10.", 2, "slow", 100, 0)).unwrap();
        let r = ht.lookup("10.1.1.1").unwrap();
        assert_eq!(r.metric, 100);
    }

    #[test]
    fn lookup_miss() {
        let mut ht = HopTable::new();
        ht.add(route("192.", 3, "lan", 1, 0)).unwrap();
        assert!(ht.lookup("10.1.1.1").is_none());
        assert_eq!(ht.total_misses(), 1);
    }

    #[test]
    fn stats() {
        let mut ht = HopTable::new();
        ht.add(route("10.", 2, "a", 1, 0)).unwrap();
        ht.lookup("10.1.1.1");
        ht.lookup("192.1.1.1");
        assert_eq!(ht.total_lookups(), 2);
        assert_eq!(ht.total_hits(), 1);
        assert_eq!(ht.total_misses(), 1);
    }

    #[test]
    fn update_metric() {
        let mut ht = HopTable::new();
        ht.add(route("10.", 2, "a", 10, 0)).unwrap();
        ht.update_metric("10.", 5).unwrap();
        assert_eq!(ht.get("10.").unwrap().metric, 5);
    }

    #[test]
    fn clear() {
        let mut ht = HopTable::new();
        ht.add(route("10.", 2, "a", 1, 0)).unwrap();
        ht.clear();
        assert_eq!(ht.route_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(HopError::NotFound { prefix: "x".into() }.to_string().contains("x"));
    }
}

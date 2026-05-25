use std::collections::BTreeMap;

struct CfData { mem: BTreeMap<Vec<u8>, Option<Vec<u8>>>, levels: Vec<BTreeMap<Vec<u8>, Option<Vec<u8>>>> }

pub struct SparLsm {
    families: BTreeMap<Vec<u8>, CfData>,
    mem_cap: usize,
    total_puts: u64,
    total_gets: u64,
    total_gcs: u64,
}

impl SparLsm {
    pub fn new(mem_cap: usize) -> Self { Self { families: BTreeMap::new(), mem_cap, total_puts: 0, total_gets: 0, total_gcs: 0 } }

    fn ensure_cf(&mut self, cf: &[u8]) {
        if !self.families.contains_key(cf) {
            self.families.insert(cf.to_vec(), CfData { mem: BTreeMap::new(), levels: Vec::new() });
        }
    }

    pub fn put(&mut self, cf: &[u8], key: &[u8], value: Vec<u8>) {
        self.total_puts += 1;
        self.ensure_cf(cf);
        let cf_data = self.families.get_mut(cf).unwrap();
        cf_data.mem.insert(key.to_vec(), Some(value));
        if cf_data.mem.len() >= self.mem_cap { self.flush_cf(cf); }
    }

    pub fn delete(&mut self, cf: &[u8], key: &[u8]) {
        self.total_puts += 1;
        self.ensure_cf(cf);
        let cf_data = self.families.get_mut(cf).unwrap();
        cf_data.mem.insert(key.to_vec(), None);
    }

    pub fn get(&mut self, cf: &[u8], key: &[u8]) -> Option<Vec<u8>> {
        self.total_gets += 1;
        let cf_data = self.families.get(cf)?;
        if let Some(v) = cf_data.mem.get(key) { return v.clone(); }
        for level in cf_data.levels.iter().rev() {
            if let Some(v) = level.get(key) { return v.clone(); }
        }
        None
    }

    fn flush_cf(&mut self, cf: &[u8]) {
        let cf_data = self.families.get_mut(cf).unwrap();
        let new_level: BTreeMap<Vec<u8>, Option<Vec<u8>>> = std::mem::take(&mut cf_data.mem);
        cf_data.levels.push(new_level);
    }

    pub fn gc(&mut self, cf: &[u8]) -> usize {
        self.total_gcs += 1;
        let cf_data = match self.families.get_mut(cf) { Some(d) => d, None => return 0 };
        let mut merged: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();
        let mut removed_tombstones = 0usize;
        for level in cf_data.levels.drain(..) {
            for (k, v) in level {
                if v.is_none() && merged.contains_key(&k) { removed_tombstones += 1; }
                merged.insert(k, v);
            }
        }
        merged.retain(|_, v| v.is_some());
        cf_data.levels.push(merged);
        removed_tombstones
    }

    pub fn cf_names(&self) -> Vec<&[u8]> { self.families.keys().map(|k| k.as_slice()).collect() }
    pub fn level_count(&self, cf: &[u8]) -> usize { self.families.get(cf).map(|d| d.levels.len()).unwrap_or(0) }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_gcs(&self) -> u64 { self.total_gcs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get() {
        let mut lsm = SparLsm::new(100);
        lsm.put(b"cf1", b"k", b"v".to_vec());
        assert_eq!(lsm.get(b"cf1", b"k"), Some(b"v".to_vec()));
    }

    #[test]
    fn cf_isolation() {
        let mut lsm = SparLsm::new(100);
        lsm.put(b"cf1", b"k", b"1".to_vec());
        lsm.put(b"cf2", b"k", b"2".to_vec());
        assert_eq!(lsm.get(b"cf1", b"k"), Some(b"1".to_vec()));
        assert_eq!(lsm.get(b"cf2", b"k"), Some(b"2".to_vec()));
    }

    #[test]
    fn delete() {
        let mut lsm = SparLsm::new(100);
        lsm.put(b"cf1", b"k", b"v".to_vec());
        lsm.delete(b"cf1", b"k");
        assert_eq!(lsm.get(b"cf1", b"k"), None);
    }

    #[test]
    fn flush_and_read() {
        let mut lsm = SparLsm::new(2);
        lsm.put(b"cf", b"a", b"1".to_vec());
        lsm.put(b"cf", b"b", b"2".to_vec());
        lsm.put(b"cf", b"c", b"3".to_vec());
        assert_eq!(lsm.get(b"cf", b"a"), Some(b"1".to_vec()));
        assert!(lsm.level_count(b"cf") > 0);
    }

    #[test]
    fn gc_removes_tombstones() {
        let mut lsm = SparLsm::new(2);
        lsm.put(b"cf", b"k", b"v".to_vec());
        lsm.put(b"cf", b"x", b"y".to_vec());
        lsm.delete(b"cf", b"k");
        lsm.gc(b"cf");
        assert_eq!(lsm.get(b"cf", b"k"), None);
    }

    #[test]
    fn cf_names() {
        let mut lsm = SparLsm::new(10);
        lsm.put(b"a", b"k", vec![]); lsm.put(b"b", b"k", vec![]);
        assert_eq!(lsm.cf_names().len(), 2);
    }

    #[test]
    fn stats() {
        let mut lsm = SparLsm::new(10);
        lsm.put(b"cf", b"k", b"v".to_vec()); lsm.get(b"cf", b"k");
        assert_eq!(lsm.total_puts(), 1);
        assert_eq!(lsm.total_gets(), 1);
    }
}

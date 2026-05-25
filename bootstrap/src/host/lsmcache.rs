use std::collections::BTreeMap;

#[derive(Clone)]
struct Layer {
    data: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    level: usize,
}

pub struct LsmCache {
    memtable: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    levels: Vec<Layer>,
    memtable_cap: usize,
    total_puts: u64,
    total_gets: u64,
    total_flushes: u64,
    total_compactions: u64,
}

impl LsmCache {
    pub fn new(memtable_cap: usize) -> Self { Self { memtable: BTreeMap::new(), levels: Vec::new(), memtable_cap, total_puts: 0, total_gets: 0, total_flushes: 0, total_compactions: 0 } }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.total_puts += 1;
        self.memtable.insert(key, Some(value));
        if self.memtable.len() >= self.memtable_cap { self.flush(); }
    }

    pub fn delete(&mut self, key: &[u8]) {
        self.total_puts += 1;
        self.memtable.insert(key.to_vec(), None);
        if self.memtable.len() >= self.memtable_cap { self.flush(); }
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.total_gets += 1;
        if let Some(v) = self.memtable.get(key) { return v.clone(); }
        for layer in &self.levels {
            if let Some(v) = layer.data.get(key) { return v.clone(); }
        }
        None
    }

    fn flush(&mut self) {
        self.total_flushes += 1;
        let data = std::mem::take(&mut self.memtable);
        self.levels.push(Layer { data, level: 0 });
        self.maybe_compact();
    }

    fn maybe_compact(&mut self) {
        let max_per_level = 4;
        let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
        for l in &self.levels { *counts.entry(l.level).or_insert(0) += 1; }
        for (&level, &count) in &counts {
            if count > max_per_level {
                self.compact_level(level);
                return;
            }
        }
    }

    fn compact_level(&mut self, target_level: usize) {
        self.total_compactions += 1;
        let mut merged = BTreeMap::new();
        let next_level = target_level + 1;
        let mut remaining = Vec::new();
        for layer in self.levels.drain(..) {
            if layer.level == target_level {
                for (k, v) in layer.data { merged.insert(k, v); }
            } else { remaining.push(layer); }
        }
        for layer in &remaining {
            if layer.level == next_level {
                for (k, v) in &layer.data { merged.entry(k.clone()).or_insert(v.clone()); }
            }
        }
        remaining.retain(|l| l.level != next_level);
        remaining.push(Layer { data: merged, level: next_level });
        self.levels = remaining;
    }

    pub fn level_count(&self) -> usize { self.levels.len() }
    pub fn memtable_len(&self) -> usize { self.memtable.len() }
    pub fn total_entries(&self) -> usize {
        let mt = self.memtable.iter().filter(|(_, v)| v.is_some()).count();
        let lv = self.levels.iter().flat_map(|l| l.data.iter()).filter(|(_, v)| v.is_some()).count();
        mt + lv
    }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_flushes(&self) -> u64 { self.total_flushes }
    pub fn total_compactions(&self) -> u64 { self.total_compactions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get() {
        let mut lc = LsmCache::new(100);
        lc.put(b"k".to_vec(), b"v".to_vec());
        assert_eq!(lc.get(b"k"), Some(b"v".to_vec()));
    }

    #[test]
    fn delete() {
        let mut lc = LsmCache::new(100);
        lc.put(b"k".to_vec(), b"v".to_vec());
        lc.delete(b"k");
        assert_eq!(lc.get(b"k"), None);
    }

    #[test]
    fn flush_to_level() {
        let mut lc = LsmCache::new(3);
        for i in 0..5u64 { lc.put(i.to_le_bytes().to_vec(), b"v".to_vec()); }
        assert!(lc.level_count() > 0);
    }

    #[test]
    fn get_from_level() {
        let mut lc = LsmCache::new(2);
        lc.put(b"a".to_vec(), b"1".to_vec());
        lc.put(b"b".to_vec(), b"2".to_vec());
        lc.put(b"c".to_vec(), b"3".to_vec());
        assert_eq!(lc.get(b"a"), Some(b"1".to_vec()));
    }

    #[test]
    fn missing() { assert_eq!(LsmCache::new(10).get(b"x"), None); }

    #[test]
    fn total_entries() {
        let mut lc = LsmCache::new(100);
        lc.put(b"a".to_vec(), b"1".to_vec()); lc.put(b"b".to_vec(), b"2".to_vec());
        assert_eq!(lc.total_entries(), 2);
    }

    #[test]
    fn stats() {
        let mut lc = LsmCache::new(2);
        lc.put(b"x".to_vec(), b"v".to_vec()); lc.get(b"x");
        assert_eq!(lc.total_puts(), 1);
        assert_eq!(lc.total_gets(), 1);
    }
}

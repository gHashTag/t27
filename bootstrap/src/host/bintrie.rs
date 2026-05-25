#[derive(Debug, Clone, PartialEq)]
pub enum BtError {
    Empty,
}

impl std::fmt::Display for BtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BtError::Empty => write!(f, "trie empty"),
        }
    }
}

impl std::error::Error for BtError {}

struct Btn {
    children: [Option<usize>; 2],
    is_end: bool,
    count: usize,
}

pub struct BinTrie {
    nodes: Vec<Btn>,
    bits: u8,
    total_inserts: u64,
    total_removes: u64,
    total_queries: u64,
}

impl BinTrie {
    pub fn new(bits: u8) -> Self {
        Self { nodes: vec![Btn { children: [None, None], is_end: false, count: 0 }], bits, total_inserts: 0, total_removes: 0, total_queries: 0 }
    }

    fn bit_at(&self, key: u64, pos: u8) -> usize { ((key >> (self.bits - 1 - pos)) & 1) as usize }

    pub fn insert(&mut self, key: u64) {
        self.total_inserts += 1;
        let mut cur = 0;
        for i in 0..self.bits {
            let b = self.bit_at(key, i);
            if self.nodes[cur].children[b].is_none() {
                self.nodes.push(Btn { children: [None, None], is_end: false, count: 0 });
                self.nodes[cur].children[b] = Some(self.nodes.len() - 1);
            }
            cur = self.nodes[cur].children[b].unwrap();
        }
        if !self.nodes[cur].is_end {
            self.nodes[cur].is_end = true;
            let mut c = 0;
            for i in 0..self.bits {
                let b = self.bit_at(key, i);
                c = self.nodes[c].children[b].unwrap();
                self.nodes[c].count += 1;
            }
        }
    }

    pub fn contains(&mut self, key: u64) -> bool {
        self.total_queries += 1;
        let mut cur = 0;
        for i in 0..self.bits {
            let b = self.bit_at(key, i);
            match self.nodes[cur].children[b] {
                Some(n) => cur = n,
                None => return false,
            }
        }
        self.nodes[cur].is_end
    }

    pub fn remove(&mut self, key: u64) -> bool {
        self.total_removes += 1;
        let mut path = vec![0];
        let mut cur = 0;
        for i in 0..self.bits {
            let b = self.bit_at(key, i);
            match self.nodes[cur].children[b] {
                Some(n) => { cur = n; path.push(cur); }
                None => return false,
            }
        }
        if !self.nodes[cur].is_end { return false; }
        self.nodes[cur].is_end = false;
        for i in 0..self.bits {
            let b = self.bit_at(key, i);
            let node_idx = path[i + 1];
            self.nodes[node_idx].count -= 1;
        }
        true
    }

    pub fn min_xor(&mut self, key: u64) -> Option<u64> {
        self.total_queries += 1;
        if self.nodes[0].count == 0 { return None; }
        let mut cur = 0;
        let mut result = 0u64;
        for i in 0..self.bits {
            let b = self.bit_at(key, i);
            let prefer = b;
            let alt = 1 - b;
            if self.nodes[cur].children[prefer].map(|n| self.nodes[n].count > 0).unwrap_or(false) {
                result |= (prefer as u64) << (self.bits - 1 - i);
                cur = self.nodes[cur].children[prefer].unwrap();
            } else {
                result |= (alt as u64) << (self.bits - 1 - i);
                cur = self.nodes[cur].children[alt].unwrap();
            }
        }
        Some(result)
    }

    pub fn max_xor(&mut self, key: u64) -> Option<u64> {
        self.total_queries += 1;
        if self.nodes[0].count == 0 { return None; }
        let mut cur = 0;
        let mut result = 0u64;
        for i in 0..self.bits {
            let b = self.bit_at(key, i);
            let prefer = 1 - b;
            if self.nodes[cur].children[prefer].map(|n| self.nodes[n].count > 0).unwrap_or(false) {
                result |= (prefer as u64) << (self.bits - 1 - i);
                cur = self.nodes[cur].children[prefer].unwrap();
            } else {
                let alt = b;
                result |= (alt as u64) << (self.bits - 1 - i);
                cur = self.nodes[cur].children[alt].unwrap();
            }
        }
        Some(result)
    }

    pub fn count(&self) -> usize { self.nodes[0].count }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bt() { let bt = BinTrie::new(8); assert_eq!(bt.count(), 0); }

    #[test]
    fn insert_contains() {
        let mut bt = BinTrie::new(8);
        bt.insert(42); bt.insert(100);
        assert!(bt.contains(42)); assert!(bt.contains(100));
        assert!(!bt.contains(50));
    }

    #[test]
    fn remove() {
        let mut bt = BinTrie::new(8);
        bt.insert(42);
        assert!(bt.remove(42));
        assert!(!bt.contains(42));
        assert_eq!(bt.count(), 0);
    }

    #[test]
    fn remove_not_present() {
        let mut bt = BinTrie::new(8);
        bt.insert(1);
        assert!(!bt.remove(99));
    }

    #[test]
    fn min_xor() {
        let mut bt = BinTrie::new(4);
        for v in [0b0000, 0b0101, 0b1010, 0b1111] { bt.insert(v); }
        let result = bt.min_xor(0b0101).unwrap();
        assert_eq!(result, 0b0101);
    }

    #[test]
    fn max_xor() {
        let mut bt = BinTrie::new(4);
        for v in [0b0000, 0b0101, 0b1010] { bt.insert(v); }
        let result = bt.max_xor(0b0101).unwrap();
        assert_eq!(result ^ 0b0101, 0b1111);
    }

    #[test]
    fn empty_xor() { assert!(BinTrie::new(4).max_xor(0).is_none()); }

    #[test]
    fn many() {
        let mut bt = BinTrie::new(16);
        for i in 0..100u64 { bt.insert(i); }
        assert_eq!(bt.count(), 100);
        for i in 0..100u64 { assert!(bt.contains(i)); }
    }

    #[test]
    fn stats() {
        let mut bt = BinTrie::new(8);
        bt.insert(1); bt.contains(1); bt.remove(1);
        assert_eq!(bt.total_inserts(), 1);
        assert_eq!(bt.total_queries(), 1);
        assert_eq!(bt.total_removes(), 1);
    }

    #[test]
    fn error_display() { assert!(BtError::Empty.to_string().contains("empty")); }
}

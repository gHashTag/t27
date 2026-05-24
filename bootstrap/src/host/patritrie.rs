#[derive(Debug, Clone, PartialEq)]
pub enum PtError {
    NotFound { key: u64 },
    NoMatch,
}

impl std::fmt::Display for PtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtError::NotFound { key } => write!(f, "key {key} not found"),
            PtError::NoMatch => write!(f, "no matching prefix"),
        }
    }
}

impl std::error::Error for PtError {}

struct PtNode {
    key: u64,
    mask: u64,
    value: Option<Vec<u8>>,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct PatriTrie {
    nodes: Vec<PtNode>,
    root: Option<usize>,
    total_inserts: u64,
    total_lookups: u64,
}

impl PatriTrie {
    pub fn new() -> Self { Self { nodes: Vec::new(), root: None, total_inserts: 0, total_lookups: 0 } }

    fn bit(&self, key: u64, bit: u8) -> bool { (key >> (63 - bit)) & 1 == 1 }

    fn first_diff_bit(&self, a: u64, b: u64) -> u8 {
        let xor = a ^ b;
        if xor == 0 { return 64; }
        63 - xor.leading_zeros() as u8
    }

    fn alloc(&mut self, key: u64, mask: u64, value: Option<Vec<u8>>) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(PtNode { key, mask, value, left: None, right: None });
        idx
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let new_idx = self.alloc(key, 0, Some(value));
        match self.root {
            None => { self.root = Some(new_idx); return; }
            Some(root) => {
                let mut cur = root;
                loop {
                    let diff = self.first_diff_bit(key, self.nodes[cur].key);
                    if self.nodes[cur].left.is_none() && self.nodes[cur].right.is_none() {
                        if diff == 64 { self.nodes[cur].value = self.nodes[new_idx].value.clone(); self.nodes.pop(); return; }
                        let split = self.alloc(key & !((1u64 << (63 - diff)) - 1), 0, None);
                        self.nodes[split].mask = diff;
                        if self.bit(key, diff) {
                            self.nodes[split].right = Some(new_idx);
                            self.nodes[split].left = Some(cur);
                        } else {
                            self.nodes[split].left = Some(new_idx);
                            self.nodes[split].right = Some(cur);
                        }
                        self.root = Some(split);
                        return;
                    }
                    let bit = self.bit(key, self.nodes[cur].mask);
                    let next = if bit { self.nodes[cur].right } else { self.nodes[cur].left };
                    match next {
                        Some(n) => { cur = n; }
                        None => {
                            if bit { self.nodes[cur].right = Some(new_idx); }
                            else { self.nodes[cur].left = Some(new_idx); }
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let root = self.root?;
        let mut cur = root;
        loop {
            let bit = self.bit(key, self.nodes[cur].mask);
            let next = if bit { self.nodes[cur].right } else { self.nodes[cur].left };
            match next {
                Some(n) => { cur = n; }
                None => {
                    return self.nodes[cur].value.as_deref();
                }
            }
            if self.nodes[cur].left.is_none() && self.nodes[cur].right.is_none() {
                return self.nodes[cur].value.as_deref();
            }
        }
    }

    pub fn longest_prefix(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let root = self.root?;
        let mut best: Option<usize> = None;
        let mut cur = root;
        loop {
            if self.nodes[cur].value.is_some() { best = Some(cur); }
            if self.nodes[cur].left.is_none() && self.nodes[cur].right.is_none() { break; }
            let bit = self.bit(key, self.nodes[cur].mask);
            let next = if bit { self.nodes[cur].right } else { self.nodes[cur].left };
            match next {
                Some(n) => { cur = n; }
                None => break,
            }
        }
        best.and_then(|idx| self.nodes[idx].value.as_deref())
    }

    pub fn len(&self) -> usize { self.nodes.iter().filter(|n| n.value.is_some()).count() }
    pub fn is_empty(&self) -> bool { self.root.is_none() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

impl Default for PatriTrie {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pt() { assert!(PatriTrie::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut pt = PatriTrie::new();
        pt.insert(0xFF00000000000000, b"prefix1".to_vec());
        pt.insert(0xFE00000000000000, b"prefix2".to_vec());
        assert_eq!(pt.get(0xFF00000000000000), Some(&b"prefix1"[..]));
    }

    #[test]
    fn two_keys() {
        let mut pt = PatriTrie::new();
        pt.insert(1, b"one".to_vec());
        pt.insert(2, b"two".to_vec());
        assert_eq!(pt.get(1), Some(&b"one"[..]));
        assert_eq!(pt.get(2), Some(&b"two"[..]));
    }

    #[test]
    fn many() {
        let mut pt = PatriTrie::new();
        for i in 0..50u64 { pt.insert(i, vec![i as u8]); }
        assert_eq!(pt.len(), 50);
        for i in 0..50u64 { assert!(pt.get(i).is_some()); }
    }

    #[test]
    fn overwrite() {
        let mut pt = PatriTrie::new();
        pt.insert(1, b"old".to_vec()); pt.insert(1, b"new".to_vec());
        assert_eq!(pt.get(1), Some(&b"new"[..]));
        assert_eq!(pt.len(), 1);
    }

    #[test]
    fn empty_get() { assert_eq!(PatriTrie::new().get(1), None); }

    #[test]
    fn stats() {
        let mut pt = PatriTrie::new();
        pt.insert(1, vec![]); pt.get(1);
        assert_eq!(pt.total_inserts(), 1);
        assert_eq!(pt.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(PtError::NoMatch.to_string().contains("no matching")); }
}

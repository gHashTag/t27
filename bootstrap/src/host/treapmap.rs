#[derive(Debug, Clone, PartialEq)]
pub enum TrError {
    NotFound { key: u64 },
}

impl std::fmt::Display for TrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for TrError {}

struct TNode {
    key: u64,
    value: Vec<u8>,
    priority: u64,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct TreapMap {
    nodes: Vec<TNode>,
    root: Option<usize>,
    free: Vec<usize>,
    rng: u64,
    total_inserts: u64,
    total_removes: u64,
    total_rotations: u64,
}

impl TreapMap {
    pub fn new() -> Self { Self { nodes: Vec::new(), root: None, free: Vec::new(), rng: 0xABCDEF0123456789, total_inserts: 0, total_removes: 0, total_rotations: 0 } }

    fn next_prio(&mut self) -> u64 {
        self.rng = self.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.rng
    }

    fn alloc(&mut self, key: u64, value: Vec<u8>, priority: u64) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = TNode { key, value, priority, left: None, right: None };
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(TNode { key, value, priority, left: None, right: None });
        idx
    }

    fn rotate_right(&mut self, idx: usize) -> usize {
        let left = self.nodes[idx].left.unwrap();
        self.nodes[idx].left = self.nodes[left].right;
        self.nodes[left].right = Some(idx);
        self.total_rotations += 1;
        left
    }

    fn rotate_left(&mut self, idx: usize) -> usize {
        let right = self.nodes[idx].right.unwrap();
        self.nodes[idx].right = self.nodes[right].left;
        self.nodes[right].left = Some(idx);
        self.total_rotations += 1;
        right
    }

    fn bubble_up(&mut self, mut idx: usize) -> usize {
        while let Some(parent_idx) = self.find_parent(idx) {
            if self.nodes[idx].priority <= self.nodes[parent_idx].priority { break; }
            let grandparent = self.find_parent(parent_idx);
            let is_left = self.nodes[parent_idx].left == Some(idx);
            let new_idx = if is_left { self.rotate_right(parent_idx) } else { self.rotate_left(parent_idx) };
            idx = new_idx;
            if let Some(gp) = grandparent {
                if self.nodes[gp].left == Some(parent_idx) { self.nodes[gp].left = Some(idx); }
                else { self.nodes[gp].right = Some(idx); }
            } else {
                self.root = Some(idx);
            }
        }
        idx
    }

    fn find_parent(&self, child: usize) -> Option<usize> {
        let key = self.nodes[child].key;
        let mut cur = self.root?;
        let mut parent: Option<usize> = None;
        loop {
            if key < self.nodes[cur].key {
                parent = Some(cur);
                cur = self.nodes[cur].left?;
            } else if key > self.nodes[cur].key {
                parent = Some(cur);
                cur = self.nodes[cur].right?;
            } else {
                return parent;
            }
        }
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let prio = self.next_prio();
        let new_idx = self.alloc(key, value, prio);
        match self.root {
            None => { self.root = Some(new_idx); return; }
            Some(_) => {}
        }
        let mut cur = self.root.unwrap();
        loop {
            if key < self.nodes[cur].key {
                match self.nodes[cur].left {
                    Some(l) => cur = l,
                    None => { self.nodes[cur].left = Some(new_idx); break; }
                }
            } else if key > self.nodes[cur].key {
                match self.nodes[cur].right {
                    Some(r) => cur = r,
                    None => { self.nodes[cur].right = Some(new_idx); break; }
                }
            } else {
                self.nodes[cur].value = self.nodes[new_idx].value.clone();
                self.nodes[cur].priority = prio;
                self.free.push(new_idx);
                self.bubble_up(cur);
                return;
            }
        }
        self.bubble_up(new_idx);
    }

    pub fn get(&self, key: u64) -> Option<&[u8]> {
        let mut cur = self.root;
        while let Some(ci) = cur {
            if key < self.nodes[ci].key { cur = self.nodes[ci].left; }
            else if key > self.nodes[ci].key { cur = self.nodes[ci].right; }
            else { return Some(&self.nodes[ci].value); }
        }
        None
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, TrError> {
        self.total_removes += 1;
        let idx = {
            let mut cur = self.root.ok_or(TrError::NotFound { key })?;
            loop {
                if key < self.nodes[cur].key { cur = self.nodes[cur].left.ok_or(TrError::NotFound { key })?; }
                else if key > self.nodes[cur].key { cur = self.nodes[cur].right.ok_or(TrError::NotFound { key })?; }
                else { break cur; }
            }
        };
        self.push_down(idx);
        let parent = self.find_parent(idx);
        match parent {
            None => { self.root = None; }
            Some(p) => {
                if self.nodes[p].left == Some(idx) { self.nodes[p].left = None; }
                else { self.nodes[p].right = None; }
            }
        }
        let value = std::mem::take(&mut self.nodes[idx].value);
        self.free.push(idx);
        Ok(value)
    }

    fn push_down(&mut self, mut idx: usize) {
        loop {
            let left = self.nodes[idx].left;
            let right = self.nodes[idx].right;
            if left.is_none() && right.is_none() { break; }
            let left_prio = left.map(|l| self.nodes[l].priority).unwrap_or(0);
            let right_prio = right.map(|r| self.nodes[r].priority).unwrap_or(0);
            let parent = self.find_parent(idx);
            if left_prio >= right_prio && left.is_some() {
                let l = left.unwrap();
                self.nodes[idx].left = self.nodes[l].right;
                self.nodes[l].right = Some(idx);
                match parent {
                    None => { self.root = Some(l); }
                    Some(p) => {
                        if self.nodes[p].left == Some(idx) { self.nodes[p].left = Some(l); }
                        else { self.nodes[p].right = Some(l); }
                    }
                }
                self.total_rotations += 1;
            } else if right.is_some() {
                let r = right.unwrap();
                self.nodes[idx].right = self.nodes[r].left;
                self.nodes[r].left = Some(idx);
                match parent {
                    None => { self.root = Some(r); }
                    Some(p) => {
                        if self.nodes[p].left == Some(idx) { self.nodes[p].left = Some(r); }
                        else { self.nodes[p].right = Some(r); }
                    }
                }
                self.total_rotations += 1;
            }
        }
    }

    pub fn len(&self) -> usize { self.nodes.len() - self.free.len() }
    pub fn is_empty(&self) -> bool { self.root.is_none() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_rotations(&self) -> u64 { self.total_rotations }
}

impl Default for TreapMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tm() { assert!(TreapMap::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut tm = TreapMap::new();
        tm.insert(1, b"one".to_vec()); tm.insert(2, b"two".to_vec());
        assert_eq!(tm.get(1), Some(&b"one"[..]));
        assert_eq!(tm.get(2), Some(&b"two"[..]));
        assert_eq!(tm.get(3), None);
    }

    #[test]
    fn overwrite() {
        let mut tm = TreapMap::new();
        tm.insert(1, b"old".to_vec()); tm.insert(1, b"new".to_vec());
        assert_eq!(tm.get(1), Some(&b"new"[..]));
        assert_eq!(tm.len(), 1);
    }

    #[test]
    fn remove() {
        let mut tm = TreapMap::new();
        tm.insert(1, b"a".to_vec()); tm.insert(2, b"b".to_vec());
        let v = tm.remove(1).unwrap();
        assert_eq!(v, b"a".to_vec());
        assert_eq!(tm.get(1), None);
    }

    #[test]
    fn remove_not_found() { assert!(TreapMap::new().remove(1).is_err()); }

    #[test]
    fn many() {
        let mut tm = TreapMap::new();
        for i in 0..100u64 { tm.insert(i, vec![i as u8]); }
        assert_eq!(tm.len(), 100);
        for i in 0..100u64 { assert_eq!(tm.get(i), Some(&[i as u8][..])); }
    }

    #[test]
    fn rotations_occur() {
        let mut tm = TreapMap::new();
        for i in 0..50 { tm.insert(i, vec![]); }
        assert!(tm.total_rotations() > 0);
    }

    #[test]
    fn remove_many() {
        let mut tm = TreapMap::new();
        for i in 0..30u64 { tm.insert(i, vec![]); }
        for i in 0..30u64 { tm.remove(i).unwrap(); }
        assert!(tm.is_empty());
    }

    #[test]
    fn stats() {
        let mut tm = TreapMap::new();
        tm.insert(1, vec![]); tm.remove(1).unwrap();
        assert_eq!(tm.total_inserts(), 1);
        assert_eq!(tm.total_removes(), 1);
    }

    #[test]
    fn error_display() { assert!(TrError::NotFound { key: 1 }.to_string().contains("not found")); }
}

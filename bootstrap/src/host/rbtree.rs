#[derive(Debug, Clone, Copy, PartialEq)]
enum Color { Red, Black }

#[derive(Debug, Clone, PartialEq)]
pub enum RbError {
    NotFound { key: u64 },
}

impl std::fmt::Display for RbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RbError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for RbError {}

struct RbNode {
    key: u64,
    value: Vec<u8>,
    color: Color,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

const SENTINEL: usize = 0;

pub struct RbTree {
    nodes: Vec<RbNode>,
    root: usize,
    free: Vec<usize>,
    total_inserts: u64,
    total_removes: u64,
    total_fixups: u64,
}

impl RbTree {
    pub fn new() -> Self {
        let sentinel = RbNode { key: 0, value: Vec::new(), color: Color::Black, left: None, right: None, parent: None };
        Self { nodes: vec![sentinel], root: SENTINEL, free: Vec::new(), total_inserts: 0, total_removes: 0, total_fixups: 0 }
    }

    fn alloc(&mut self, key: u64, value: Vec<u8>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = RbNode { key, value, color: Color::Red, left: Some(SENTINEL), right: Some(SENTINEL), parent: None };
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(RbNode { key, value, color: Color::Red, left: Some(SENTINEL), right: Some(SENTINEL), parent: None });
        idx
    }

    fn set_left(&mut self, parent: usize, child: usize) { self.nodes[parent].left = Some(child); self.nodes[child].parent = Some(parent); }
    fn set_right(&mut self, parent: usize, child: usize) { self.nodes[parent].right = Some(child); self.nodes[child].parent = Some(parent); }

    fn rotate_left(&mut self, x: usize) {
        let y = self.nodes[x].right.unwrap();
        self.nodes[x].right = self.nodes[y].left;
        if let Some(yl) = self.nodes[y].left { self.nodes[yl].parent = Some(x); }
        self.nodes[y].parent = self.nodes[x].parent;
        match self.nodes[x].parent {
            None => { self.root = y; }
            Some(p) => {
                if self.nodes[p].left == Some(x) { self.nodes[p].left = Some(y); }
                else { self.nodes[p].right = Some(y); }
            }
        }
        self.nodes[y].left = Some(x);
        self.nodes[x].parent = Some(y);
        self.total_fixups += 1;
    }

    fn rotate_right(&mut self, y: usize) {
        let x = self.nodes[y].left.unwrap();
        self.nodes[y].left = self.nodes[x].right;
        if let Some(xr) = self.nodes[x].right { self.nodes[xr].parent = Some(y); }
        self.nodes[x].parent = self.nodes[y].parent;
        match self.nodes[y].parent {
            None => { self.root = x; }
            Some(p) => {
                if self.nodes[p].left == Some(y) { self.nodes[p].left = Some(x); }
                else { self.nodes[p].right = Some(x); }
            }
        }
        self.nodes[x].right = Some(y);
        self.nodes[y].parent = Some(x);
        self.total_fixups += 1;
    }

    fn insert_fixup(&mut self, mut z: usize) {
        while self.nodes[z].parent.map(|p| self.nodes[p].color) == Some(Color::Red) {
            let p = self.nodes[z].parent.unwrap();
            let gp = self.nodes[p].parent.unwrap();
            if self.nodes[gp].left == Some(p) {
                let uncle = self.nodes[gp].right.unwrap_or(SENTINEL);
                if self.nodes[uncle].color == Color::Red {
                    self.nodes[p].color = Color::Black;
                    self.nodes[uncle].color = Color::Black;
                    self.nodes[gp].color = Color::Red;
                    z = gp;
                } else {
                    if self.nodes[p].right == Some(z) { z = p; self.rotate_left(z); }
                    let p2 = self.nodes[z].parent.unwrap();
                    let gp2 = self.nodes[p2].parent.unwrap();
                    self.nodes[p2].color = Color::Black;
                    self.nodes[gp2].color = Color::Red;
                    self.rotate_right(gp2);
                }
            } else {
                let uncle = self.nodes[gp].left.unwrap_or(SENTINEL);
                if self.nodes[uncle].color == Color::Red {
                    self.nodes[p].color = Color::Black;
                    self.nodes[uncle].color = Color::Black;
                    self.nodes[gp].color = Color::Red;
                    z = gp;
                } else {
                    if self.nodes[p].left == Some(z) { z = p; self.rotate_right(z); }
                    let p2 = self.nodes[z].parent.unwrap();
                    let gp2 = self.nodes[p2].parent.unwrap();
                    self.nodes[p2].color = Color::Black;
                    self.nodes[gp2].color = Color::Red;
                    self.rotate_left(gp2);
                }
            }
        }
        self.nodes[self.root].color = Color::Black;
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let mut cur = self.root;
        let mut parent: Option<usize> = None;
        while cur != SENTINEL {
            parent = Some(cur);
            if key < self.nodes[cur].key { cur = self.nodes[cur].left.unwrap_or(SENTINEL); }
            else if key > self.nodes[cur].key { cur = self.nodes[cur].right.unwrap_or(SENTINEL); }
            else { self.nodes[cur].value = value; return; }
        }
        let z = self.alloc(key, value);
        match parent {
            None => { self.root = z; self.nodes[z].parent = None; }
            Some(p) => {
                if key < self.nodes[p].key { self.set_left(p, z); } else { self.set_right(p, z); }
            }
        }
        self.nodes[z].left = Some(SENTINEL);
        self.nodes[z].right = Some(SENTINEL);
        self.nodes[z].color = Color::Red;
        self.insert_fixup(z);
    }

    pub fn get(&self, key: u64) -> Option<&[u8]> {
        let mut cur = self.root;
        while cur != SENTINEL {
            if key < self.nodes[cur].key { cur = self.nodes[cur].left.unwrap_or(SENTINEL); }
            else if key > self.nodes[cur].key { cur = self.nodes[cur].right.unwrap_or(SENTINEL); }
            else { return Some(&self.nodes[cur].value); }
        }
        None
    }

    fn transplant(&mut self, u: usize, v: usize) {
        match self.nodes[u].parent {
            None => { self.root = v; }
            Some(p) => {
                if self.nodes[p].left == Some(u) { self.nodes[p].left = Some(v); }
                else { self.nodes[p].right = Some(v); }
            }
        }
        self.nodes[v].parent = self.nodes[u].parent;
    }

    fn minimum(&self, mut idx: usize) -> usize {
        while let Some(l) = self.nodes[idx].left { if l == SENTINEL { break; } idx = l; }
        idx
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, RbError> {
        self.total_removes += 1;
        let z = {
            let mut cur = self.root;
            loop {
                if cur == SENTINEL { return Err(RbError::NotFound { key }); }
                if key < self.nodes[cur].key { cur = self.nodes[cur].left.unwrap_or(SENTINEL); }
                else if key > self.nodes[cur].key { cur = self.nodes[cur].right.unwrap_or(SENTINEL); }
                else { break cur; }
            }
        };
        let z_color = self.nodes[z].color;
        let y;
        let x;
        if self.nodes[z].left == Some(SENTINEL) || self.nodes[z].left.is_none() {
            x = self.nodes[z].right.unwrap_or(SENTINEL);
            self.transplant(z, x);
            y = z;
        } else if self.nodes[z].right == Some(SENTINEL) || self.nodes[z].right.is_none() {
            x = self.nodes[z].left.unwrap_or(SENTINEL);
            self.transplant(z, x);
            y = z;
        } else {
            y = self.minimum(self.nodes[z].right.unwrap());
            let y_color = self.nodes[y].color;
            x = self.nodes[y].right.unwrap_or(SENTINEL);
            if self.nodes[y].parent == Some(z) {
                self.nodes[x].parent = Some(y);
            } else {
                self.transplant(y, x);
                let zr = self.nodes[z].right.unwrap();
                self.nodes[y].right = Some(zr);
                self.nodes[zr].parent = Some(y);
            }
            self.transplant(z, y);
            let zl = self.nodes[z].left.unwrap();
            self.nodes[y].left = Some(zl);
            self.nodes[zl].parent = Some(y);
            self.nodes[y].color = self.nodes[z].color;
            if y_color == Color::Black { self.delete_fixup(x); }
            let value = std::mem::take(&mut self.nodes[z].value);
            self.free.push(z);
            return Ok(value);
        }
        if z_color == Color::Black { self.delete_fixup(x); }
        let value = std::mem::take(&mut self.nodes[z].value);
        self.free.push(z);
        Ok(value)
    }

    fn delete_fixup(&mut self, mut x: usize) {
        while x != self.root && self.nodes[x].color == Color::Black {
            let p = self.nodes[x].parent.unwrap();
            if self.nodes[p].left == Some(x) {
                let mut w = self.nodes[p].right.unwrap_or(SENTINEL);
                if self.nodes[w].color == Color::Red {
                    self.nodes[w].color = Color::Black;
                    self.nodes[p].color = Color::Red;
                    self.rotate_left(p);
                    w = self.nodes[p].right.unwrap_or(SENTINEL);
                }
                let wl = self.nodes[w].left.unwrap_or(SENTINEL);
                let wr = self.nodes[w].right.unwrap_or(SENTINEL);
                if self.nodes[wl].color == Color::Black && self.nodes[wr].color == Color::Black {
                    self.nodes[w].color = Color::Red;
                    x = p;
                } else {
                    if self.nodes[wr].color == Color::Black {
                        self.nodes[wl].color = Color::Black;
                        self.nodes[w].color = Color::Red;
                        self.rotate_right(w);
                        w = self.nodes[p].right.unwrap_or(SENTINEL);
                    }
                    self.nodes[w].color = self.nodes[p].color;
                    self.nodes[p].color = Color::Black;
                    let wr2 = self.nodes[w].right.unwrap_or(SENTINEL);
                    self.nodes[wr2].color = Color::Black;
                    self.rotate_left(p);
                    x = self.root;
                }
            } else {
                let mut w = self.nodes[p].left.unwrap_or(SENTINEL);
                if self.nodes[w].color == Color::Red {
                    self.nodes[w].color = Color::Black;
                    self.nodes[p].color = Color::Red;
                    self.rotate_right(p);
                    w = self.nodes[p].left.unwrap_or(SENTINEL);
                }
                let wl = self.nodes[w].left.unwrap_or(SENTINEL);
                let wr = self.nodes[w].right.unwrap_or(SENTINEL);
                if self.nodes[wl].color == Color::Black && self.nodes[wr].color == Color::Black {
                    self.nodes[w].color = Color::Red;
                    x = p;
                } else {
                    if self.nodes[wl].color == Color::Black {
                        self.nodes[wr].color = Color::Black;
                        self.nodes[w].color = Color::Red;
                        self.rotate_left(w);
                        w = self.nodes[p].left.unwrap_or(SENTINEL);
                    }
                    self.nodes[w].color = self.nodes[p].color;
                    self.nodes[p].color = Color::Black;
                    let wl2 = self.nodes[w].left.unwrap_or(SENTINEL);
                    self.nodes[wl2].color = Color::Black;
                    self.rotate_right(p);
                    x = self.root;
                }
            }
        }
        self.nodes[x].color = Color::Black;
    }

    pub fn len(&self) -> usize { self.nodes.len() - 1 - self.free.len() }
    pub fn is_empty(&self) -> bool { self.root == SENTINEL }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_fixups(&self) -> u64 { self.total_fixups }
}

impl Default for RbTree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rb() { assert!(RbTree::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut rb = RbTree::new();
        rb.insert(1, b"one".to_vec()); rb.insert(2, b"two".to_vec()); rb.insert(3, b"three".to_vec());
        assert_eq!(rb.get(1), Some(&b"one"[..]));
        assert_eq!(rb.get(2), Some(&b"two"[..]));
        assert_eq!(rb.get(3), Some(&b"three"[..]));
    }

    #[test]
    fn overwrite() {
        let mut rb = RbTree::new();
        rb.insert(1, b"old".to_vec()); rb.insert(1, b"new".to_vec());
        assert_eq!(rb.get(1), Some(&b"new"[..]));
        assert_eq!(rb.len(), 1);
    }

    #[test]
    fn remove() {
        let mut rb = RbTree::new();
        rb.insert(1, b"a".to_vec()); rb.insert(2, b"b".to_vec());
        let v = rb.remove(1).unwrap();
        assert_eq!(v, b"a".to_vec());
        assert_eq!(rb.get(1), None);
    }

    #[test]
    fn remove_not_found() { assert!(RbTree::new().remove(1).is_err()); }

    #[test]
    fn many() {
        let mut rb = RbTree::new();
        for i in 0..200u64 { rb.insert(i, vec![i as u8]); }
        assert_eq!(rb.len(), 200);
        for i in 0..200u64 { assert_eq!(rb.get(i), Some(&[i as u8][..])); }
    }

    #[test]
    fn fixups() {
        let mut rb = RbTree::new();
        for i in 0..50 { rb.insert(i, vec![]); }
        assert!(rb.total_fixups() > 0);
    }

    #[test]
    fn remove_all() {
        let mut rb = RbTree::new();
        for i in 0..50u64 { rb.insert(i, vec![]); }
        for i in 0..50u64 { rb.remove(i).unwrap(); }
        assert!(rb.is_empty());
    }

    #[test]
    fn sequential_insert_remove() {
        let mut rb = RbTree::new();
        for i in (0..20u64).rev() { rb.insert(i, vec![]); }
        for i in 0..20u64 { assert!(rb.get(i).is_some()); }
        for i in 10..20u64 { rb.remove(i).unwrap(); }
        assert_eq!(rb.len(), 10);
    }

    #[test]
    fn stats() {
        let mut rb = RbTree::new();
        rb.insert(1, vec![]);
        assert_eq!(rb.total_inserts(), 1);
    }

    #[test]
    fn error_display() { assert!(RbError::NotFound { key: 1 }.to_string().contains("not found")); }
}

use std::cmp::Ordering;

pub struct BTree2<K: Ord, V> {
    root: Option<Node<K, V>>,
    len: usize,
}

struct Node<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<Box<Node<K, V>>>,
    leaf: bool,
}

impl<K: Ord, V> BTree2<K, V> {
    pub fn new() -> Self { Self { root: None, len: 0 } }

    pub fn insert(&mut self, key: K, value: V) {
        let new = self.root.take();
        match new {
            None => {
                self.root = Some(Node { keys: vec![key], values: vec![value], children: Vec::new(), leaf: true });
                self.len = 1;
            }
            Some(mut root) => {
                if root.keys.len() >= 3 {
                    let mut new_root = Node { keys: Vec::new(), values: Vec::new(), children: Vec::new(), leaf: false };
                    new_root.children.push(Box::new(root));
                    Self::split_child(&mut new_root, 0);
                    Self::insert_nonfull(&mut new_root, key, value, &mut self.len);
                    self.root = Some(new_root);
                } else {
                    Self::insert_nonfull(&mut root, key, value, &mut self.len);
                    self.root = Some(root);
                }
            }
        }
    }

    fn split_child(parent: &mut Node<K, V>, idx: usize) {
        let full = &mut parent.children[idx];
        let mid_key = full.keys.remove(1);
        let mid_val = full.values.remove(1);
        let mut right = Node {
            keys: vec![full.keys.remove(1)],
            values: vec![full.values.remove(1)],
            children: if full.leaf { Vec::new() } else {
                vec![full.children.remove(2), full.children.remove(2)]
            },
            leaf: full.leaf,
        };
        parent.keys.insert(idx, mid_key);
        parent.values.insert(idx, mid_val);
        parent.children.insert(idx + 1, Box::new(right));
    }

    fn insert_nonfull(node: &mut Node<K, V>, key: K, value: V, len: &mut usize) {
        let idx = node.keys.iter().position(|k| key.cmp(k) != Ordering::Greater).unwrap_or(node.keys.len());
        if idx < node.keys.len() && node.keys[idx] == key {
            node.values[idx] = value;
            return;
        }
        if node.leaf {
            node.keys.insert(idx, key);
            node.values.insert(idx, value);
            *len += 1;
        } else {
            if node.children[idx].keys.len() >= 3 {
                Self::split_child(node, idx);
                if key > node.keys[idx] {
                    Self::insert_nonfull(&mut node.children[idx + 1], key, value, len);
                } else {
                    Self::insert_nonfull(&mut node.children[idx], key, value, len);
                }
            } else {
                Self::insert_nonfull(&mut node.children[idx], key, value, len);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        Self::search(self.root.as_ref()?, key)
    }

    fn search<'a>(node: &'a Node<K, V>, key: &K) -> Option<&'a V> {
        let idx = node.keys.iter().position(|k| key.cmp(k) != Ordering::Greater).unwrap_or(node.keys.len());
        if idx < node.keys.len() && &node.keys[idx] == key { return Some(&node.values[idx]); }
        if node.leaf { None } else { Self::search(&node.children[idx], key) }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut bt = BTree2::new();
        bt.insert(5, 'a'); bt.insert(3, 'b'); bt.insert(7, 'c');
        assert_eq!(bt.get(&5), Some(&'a'));
        assert_eq!(bt.get(&3), Some(&'b'));
        assert_eq!(bt.get(&7), Some(&'c'));
        assert_eq!(bt.get(&1), None);
    }

    #[test]
    fn overwrite() {
        let mut bt = BTree2::new();
        bt.insert(1, 'x'); bt.insert(1, 'y');
        assert_eq!(bt.get(&1), Some(&'y'));
        assert_eq!(bt.len(), 1);
    }

    #[test]
    fn many_inserts() {
        let mut bt = BTree2::new();
        for i in 0..50 { bt.insert(i, i * 10); }
        assert_eq!(bt.len(), 50);
        for i in 0..50 { assert_eq!(bt.get(&i), Some(&(i * 10))); }
    }

    #[test]
    fn reverse_insert() {
        let mut bt = BTree2::new();
        for i in (0..20).rev() { bt.insert(i, i); }
        assert_eq!(bt.len(), 20);
        for i in 0..20 { assert_eq!(bt.get(&i), Some(&i)); }
    }

    #[test]
    fn empty() {
        let bt: BTree2<i32, i32> = BTree2::new();
        assert!(bt.is_empty());
        assert_eq!(bt.get(&1), None);
    }
}

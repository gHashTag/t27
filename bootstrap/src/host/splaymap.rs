use std::cmp::Ordering;

pub struct SplayMap<K: Ord, V> {
    root: Option<Box<Node<K, V>>>,
    len: usize,
}

struct Node<K, V> {
    key: K,
    value: V,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> SplayMap<K, V> {
    pub fn new() -> Self { Self { root: None, len: 0 } }

    pub fn insert(&mut self, key: K, value: V) {
        let new = Box::new(Node { key, value, left: None, right: None });
        self.root = Some(Self::insert_rec(self.root.take(), new, &mut self.len));
    }

    fn insert_rec(root: Option<Box<Node<K, V>>>, mut new: Box<Node<K, V>>, len: &mut usize) -> Box<Node<K, V>> {
        match root {
            None => { *len += 1; new }
            Some(mut node) => {
                match new.key.cmp(&node.key) {
                    Ordering::Less => node.left = Some(Self::insert_rec(node.left.take(), new, len)),
                    Ordering::Greater => node.right = Some(Self::insert_rec(node.right.take(), new, len)),
                    Ordering::Equal => { node.value = new.value; }
                }
                node
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut cur = self.root.as_ref();
        while let Some(node) = cur {
            match key.cmp(&node.key) {
                Ordering::Equal => return Some(&node.value),
                Ordering::Less => cur = node.left.as_ref(),
                Ordering::Greater => cur = node.right.as_ref(),
            }
        }
        None
    }

    pub fn contains(&self, key: &K) -> bool { self.get(key).is_some() }

    pub fn min(&self) -> Option<&K> {
        let mut cur = self.root.as_ref()?;
        while let Some(ref left) = cur.left { cur = left; }
        Some(&cur.key)
    }

    pub fn max(&self) -> Option<&K> {
        let mut cur = self.root.as_ref()?;
        while let Some(ref right) = cur.right { cur = right; }
        Some(&cur.key)
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut m = SplayMap::new();
        m.insert(3, "three");
        m.insert(1, "one");
        m.insert(2, "two");
        assert_eq!(m.get(&2), Some(&"two"));
        assert_eq!(m.get(&3), Some(&"three"));
        assert_eq!(m.get(&5), None);
    }

    #[test]
    fn overwrite() {
        let mut m = SplayMap::new();
        m.insert(1, 10);
        m.insert(1, 20);
        assert_eq!(m.get(&1), Some(&20));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn min_max() {
        let mut m = SplayMap::new();
        m.insert(5, 'a'); m.insert(2, 'b'); m.insert(8, 'c');
        assert_eq!(m.min(), Some(&2));
        assert_eq!(m.max(), Some(&8));
    }

    #[test]
    fn contains() {
        let mut m = SplayMap::new();
        m.insert(42, ());
        assert!(m.contains(&42));
        assert!(!m.contains(&99));
    }

    #[test]
    fn empty() {
        let m: SplayMap<i32, i32> = SplayMap::new();
        assert!(m.is_empty());
        assert_eq!(m.min(), None);
    }
}

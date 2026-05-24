use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PgError {
    InvalidCursor { cursor: u64 },
    EmptyStore,
}

impl std::fmt::Display for PgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PgError::InvalidCursor { cursor } => write!(f, "invalid cursor {cursor}"),
            PgError::EmptyStore => write!(f, "empty store"),
        }
    }
}

impl std::error::Error for PgError {}

struct Record {
    id: u64,
    data: Vec<u8>,
    sort_key: i64,
}

pub struct Paginator {
    records: BTreeMap<u64, Record>,
    page_size: usize,
    total_inserts: u64,
    total_queries: u64,
    total_pages_served: u64,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub items: Vec<(u64, Vec<u8>)>,
    pub next_cursor: Option<u64>,
    pub has_more: bool,
}

impl Paginator {
    pub fn new(page_size: usize) -> Self {
        Self { records: BTreeMap::new(), page_size, total_inserts: 0, total_queries: 0, total_pages_served: 0 }
    }

    pub fn insert(&mut self, id: u64, data: Vec<u8>, sort_key: i64) {
        self.records.insert(id, Record { id, data, sort_key });
        self.total_inserts += 1;
    }

    pub fn remove(&mut self, id: u64) -> Option<Vec<u8>> {
        self.records.remove(&id).map(|r| r.data)
    }

    pub fn query(&mut self, cursor: Option<u64>, ascending: bool) -> Result<Page, PgError> {
        if self.records.is_empty() { return Err(PgError::EmptyStore); }
        self.total_queries += 1;
        self.total_pages_served += 1;
        let mut sorted: Vec<&Record> = self.records.values().collect();
        sorted.sort_by_key(|r| r.sort_key);
        if !ascending { sorted.reverse(); }
        let start = match cursor {
            Some(c) => {
                let pos = sorted.iter().position(|r| r.id == c).ok_or(PgError::InvalidCursor { cursor: c })?;
                pos + 1
            }
            None => 0,
        };
        let end = (start + self.page_size).min(sorted.len());
        let items: Vec<(u64, Vec<u8>)> = sorted[start..end].iter().map(|r| (r.id, r.data.clone())).collect();
        let has_more = end < sorted.len();
        let next_cursor = if has_more { Some(sorted[end - 1].id) } else { None };
        Ok(Page { items, next_cursor, has_more })
    }

    pub fn count(&self) -> usize { self.records.len() }
    pub fn page_size(&self) -> usize { self.page_size }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
    pub fn total_pages_served(&self) -> u64 { self.total_pages_served }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_paginator() { let p = Paginator::new(10); assert_eq!(p.page_size(), 10); }

    #[test]
    fn insert_query_first_page() {
        let mut p = Paginator::new(2);
        p.insert(1, b"a".to_vec(), 10);
        p.insert(2, b"b".to_vec(), 20);
        p.insert(3, b"c".to_vec(), 30);
        let page = p.query(None, true).unwrap();
        assert_eq!(page.items.len(), 2);
        assert!(page.has_more);
        assert_eq!(page.items[0].0, 1);
    }

    #[test]
    fn next_page() {
        let mut p = Paginator::new(2);
        p.insert(1, b"a".to_vec(), 10);
        p.insert(2, b"b".to_vec(), 20);
        p.insert(3, b"c".to_vec(), 30);
        let page1 = p.query(None, true).unwrap();
        let page2 = p.query(page1.next_cursor, true).unwrap();
        assert_eq!(page2.items.len(), 1);
        assert!(!page2.has_more);
        assert_eq!(page2.items[0].0, 3);
    }

    #[test]
    fn descending() {
        let mut p = Paginator::new(2);
        p.insert(1, b"a".to_vec(), 10);
        p.insert(2, b"b".to_vec(), 20);
        let page = p.query(None, false).unwrap();
        assert_eq!(page.items[0].0, 2);
    }

    #[test]
    fn invalid_cursor() {
        let mut p = Paginator::new(10);
        p.insert(1, b"a".to_vec(), 1);
        let err = p.query(Some(99), true).unwrap_err();
        assert!(matches!(err, PgError::InvalidCursor { .. }));
    }

    #[test]
    fn empty_store() {
        let mut p = Paginator::new(10);
        let err = p.query(None, true).unwrap_err();
        assert!(matches!(err, PgError::EmptyStore));
    }

    #[test]
    fn remove() {
        let mut p = Paginator::new(10);
        p.insert(1, b"a".to_vec(), 1);
        let data = p.remove(1).unwrap();
        assert_eq!(data, b"a");
        assert!(p.query(None, true).is_err());
    }

    #[test]
    fn single_page() {
        let mut p = Paginator::new(10);
        p.insert(1, b"a".to_vec(), 1);
        let page = p.query(None, true).unwrap();
        assert_eq!(page.items.len(), 1);
        assert!(!page.has_more);
        assert_eq!(page.next_cursor, None);
    }

    #[test]
    fn full_pagination() {
        let mut p = Paginator::new(3);
        for i in 1..=7 { p.insert(i, vec![i as u8], i as i64 * 10); }
        let p1 = p.query(None, true).unwrap();
        assert_eq!(p1.items.len(), 3);
        let p2 = p.query(p1.next_cursor, true).unwrap();
        assert_eq!(p2.items.len(), 3);
        let p3 = p.query(p2.next_cursor, true).unwrap();
        assert_eq!(p3.items.len(), 1);
        assert!(!p3.has_more);
    }

    #[test]
    fn stats() {
        let mut p = Paginator::new(10);
        p.insert(1, b"x".to_vec(), 1);
        p.query(None, true).unwrap();
        assert_eq!(p.total_inserts(), 1);
        assert_eq!(p.total_pages_served(), 1);
    }

    #[test]
    fn error_display() { assert!(PgError::EmptyStore.to_string().contains("empty")); }
}

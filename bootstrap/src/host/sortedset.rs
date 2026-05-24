use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SsError {
    MemberExists { member: u64 },
    NotFound { member: u64 },
}

impl std::fmt::Display for SsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsError::MemberExists { member } => write!(f, "member {member} exists"),
            SsError::NotFound { member } => write!(f, "member {member} not found"),
        }
    }
}

impl std::error::Error for SsError {}

pub struct SortedSet {
    members: BTreeMap<u64, i64>,
    total_inserts: u64,
    total_removes: u64,
    total_rank_queries: u64,
    total_range_queries: u64,
}

impl SortedSet {
    pub fn new() -> Self { Self { members: BTreeMap::new(), total_inserts: 0, total_removes: 0, total_rank_queries: 0, total_range_queries: 0 } }

    pub fn insert(&mut self, member: u64, score: i64) -> Result<(), SsError> {
        if self.members.contains_key(&member) { return Err(SsError::MemberExists { member }); }
        self.members.insert(member, score);
        self.total_inserts += 1;
        Ok(())
    }

    pub fn upsert(&mut self, member: u64, score: i64) {
        self.members.insert(member, score);
        self.total_inserts += 1;
    }

    pub fn remove(&mut self, member: u64) -> Result<i64, SsError> {
        self.members.remove(&member).ok_or(SsError::NotFound { member }).map(|s| { self.total_removes += 1; s })
    }

    pub fn score(&self, member: u64) -> Option<i64> { self.members.get(&member).copied() }

    pub fn rank(&mut self, member: u64) -> Option<usize> {
        self.total_rank_queries += 1;
        if !self.members.contains_key(&member) { return None; }
        let target_score = self.members[&member];
        Some(self.members.iter().take_while(|(&m, &s)| s < target_score || (s == target_score && m < member)).count())
    }

    pub fn range_by_rank(&mut self, start: usize, end: usize) -> Vec<(u64, i64)> {
        self.total_range_queries += 1;
        self.members.iter().skip(start).take(end - start).map(|(&m, &s)| (m, s)).collect()
    }

    pub fn range_by_score(&mut self, min: i64, max: i64) -> Vec<(u64, i64)> {
        self.total_range_queries += 1;
        self.members.iter().filter(|(_, &s)| s >= min && s <= max).map(|(&m, &s)| (m, s)).collect()
    }

    pub fn top_k(&mut self, k: usize) -> Vec<(u64, i64)> {
        self.total_range_queries += 1;
        let n = self.members.len();
        self.members.iter().skip(n.saturating_sub(k)).map(|(&m, &s)| (m, s)).collect()
    }

    pub fn bottom_k(&mut self, k: usize) -> Vec<(u64, i64)> {
        self.total_range_queries += 1;
        self.members.iter().take(k).map(|(&m, &s)| (m, s)).collect()
    }

    pub fn len(&self) -> usize { self.members.len() }
    pub fn is_empty(&self) -> bool { self.members.is_empty() }
    pub fn contains(&self, member: u64) -> bool { self.members.contains_key(&member) }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_rank_queries(&self) -> u64 { self.total_rank_queries }
    pub fn total_range_queries(&self) -> u64 { self.total_range_queries }
}

impl Default for SortedSet {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ss() { assert!(SortedSet::new().is_empty()); }

    #[test]
    fn insert_score() {
        let mut ss = SortedSet::new();
        ss.insert(1, 100).unwrap();
        ss.insert(2, 50).unwrap();
        assert_eq!(ss.score(1), Some(100));
        assert_eq!(ss.score(2), Some(50));
    }

    #[test]
    fn duplicate_err() {
        let mut ss = SortedSet::new();
        ss.insert(1, 100).unwrap();
        assert!(ss.insert(1, 200).is_err());
    }

    #[test]
    fn upsert() {
        let mut ss = SortedSet::new();
        ss.upsert(1, 100); ss.upsert(1, 200);
        assert_eq!(ss.score(1), Some(200));
    }

    #[test]
    fn remove() {
        let mut ss = SortedSet::new();
        ss.insert(1, 100).unwrap();
        assert_eq!(ss.remove(1).unwrap(), 100);
        assert!(!ss.contains(1));
    }

    #[test]
    fn remove_not_found() { assert!(SortedSet::new().remove(1).is_err()); }

    #[test]
    fn rank() {
        let mut ss = SortedSet::new();
        ss.insert(1, 10).unwrap(); ss.insert(2, 20).unwrap(); ss.insert(3, 30).unwrap();
        assert_eq!(ss.rank(1), Some(0));
        assert_eq!(ss.rank(2), Some(1));
        assert_eq!(ss.rank(3), Some(2));
    }

    #[test]
    fn range_by_score() {
        let mut ss = SortedSet::new();
        for i in 1..=10 { ss.insert(i, i * 10).unwrap(); }
        let r = ss.range_by_score(30, 60);
        assert_eq!(r.len(), 4);
        assert_eq!(r[0], (3, 30));
    }

    #[test]
    fn top_k() {
        let mut ss = SortedSet::new();
        for i in 1..=10 { ss.insert(i, i * 10).unwrap(); }
        let top = ss.top_k(3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[2].1, 100);
    }

    #[test]
    fn bottom_k() {
        let mut ss = SortedSet::new();
        for i in 1..=5 { ss.insert(i, i * 10).unwrap(); }
        let bot = ss.bottom_k(2);
        assert_eq!(bot[0], (1, 10));
        assert_eq!(bot[1], (2, 20));
    }

    #[test]
    fn stats() {
        let mut ss = SortedSet::new();
        ss.insert(1, 10).unwrap(); ss.rank(1); ss.range_by_score(0, 100);
        assert_eq!(ss.total_inserts(), 1);
        assert_eq!(ss.total_rank_queries(), 1);
        assert_eq!(ss.total_range_queries(), 1);
    }

    #[test]
    fn error_display() { assert!(SsError::NotFound { member: 1 }.to_string().contains("not found")); }
}

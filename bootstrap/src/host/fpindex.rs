use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FpError {
    DocExists { id: u64 },
    DocNotFound { id: u64 },
}

impl std::fmt::Display for FpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FpError::DocExists { id } => write!(f, "doc {id} exists"),
            FpError::DocNotFound { id } => write!(f, "doc {id} not found"),
        }
    }
}

impl std::error::Error for FpError {}

const NUM_BANDS: usize = 8;
const ROWS_PER_BAND: usize = 4;
const SIG_LEN: usize = NUM_BANDS * ROWS_PER_BAND;

fn minhash_signature(shingles: &[u64]) -> [u64; SIG_LEN] {
    let mut sig = [u64::MAX; SIG_LEN];
    for (h, sig_val) in sig.iter_mut().enumerate() {
        let a = (h as u64).wrapping_mul(127 + h as u64);
        let b = (h as u64).wrapping_mul(311);
        for &s in shingles {
            let v = a.wrapping_mul(s).wrapping_add(b);
            if v < *sig_val { *sig_val = v; }
        }
    }
    sig
}

fn band_hash(sig: &[u64], band: usize) -> u64 {
    let start = band * ROWS_PER_BAND;
    let mut h: u64 = 0xcbf29ce484222325;
    for i in 0..ROWS_PER_BAND {
        h ^= sig[start + i];
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

struct Doc {
    id: u64,
    signature: [u64; SIG_LEN],
    shingles: Vec<u64>,
}

pub struct FingerprintIndex {
    docs: BTreeMap<u64, Doc>,
    bands: Vec<BTreeMap<u64, Vec<u64>>>,
    total_inserts: u64,
    total_queries: u64,
    total_candidates: u64,
}

impl FingerprintIndex {
    pub fn new() -> Self {
        let bands = (0..NUM_BANDS).map(|_| BTreeMap::new()).collect();
        Self { docs: BTreeMap::new(), bands, total_inserts: 0, total_queries: 0, total_candidates: 0 }
    }

    pub fn insert(&mut self, id: u64, shingles: Vec<u64>) -> Result<(), FpError> {
        if self.docs.contains_key(&id) { return Err(FpError::DocExists { id }); }
        let sig = minhash_signature(&shingles);
        for band in 0..NUM_BANDS {
            let bh = band_hash(&sig, band);
            self.bands[band].entry(bh).or_default().push(id);
        }
        self.docs.insert(id, Doc { id, signature: sig, shingles });
        self.total_inserts += 1;
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<(), FpError> {
        let doc = self.docs.remove(&id).ok_or(FpError::DocNotFound { id })?;
        for band in 0..NUM_BANDS {
            let bh = band_hash(&doc.signature, band);
            if let Some(ids) = self.bands[band].get_mut(&bh) {
                ids.retain(|&x| x != id);
            }
        }
        Ok(())
    }

    pub fn query(&mut self, shingles: &[u64], threshold: f64) -> Vec<(u64, f64)> {
        self.total_queries += 1;
        let sig = minhash_signature(shingles);
        let mut candidates: BTreeMap<u64, usize> = BTreeMap::new();
        for band in 0..NUM_BANDS {
            let bh = band_hash(&sig, band);
            if let Some(ids) = self.bands[band].get(&bh) {
                for &id in ids {
                    *candidates.entry(id).or_insert(0) += 1;
                }
            }
        }
        self.total_candidates += candidates.len() as u64;
        let mut results = Vec::new();
        for (id, matching_bands) in &candidates {
            let doc = match self.docs.get(id) {
                Some(d) => d,
                None => continue,
            };
            let sim = jaccard(&doc.shingles, shingles);
            if sim >= threshold { results.push((*id, sim)); }
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn similarity(&self, id_a: u64, id_b: u64) -> Option<f64> {
        let a = self.docs.get(&id_a)?;
        let b = self.docs.get(&id_b)?;
        Some(jaccard(&a.shingles, &b.shingles))
    }

    pub fn doc_count(&self) -> usize { self.docs.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
    pub fn total_candidates(&self) -> u64 { self.total_candidates }
}

fn jaccard(a: &[u64], b: &[u64]) -> f64 {
    use std::collections::BTreeSet;
    let sa: BTreeSet<_> = a.iter().copied().collect();
    let sb: BTreeSet<_> = b.iter().copied().collect();
    let inter = sa.intersection(&sb).count() as f64;
    let union = sa.union(&sb).count() as f64;
    if union == 0.0 { 0.0 } else { inter / union }
}

impl Default for FingerprintIndex {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_idx() { assert_eq!(FingerprintIndex::new().doc_count(), 0); }

    #[test]
    fn insert() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![1, 2, 3, 4, 5]).unwrap();
        assert_eq!(fi.doc_count(), 1);
    }

    #[test]
    fn duplicate() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![1, 2, 3]).unwrap();
        let err = fi.insert(1, vec![1, 2, 3]).unwrap_err();
        assert!(matches!(err, FpError::DocExists { .. }));
    }

    #[test]
    fn remove() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![1, 2, 3]).unwrap();
        fi.remove(1).unwrap();
        assert_eq!(fi.doc_count(), 0);
    }

    #[test]
    fn similarity_identical() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![1, 2, 3]).unwrap();
        fi.insert(2, vec![1, 2, 3]).unwrap();
        let sim = fi.similarity(1, 2).unwrap();
        assert!(sim > 0.99);
    }

    #[test]
    fn similarity_disjoint() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![1, 2, 3]).unwrap();
        fi.insert(2, vec![4, 5, 6]).unwrap();
        let sim = fi.similarity(1, 2).unwrap();
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn query_similar() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![1, 2, 3, 4, 5]).unwrap();
        fi.insert(2, vec![1, 2, 3, 4, 5]).unwrap();
        let results = fi.query(&[1, 2, 3, 4, 5], 0.8);
        assert!(results.len() >= 1);
    }

    #[test]
    fn query_no_match() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![100, 200, 300]).unwrap();
        let results = fi.query(&[1, 2, 3], 0.8);
        assert!(results.is_empty());
    }

    #[test]
    fn not_found() {
        let mut fi = FingerprintIndex::new();
        let err = fi.remove(99).unwrap_err();
        assert!(matches!(err, FpError::DocNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut fi = FingerprintIndex::new();
        fi.insert(1, vec![1, 2, 3]).unwrap();
        fi.query(&[1, 2], 0.5);
        assert_eq!(fi.total_inserts(), 1);
        assert_eq!(fi.total_queries(), 1);
    }
}

use std::collections::BTreeMap;

pub struct RefBitVec {
    segments: BTreeMap<usize, Segment>,
    seg_bits: usize,
    total_reads: u64,
    total_writes: u64,
}

#[derive(Clone)]
struct Segment {
    data: Vec<u64>,
    ref_count: u32,
}

impl RefBitVec {
    pub fn new(seg_bits: usize) -> Self { Self { segments: BTreeMap::new(), seg_bits: seg_bits.max(64), total_reads: 0, total_writes: 0 } }

    fn seg_index(&self, bit: usize) -> usize { bit / self.seg_bits }
    fn bit_offset(&self, bit: usize) -> usize { bit % self.seg_bits }

    pub fn get(&mut self, bit: usize) -> bool {
        self.total_reads += 1;
        let si = self.seg_index(bit);
        let bo = self.bit_offset(bit);
        let seg = self.segments.get(&si);
        let word = bo / 64;
        let mask = 1u64 << (bo % 64);
        seg.map_or(false, |s| s.data.get(word).map_or(false, |&w| w & mask != 0))
    }

    pub fn set(&mut self, bit: usize, val: bool) {
        self.total_writes += 1;
        let si = self.seg_index(bit);
        let bo = self.bit_offset(bit);
        let word = bo / 64;
        let mask = 1u64 << (bo % 64);
        let words = (self.seg_bits + 63) / 64;
        let seg = self.segments.entry(si).or_insert_with(|| Segment { data: vec![0u64; words], ref_count: 1 });
        if seg.ref_count > 1 {
            let new_seg = seg.clone();
            *seg = new_seg;
            seg.ref_count = 1;
        }
        if val { seg.data[word] |= mask; } else { seg.data[word] &= !mask; }
    }

    pub fn clone_segment(&mut self, from_seg: usize, to_seg: usize) -> bool {
        if let Some(seg) = self.segments.get_mut(&from_seg) {
            seg.ref_count += 1;
            let cloned = seg.clone();
            self.segments.insert(to_seg, cloned);
            true
        } else { false }
    }

    pub fn popcount(&mut self) -> usize {
        self.total_reads += 1;
        self.segments.values().flat_map(|s| s.data.iter()).map(|&w| w.count_ones() as usize).sum()
    }

    pub fn and(&mut self, other: &RefBitVec) {
        self.total_reads += 1;
        self.total_writes += 1;
        let mut result = BTreeMap::new();
        for (&si, seg) in &self.segments {
            if let Some(os) = other.segments.get(&si) {
                let words = seg.data.len().min(os.data.len());
                let mut data = vec![0u64; words];
                for i in 0..words { data[i] = seg.data[i] & os.data[i]; }
                result.insert(si, Segment { data, ref_count: 1 });
            }
        }
        self.segments = result;
    }

    pub fn segment_count(&self) -> usize { self.segments.len() }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_writes(&self) -> u64 { self.total_writes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let mut bv = RefBitVec::new(64);
        bv.set(10, true);
        assert!(bv.get(10));
        assert!(!bv.get(11));
    }

    #[test]
    fn clear() {
        let mut bv = RefBitVec::new(64);
        bv.set(5, true); bv.set(5, false);
        assert!(!bv.get(5));
    }

    #[test]
    fn multi_segment() {
        let mut bv = RefBitVec::new(64);
        bv.set(0, true); bv.set(100, true);
        assert_eq!(bv.segment_count(), 2);
    }

    #[test]
    fn popcount() {
        let mut bv = RefBitVec::new(64);
        bv.set(0, true); bv.set(1, true); bv.set(10, true);
        assert_eq!(bv.popcount(), 3);
    }

    #[test]
    fn clone_segment() {
        let mut bv = RefBitVec::new(64);
        bv.set(0, true);
        bv.clone_segment(0, 5);
        assert!(bv.get(5 * 64));
    }

    #[test]
    fn and() {
        let mut a = RefBitVec::new(64);
        let mut b = RefBitVec::new(64);
        a.set(0, true); a.set(1, true);
        b.set(1, true); b.set(2, true);
        a.and(&b);
        assert!(!a.get(0)); assert!(a.get(1)); assert!(!a.get(2));
    }

    #[test]
    fn empty_get() { assert!(!RefBitVec::new(64).get(0)); }

    #[test]
    fn stats() {
        let mut bv = RefBitVec::new(64);
        bv.set(0, true); bv.get(0);
        assert_eq!(bv.total_writes(), 1);
        assert_eq!(bv.total_reads(), 1);
    }
}

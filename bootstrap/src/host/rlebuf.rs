#[derive(Clone)]
struct Run { value: bool, count: usize }

pub struct RleBuf {
    runs: Vec<Run>,
    len: usize,
    total_encoded: u64,
    total_decoded: u64,
}

impl RleBuf {
    pub fn new() -> Self { Self { runs: Vec::new(), len: 0, total_encoded: 0, total_decoded: 0 } }

    pub fn encode(&mut self, data: &[bool]) {
        self.total_encoded += 1;
        self.runs.clear();
        self.len = data.len();
        if data.is_empty() { return; }
        let mut current = data[0];
        let mut count = 1usize;
        for &b in &data[1..] {
            if b == current { count += 1; }
            else { self.runs.push(Run { value: current, count }); current = b; count = 1; }
        }
        self.runs.push(Run { value: current, count });
    }

    pub fn decode(&mut self) -> Vec<bool> {
        self.total_decoded += 1;
        let mut out = Vec::with_capacity(self.len);
        for run in &self.runs { for _ in 0..run.count { out.push(run.value); } }
        out
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index >= self.len { return None; }
        let mut offset = 0usize;
        for run in &self.runs {
            if index < offset + run.count { return Some(run.value); }
            offset += run.count;
        }
        None
    }

    pub fn set(&mut self, index: usize, value: bool) -> bool {
        if index >= self.len { return false; }
        let mut offset = 0usize;
        for i in 0..self.runs.len() {
            let run = &self.runs[i];
            if index < offset + run.count {
                if run.value == value { return true; }
                let pos = index - offset;
                let old_count = run.count;
                let old_value = run.value;
                let mut new_runs = Vec::new();
                if pos > 0 { new_runs.push(Run { value: old_value, count: pos }); }
                new_runs.push(Run { value, count: 1 });
                if pos + 1 < old_count { new_runs.push(Run { value: old_value, count: old_count - pos - 1 }); }
                self.runs.splice(i..i + 1, new_runs);
                self.merge_adjacent();
                return true;
            }
            offset += run.count;
        }
        false
    }

    fn merge_adjacent(&mut self) {
        let mut i = 1;
        while i < self.runs.len() {
            if self.runs[i - 1].value == self.runs[i].value {
                self.runs[i - 1].count += self.runs[i].count;
                self.runs.remove(i);
            } else { i += 1; }
        }
    }

    pub fn popcount(&self) -> usize {
        self.runs.iter().filter(|r| r.value).map(|r| r.count).sum()
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.len == 0 { return 1.0; }
        (self.runs.len() as f64 * 2.0) / self.len as f64
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn run_count(&self) -> usize { self.runs.len() }
    pub fn total_encoded(&self) -> u64 { self.total_encoded }
    pub fn total_decoded(&self) -> u64 { self.total_decoded }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let mut rle = RleBuf::new();
        rle.encode(&[true, true, false, false, false, true]);
        let dec = rle.decode();
        assert_eq!(dec, vec![true, true, false, false, false, true]);
    }

    #[test]
    fn run_count() {
        let mut rle = RleBuf::new();
        rle.encode(&[true, true, false, false, true]);
        assert_eq!(rle.run_count(), 3);
    }

    #[test]
    fn get() {
        let mut rle = RleBuf::new();
        rle.encode(&[true, false, true]);
        assert_eq!(rle.get(0), Some(true));
        assert_eq!(rle.get(1), Some(false));
        assert_eq!(rle.get(5), None);
    }

    #[test]
    fn set() {
        let mut rle = RleBuf::new();
        rle.encode(&[true, true, true]);
        rle.set(1, false);
        assert_eq!(rle.get(0), Some(true));
        assert_eq!(rle.get(1), Some(false));
        assert_eq!(rle.get(2), Some(true));
    }

    #[test]
    fn set_merge() {
        let mut rle = RleBuf::new();
        rle.encode(&[true, false, true]);
        rle.set(1, true);
        assert_eq!(rle.run_count(), 1);
    }

    #[test]
    fn popcount() {
        let mut rle = RleBuf::new();
        rle.encode(&[true, true, false, true]);
        assert_eq!(rle.popcount(), 3);
    }

    #[test]
    fn compression_ratio() {
        let mut rle = RleBuf::new();
        rle.encode(&[true; 1000]);
        assert!(rle.compression_ratio() < 0.01);
    }

    #[test]
    fn empty() {
        let mut rle = RleBuf::new();
        rle.encode(&[]);
        assert!(rle.is_empty());
    }

    #[test]
    fn stats() {
        let mut rle = RleBuf::new();
        rle.encode(&[true]); rle.decode();
        assert_eq!(rle.total_encoded(), 1);
        assert_eq!(rle.total_decoded(), 1);
    }
}

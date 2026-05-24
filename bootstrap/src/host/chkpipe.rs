#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgo {
    Sum8,
    Xor8,
    Crc16,
    Fletcher16,
}

impl std::fmt::Display for ChecksumAlgo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChecksumAlgo::Sum8 => write!(f, "sum8"),
            ChecksumAlgo::Xor8 => write!(f, "xor8"),
            ChecksumAlgo::Crc16 => write!(f, "crc16"),
            ChecksumAlgo::Fletcher16 => write!(f, "fletcher16"),
        }
    }
}

pub fn compute_sum8(data: &[u8]) -> u16 {
    data.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16))
}

pub fn compute_xor8(data: &[u8]) -> u16 {
    data.iter().fold(0u16, |acc, &b| acc ^ (b as u16))
}

pub fn compute_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= b as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0x8408;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

pub fn compute_fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u8 = 0;
    let mut sum2: u8 = 0;
    for &b in data {
        sum1 = sum1.wrapping_add(b);
        sum2 = sum2.wrapping_add(sum1);
    }
    ((sum2 as u16) << 8) | (sum1 as u16)
}

pub fn compute(algo: ChecksumAlgo, data: &[u8]) -> u16 {
    match algo {
        ChecksumAlgo::Sum8 => compute_sum8(data),
        ChecksumAlgo::Xor8 => compute_xor8(data),
        ChecksumAlgo::Crc16 => compute_crc16(data),
        ChecksumAlgo::Fletcher16 => compute_fletcher16(data),
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub algo: ChecksumAlgo,
    pub result: u16,
}

#[derive(Debug, Clone)]
pub struct ChecksumPipeline {
    stages: Vec<ChecksumAlgo>,
    last_results: Vec<PipelineStage>,
    total_computed: u64,
}

impl ChecksumPipeline {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            last_results: Vec::new(),
            total_computed: 0,
        }
    }

    pub fn add_stage(&mut self, algo: ChecksumAlgo) {
        self.stages.push(algo);
    }

    pub fn clear_stages(&mut self) {
        self.stages.clear();
        self.last_results.clear();
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    pub fn compute_all(&mut self, data: &[u8]) -> Vec<PipelineStage> {
        let mut results = Vec::with_capacity(self.stages.len());
        for &algo in &self.stages {
            let result = compute(algo, data);
            results.push(PipelineStage { algo, result });
            self.total_computed += 1;
        }
        self.last_results = results.clone();
        results
    }

    pub fn compute_chain(&mut self, data: &[u8]) -> u16 {
        let mut current = data.to_vec();
        let mut final_result: u16 = 0;
        for &algo in &self.stages {
            let result = compute(algo, &current);
            self.total_computed += 1;
            final_result = result;
            current = result.to_le_bytes().to_vec();
        }
        final_result
    }

    pub fn last_results(&self) -> &[PipelineStage] {
        &self.last_results
    }

    pub fn total_computed(&self) -> u64 {
        self.total_computed
    }

    pub fn verify(&mut self, data: &[u8], expected: &[(ChecksumAlgo, u16)]) -> bool {
        let results = self.compute_all(data);
        if results.len() != expected.len() {
            return false;
        }
        for (r, (algo, exp)) in results.iter().zip(expected.iter()) {
            if r.algo != *algo || r.result != *exp {
                return false;
            }
        }
        true
    }
}

impl Default for ChecksumPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum8_basic() {
        assert_eq!(compute_sum8(&[1, 2, 3]), 6);
    }

    #[test]
    fn sum8_overflow() {
        assert_eq!(compute_sum8(&[0xFF, 0x02]), 0x0101);
    }

    #[test]
    fn xor8_basic() {
        assert_eq!(compute_xor8(&[0xAA, 0x55]), 0xFF);
        assert_eq!(compute_xor8(&[0xAA, 0xAA]), 0);
    }

    #[test]
    fn fletcher16_basic() {
        let r = compute_fletcher16(&[1, 2]);
        assert_ne!(r, 0);
    }

    #[test]
    fn crc16_deterministic() {
        let a = compute_crc16(b"hello");
        let b = compute_crc16(b"hello");
        assert_eq!(a, b);
        let c = compute_crc16(b"world");
        assert_ne!(a, c);
    }

    #[test]
    fn compute_dispatch() {
        let data = b"test";
        let s = compute(ChecksumAlgo::Sum8, data);
        assert_eq!(s, compute_sum8(data));
    }

    #[test]
    fn algo_display() {
        assert_eq!(ChecksumAlgo::Sum8.to_string(), "sum8");
        assert_eq!(ChecksumAlgo::Fletcher16.to_string(), "fletcher16");
    }

    #[test]
    fn pipeline_add_and_count() {
        let mut p = ChecksumPipeline::new();
        p.add_stage(ChecksumAlgo::Sum8);
        p.add_stage(ChecksumAlgo::Crc16);
        assert_eq!(p.stage_count(), 2);
    }

    #[test]
    fn pipeline_compute_all() {
        let mut p = ChecksumPipeline::new();
        p.add_stage(ChecksumAlgo::Sum8);
        p.add_stage(ChecksumAlgo::Xor8);
        let results = p.compute_all(b"abc");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].algo, ChecksumAlgo::Sum8);
        assert_eq!(results[1].algo, ChecksumAlgo::Xor8);
        assert_eq!(p.total_computed(), 2);
    }

    #[test]
    fn pipeline_last_results() {
        let mut p = ChecksumPipeline::new();
        p.add_stage(ChecksumAlgo::Sum8);
        p.compute_all(b"test");
        assert_eq!(p.last_results().len(), 1);
    }

    #[test]
    fn pipeline_chain() {
        let mut p = ChecksumPipeline::new();
        p.add_stage(ChecksumAlgo::Sum8);
        p.add_stage(ChecksumAlgo::Crc16);
        let result = p.compute_chain(b"hello");
        assert_ne!(result, 0);
        assert_eq!(p.total_computed(), 2);
    }

    #[test]
    fn pipeline_verify_ok() {
        let mut p = ChecksumPipeline::new();
        p.add_stage(ChecksumAlgo::Sum8);
        let sum = compute_sum8(b"abc");
        assert!(p.verify(b"abc", &[(ChecksumAlgo::Sum8, sum)]));
    }

    #[test]
    fn pipeline_verify_fail() {
        let mut p = ChecksumPipeline::new();
        p.add_stage(ChecksumAlgo::Sum8);
        assert!(!p.verify(b"abc", &[(ChecksumAlgo::Sum8, 9999)]));
    }

    #[test]
    fn clear_stages() {
        let mut p = ChecksumPipeline::new();
        p.add_stage(ChecksumAlgo::Sum8);
        p.clear_stages();
        assert_eq!(p.stage_count(), 0);
        assert!(p.last_results().is_empty());
    }
}

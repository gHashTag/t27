#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ternary {
    MinusOne,
    Zero,
    PlusOne,
}

impl Ternary {
    pub fn to_i8(&self) -> i8 {
        match self {
            Ternary::MinusOne => -1,
            Ternary::Zero => 0,
            Ternary::PlusOne => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::MinusOne),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::PlusOne),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RleRun {
    pub value: Ternary,
    pub count: u32,
}

impl RleRun {
    pub fn new(value: Ternary, count: u32) -> Self {
        Self { value, count }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressError {
    EmptyInput,
    RunTooLong { max: u32 },
}

impl std::fmt::Display for CompressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressError::EmptyInput => write!(f, "empty input"),
            CompressError::RunTooLong { max } => write!(f, "run exceeds {max}"),
        }
    }
}

impl std::error::Error for CompressError {}

#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_count: usize,
    pub compressed_runs: usize,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct WeightCompressor {
    max_run: u32,
    total_compressed: u64,
    total_weights: u64,
}

impl WeightCompressor {
    pub fn new(max_run: u32) -> Self {
        Self {
            max_run,
            total_compressed: 0,
            total_weights: 0,
        }
    }

    pub fn compress(&mut self, weights: &[Ternary]) -> Result<Vec<RleRun>, CompressError> {
        if weights.is_empty() {
            return Err(CompressError::EmptyInput);
        }
        let mut runs = Vec::new();
        let mut current = weights[0];
        let mut count = 1u32;
        for &w in &weights[1..] {
            if w == current && count < self.max_run {
                count += 1;
            } else {
                runs.push(RleRun::new(current, count));
                current = w;
                count = 1;
            }
        }
        runs.push(RleRun::new(current, count));
        self.total_compressed += 1;
        self.total_weights += weights.len() as u64;
        Ok(runs)
    }

    pub fn decompress(runs: &[RleRun]) -> Vec<Ternary> {
        let mut weights = Vec::new();
        for run in runs {
            for _ in 0..run.count {
                weights.push(run.value);
            }
        }
        weights
    }

    pub fn stats(&self, runs: &[RleRun], original: usize) -> CompressionStats {
        let total_elements: u32 = runs.iter().map(|r| r.count).sum();
        let ratio = if original == 0 {
            0.0
        } else {
            (runs.len() as f64) / (original as f64)
        };
        CompressionStats {
            original_count: original,
            compressed_runs: runs.len(),
            compression_ratio: ratio,
        }
    }

    pub fn verify(runs: &[RleRun], original: &[Ternary]) -> bool {
        let decompressed = Self::decompress(runs);
        decompressed == original
    }

    pub fn encode_to_bytes(runs: &[RleRun]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(runs.len() * 5);
        for run in runs {
            let v = match run.value {
                Ternary::MinusOne => 0u8,
                Ternary::Zero => 1,
                Ternary::PlusOne => 2,
            };
            buf.push(v);
            buf.extend_from_slice(&run.count.to_le_bytes());
        }
        buf
    }

    pub fn decode_from_bytes(data: &[u8]) -> Result<Vec<RleRun>, CompressError> {
        if data.is_empty() {
            return Err(CompressError::EmptyInput);
        }
        if data.len() % 5 != 0 {
            return Err(CompressError::EmptyInput);
        }
        let mut runs = Vec::new();
        let mut i = 0;
        while i + 5 <= data.len() {
            let value = match data[i] {
                0 => Ternary::MinusOne,
                1 => Ternary::Zero,
                2 => Ternary::PlusOne,
                _ => Ternary::Zero,
            };
            let count = u32::from_le_bytes([data[i + 1], data[i + 2], data[i + 3], data[i + 4]]);
            runs.push(RleRun::new(value, count));
            i += 5;
        }
        Ok(runs)
    }

    pub fn total_compressed(&self) -> u64 {
        self.total_compressed
    }

    pub fn total_weights(&self) -> u64 {
        self.total_weights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ternary_roundtrip() {
        assert_eq!(Ternary::from_i8(-1), Some(Ternary::MinusOne));
        assert_eq!(Ternary::from_i8(0), Some(Ternary::Zero));
        assert_eq!(Ternary::from_i8(1), Some(Ternary::PlusOne));
        assert_eq!(Ternary::from_i8(2), None);
    }

    #[test]
    fn ternary_to_i8() {
        assert_eq!(Ternary::MinusOne.to_i8(), -1);
        assert_eq!(Ternary::PlusOne.to_i8(), 1);
    }

    #[test]
    fn compress_uniform() {
        let mut wc = WeightCompressor::new(1024);
        let weights = vec![Ternary::PlusOne; 10];
        let runs = wc.compress(&weights).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].count, 10);
    }

    #[test]
    fn compress_alternating() {
        let mut wc = WeightCompressor::new(1024);
        let weights = vec![Ternary::PlusOne, Ternary::MinusOne, Ternary::PlusOne, Ternary::MinusOne];
        let runs = wc.compress(&weights).unwrap();
        assert_eq!(runs.len(), 4);
        assert_eq!(runs[0].count, 1);
    }

    #[test]
    fn compress_empty() {
        let mut wc = WeightCompressor::new(1024);
        let err = wc.compress(&[]).unwrap_err();
        assert!(matches!(err, CompressError::EmptyInput));
    }

    #[test]
    fn compress_max_run_split() {
        let mut wc = WeightCompressor::new(3);
        let weights = vec![Ternary::Zero; 7];
        let runs = wc.compress(&weights).unwrap();
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].count, 3);
        assert_eq!(runs[1].count, 3);
        assert_eq!(runs[2].count, 1);
    }

    #[test]
    fn decompress() {
        let runs = vec![RleRun::new(Ternary::PlusOne, 3), RleRun::new(Ternary::Zero, 2)];
        let weights = WeightCompressor::decompress(&runs);
        assert_eq!(weights.len(), 5);
        assert_eq!(weights[0], Ternary::PlusOne);
        assert_eq!(weights[3], Ternary::Zero);
    }

    #[test]
    fn verify_ok() {
        let mut wc = WeightCompressor::new(1024);
        let weights = vec![Ternary::PlusOne, Ternary::Zero, Ternary::MinusOne, Ternary::Zero];
        let runs = wc.compress(&weights).unwrap();
        assert!(WeightCompressor::verify(&runs, &weights));
    }

    #[test]
    fn encode_decode_bytes() {
        let runs = vec![RleRun::new(Ternary::PlusOne, 100), RleRun::new(Ternary::MinusOne, 50)];
        let bytes = WeightCompressor::encode_to_bytes(&runs);
        let decoded = WeightCompressor::decode_from_bytes(&bytes).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].value, Ternary::PlusOne);
        assert_eq!(decoded[0].count, 100);
        assert_eq!(decoded[1].value, Ternary::MinusOne);
    }

    #[test]
    fn stats() {
        let mut wc = WeightCompressor::new(1024);
        let weights = vec![Ternary::Zero; 100];
        let runs = wc.compress(&weights).unwrap();
        let s = wc.stats(&runs, 100);
        assert_eq!(s.original_count, 100);
        assert_eq!(s.compressed_runs, 1);
    }

    #[test]
    fn totals() {
        let mut wc = WeightCompressor::new(1024);
        wc.compress(&[Ternary::Zero; 10]).unwrap();
        wc.compress(&[Ternary::PlusOne; 5]).unwrap();
        assert_eq!(wc.total_compressed(), 2);
        assert_eq!(wc.total_weights(), 15);
    }

    #[test]
    fn error_display() {
        assert!(CompressError::EmptyInput.to_string().contains("empty"));
    }
}

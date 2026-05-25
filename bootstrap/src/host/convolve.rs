#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PadMode {
    Zero,
    Clamp,
    Reflect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CvError {
    EmptyInput,
    EmptyKernel,
}

impl std::fmt::Display for CvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CvError::EmptyInput => write!(f, "empty input"),
            CvError::EmptyKernel => write!(f, "empty kernel"),
        }
    }
}

impl std::error::Error for CvError {}

pub struct Convolve {
    total_ops: u64,
    total_flops: u64,
}

impl Convolve {
    pub fn new() -> Self { Self { total_ops: 0, total_flops: 0 } }

    fn pad_value(&self, data: &[i64], idx: isize, mode: PadMode) -> i64 {
        if idx >= 0 && (idx as usize) < data.len() { return data[idx as usize]; }
        match mode {
            PadMode::Zero => 0,
            PadMode::Clamp => {
                if idx < 0 { data[0] } else { *data.last().unwrap() }
            }
            PadMode::Reflect => {
                let len = data.len() as isize;
                let mut i = idx;
                while i < 0 || i >= len {
                    if i < 0 { i = -i - 1; }
                    if i >= len { i = 2 * len - i - 1; }
                }
                data[i as usize]
            }
        }
    }

    pub fn convolve1d(&mut self, data: &[i64], kernel: &[i64], stride: usize, mode: PadMode) -> Result<Vec<i64>, CvError> {
        if data.is_empty() { return Err(CvError::EmptyInput); }
        if kernel.is_empty() { return Err(CvError::EmptyKernel); }
        self.total_ops += 1;
        let k_half = (kernel.len() / 2) as isize;
        let out_len = (data.len() + stride - 1) / stride;
        let mut result = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let center = (i * stride) as isize;
            let mut sum: i64 = 0;
            for k in 0..kernel.len() {
                let idx = center + k as isize - k_half;
                sum += self.pad_value(data, idx, mode) * kernel[k];
                self.total_flops += 2;
            }
            result.push(sum);
        }
        Ok(result)
    }

    pub fn correlate1d(&mut self, data: &[i64], kernel: &[i64], stride: usize, mode: PadMode) -> Result<Vec<i64>, CvError> {
        if data.is_empty() { return Err(CvError::EmptyInput); }
        if kernel.is_empty() { return Err(CvError::EmptyKernel); }
        self.total_ops += 1;
        let out_len = (data.len() + stride - 1) / stride;
        let mut result = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let center = (i * stride) as isize;
            let mut sum: i64 = 0;
            for k in 0..kernel.len() {
                let idx = center + k as isize;
                sum += self.pad_value(data, idx, mode) * kernel[k];
                self.total_flops += 2;
            }
            result.push(sum);
        }
        Ok(result)
    }

    pub fn moving_avg(&mut self, data: &[i64], window: usize) -> Vec<i64> {
        self.total_ops += 1;
        if window == 0 || data.is_empty() { return Vec::new(); }
        let mut result = Vec::with_capacity(data.len());
        let mut sum: i64 = 0;
        for i in 0..data.len() {
            sum += data[i];
            if i >= window { sum -= data[i - window]; }
            if i >= window - 1 { result.push(sum / window as i64); }
        }
        result
    }

    pub fn total_ops(&self) -> u64 { self.total_ops }
    pub fn total_flops(&self) -> u64 { self.total_flops }
}

impl Default for Convolve {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convolve1d_identity() {
        let mut cv = Convolve::new();
        let data = vec![1, 2, 3, 4, 5];
        let kernel = vec![1];
        let result = cv.convolve1d(&data, &kernel, 1, PadMode::Zero).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn convolve1d_box() {
        let mut cv = Convolve::new();
        let data = vec![1, 2, 3, 4, 5];
        let kernel = vec![1, 1, 1];
        let result = cv.convolve1d(&data, &kernel, 1, PadMode::Zero).unwrap();
        assert_eq!(result[0], 3);
        assert_eq!(result[1], 6);
    }

    #[test]
    fn pad_clamp() {
        let mut cv = Convolve::new();
        let data = vec![10, 20, 30];
        let kernel = vec![0, 1, 0];
        let result = cv.convolve1d(&data, &kernel, 1, PadMode::Clamp).unwrap();
        assert_eq!(result[1], 20);
    }

    #[test]
    fn stride() {
        let mut cv = Convolve::new();
        let data = vec![1, 2, 3, 4, 5, 6];
        let kernel = vec![1];
        let result = cv.convolve1d(&data, &kernel, 2, PadMode::Zero).unwrap();
        assert_eq!(result, vec![1, 3, 5]);
    }

    #[test]
    fn correlate() {
        let mut cv = Convolve::new();
        let data = vec![1, 2, 3];
        let kernel = vec![1, 2];
        let result = cv.correlate1d(&data, &kernel, 1, PadMode::Zero).unwrap();
        assert_eq!(result[0], 1 * 1 + 2 * 2);
    }

    #[test]
    fn moving_avg() {
        let mut cv = Convolve::new();
        let data = vec![2, 4, 6, 8, 10];
        let result = cv.moving_avg(&data, 3);
        assert_eq!(result, vec![4, 6, 8]);
    }

    #[test]
    fn empty_input() { assert!(Convolve::new().convolve1d(&[], &[1], 1, PadMode::Zero).is_err()); }
    #[test]
    fn empty_kernel() { assert!(Convolve::new().convolve1d(&[1], &[], 1, PadMode::Zero).is_err()); }

    #[test]
    fn stats() {
        let mut cv = Convolve::new();
        cv.convolve1d(&[1, 2], &[1], 1, PadMode::Zero).unwrap();
        assert_eq!(cv.total_ops(), 1);
        assert!(cv.total_flops() > 0);
    }
}

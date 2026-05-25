pub struct Corput;

impl Corput {
    pub fn vdc(index: usize, base: usize) -> f64 {
        let mut n = index;
        let mut result = 0.0f64;
        let mut f = 1.0 / base as f64;
        while n > 0 {
            result += (n % base) as f64 * f;
            n /= base;
            f /= base as f64;
        }
        result
    }

    pub fn halton(index: usize, base_x: usize, base_y: usize) -> (f64, f64) {
        (Self::vdc(index, base_x), Self::vdc(index, base_y))
    }

    pub fn halton_sequence(n: usize) -> Vec<(f64, f64)> {
        (0..n).map(|i| Self::halton(i, 2, 3)).collect()
    }

    pub fn discrepancy(samples: &[f64]) -> f64 {
        if samples.is_empty() { return 1.0; }
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = sorted.len() as f64;
        let mut max_disc = 0.0f64;
        for (i, &s) in sorted.iter().enumerate() {
            let expected = (i + 1) as f64 / n;
            let disc = (expected - s).abs();
            max_disc = max_disc.max(disc);
        }
        max_disc
    }

    pub fn scrambled_vdc(index: usize, base: usize, seed: u64) -> f64 {
        let mut state = seed;
        let perm = Self::faure_permutation(base, &mut state);
        let mut n = index;
        let mut result = 0.0f64;
        let mut f = 1.0 / base as f64;
        while n > 0 {
            let digit = n % base;
            result += perm[digit] as f64 * f;
            n /= base;
            f /= base as f64;
        }
        result
    }

    fn faure_permutation(base: usize, state: &mut u64) -> Vec<usize> {
        let mut perm: Vec<usize> = (0..base).collect();
        for i in (1..base).rev() {
            *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (*state >> 33) as usize % (i + 1);
            perm.swap(i, j);
        }
        perm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vdc_base2() {
        assert!((Corput::vdc(0, 2) - 0.0).abs() < 1e-9);
        assert!((Corput::vdc(1, 2) - 0.5).abs() < 1e-9);
        assert!((Corput::vdc(2, 2) - 0.25).abs() < 1e-9);
        assert!((Corput::vdc(3, 2) - 0.75).abs() < 1e-9);
    }

    #[test]
    fn vdc_range() {
        for i in 0..100 {
            let v = Corput::vdc(i, 2);
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn halton() {
        let (x, y) = Corput::halton(5, 2, 3);
        assert!(x >= 0.0 && x < 1.0);
        assert!(y >= 0.0 && y < 1.0);
    }

    #[test]
    fn halton_sequence_len() {
        let seq = Corput::halton_sequence(50);
        assert_eq!(seq.len(), 50);
    }

    #[test]
    fn discrepancy_low() {
        let samples: Vec<f64> = (0..100).map(|i| Corput::vdc(i, 2)).collect();
        let disc = Corput::discrepancy(&samples);
        assert!(disc < 0.1);
    }
}

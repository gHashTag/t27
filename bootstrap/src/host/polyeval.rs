pub struct PolyEval;

impl PolyEval {
    pub fn evaluate(coeffs: &[f64], x: f64) -> f64 {
        let mut result = 0.0f64;
        for &c in coeffs.iter().rev() { result = result * x + c; }
        result
    }

    pub fn evaluate_batch(coeffs: &[f64], xs: &[f64]) -> Vec<f64> {
        xs.iter().map(|&x| Self::evaluate(coeffs, x)).collect()
    }

    pub fn lagrange(xs: &[f64], ys: &[f64]) -> Vec<f64> {
        let n = xs.len();
        if n == 0 { return Vec::new(); }
        let mut coeffs = vec![0.0f64; n];
        for i in 0..n {
            let mut basis = vec![0.0f64; n + 1];
            basis[0] = 1.0;
            let mut degree = 0usize;
            for j in 0..n {
                if i == j { continue; }
                let denom = xs[i] - xs[j];
                for k in (0..=degree).rev() {
                    basis[k + 1] += basis[k] / denom;
                    basis[k] = -basis[k] * xs[j] / denom;
                }
                degree += 1;
            }
            for k in 0..n { coeffs[k] += ys[i] * basis[k]; }
        }
        while coeffs.len() > 1 && coeffs.last() == Some(&0.0) { coeffs.pop(); }
        coeffs
    }

    pub fn derivative(coeffs: &[f64]) -> Vec<f64> {
        if coeffs.is_empty() { return Vec::new(); }
        coeffs.iter().enumerate().skip(1).map(|(i, &c)| c * i as f64).collect()
    }

    pub fn integrate(coeffs: &[f64]) -> Vec<f64> {
        let mut result = vec![0.0f64; coeffs.len() + 1];
        for (i, &c) in coeffs.iter().enumerate() { result[i + 1] = c / (i as f64 + 1.0); }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_linear() { assert!((PolyEval::evaluate(&[1.0, 2.0], 3.0) - 7.0).abs() < 1e-9); }

    #[test]
    fn evaluate_constant() { assert!((PolyEval::evaluate(&[5.0], 100.0) - 5.0).abs() < 1e-9); }

    #[test]
    fn lagrange_identity() {
        let xs = [0.0, 1.0, 2.0];
        let ys = [0.0, 1.0, 2.0];
        let p = PolyEval::lagrange(&xs, &ys);
        for &x in &xs { assert!((PolyEval::evaluate(&p, x) - x).abs() < 1e-6); }
    }

    #[test]
    fn derivative() {
        let d = PolyEval::derivative(&[1.0, 2.0, 3.0]);
        assert_eq!(d.len(), 2);
        assert!((d[0] - 2.0).abs() < 1e-9);
        assert!((d[1] - 6.0).abs() < 1e-9);
    }

    #[test]
    fn integrate() {
        let r = PolyEval::integrate(&[2.0, 3.0]);
        assert!((r[0] - 0.0).abs() < 1e-9);
        assert!((r[1] - 2.0).abs() < 1e-9);
        assert!((r[2] - 1.5).abs() < 1e-9);
    }

    #[test]
    fn batch() {
        let r = PolyEval::evaluate_batch(&[1.0, 0.0, 1.0], &[0.0, 1.0, 2.0]);
        assert!((r[0] - 1.0).abs() < 1e-9);
        assert!((r[1] - 2.0).abs() < 1e-9);
        assert!((r[2] - 5.0).abs() < 1e-9);
    }
}

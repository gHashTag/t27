pub struct MatrixChain;

impl MatrixChain {
    pub fn optimal_cost(dims: &[usize]) -> (usize, Vec<Vec<usize>>) {
        let n = dims.len();
        if n < 2 { return (0, Vec::new()); }
        let n = n - 1;
        let mut dp = vec![vec![0usize; n]; n];
        let mut split = vec![vec![0usize; n]; n];
        for len in 2..=n {
            for i in 0..=n - len {
                let j = i + len - 1;
                dp[i][j] = usize::MAX;
                for k in i..j {
                    let cost = dp[i][k] + dp[k + 1][j] + dims[i] * dims[k + 1] * dims[j + 1];
                    if cost < dp[i][j] {
                        dp[i][j] = cost;
                        split[i][j] = k;
                    }
                }
            }
        }
        (dp[0][n - 1], split)
    }

    pub fn parenthesize(split: &[Vec<usize>], i: usize, j: usize) -> String {
        if i == j { return format!("A{}", i); }
        let k = split[i][j];
        format!("({} x {})", Self::parenthesize(split, i, k), Self::parenthesize(split, k + 1, j))
    }

    pub fn scalar_count(dims: &[usize]) -> usize {
        let mut total = 0usize;
        for i in 0..dims.len() - 1 { total += dims[i] * dims[i + 1]; }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_matrices() {
        let dims = vec![10, 30, 5, 60];
        let (cost, split) = MatrixChain::optimal_cost(&dims);
        assert_eq!(cost, 4500);
        let s = MatrixChain::parenthesize(&split, 0, 2);
        assert!(s.contains("A") && s.contains("x"));
    }

    #[test]
    fn two_matrices() {
        let dims = vec![10, 20];
        let (cost, _) = MatrixChain::optimal_cost(&dims);
        assert_eq!(cost, 0);
    }

    #[test]
    fn single_matrix() {
        let (cost, _) = MatrixChain::optimal_cost(&[5]);
        assert_eq!(cost, 0);
    }

    #[test]
    fn classic() {
        let dims = vec![30, 35, 15, 5, 10, 20, 25];
        let (cost, _) = MatrixChain::optimal_cost(&dims);
        assert_eq!(cost, 15125);
    }

    #[test]
    fn scalar_count() {
        assert_eq!(MatrixChain::scalar_count(&[10, 20, 30]), 800);
    }
}

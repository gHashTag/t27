pub struct StrasMat {
    total_mults: u64,
    total_adds: u64,
}

impl StrasMat {
    pub fn new() -> Self { Self { total_mults: 0, total_adds: 0 } }

    pub fn multiply(&mut self, a: &[Vec<i64>], b: &[Vec<i64>]) -> Vec<Vec<i64>> {
        let m = a.len();
        let n = b[0].len();
        let p = b.len();
        let block = 16;
        let mut c = vec![vec![0i64; n]; m];
        for ii in (0..m).step_by(block) {
            for jj in (0..n).step_by(block) {
                for kk in (0..p).step_by(block) {
                    let ie = (ii + block).min(m);
                    let je = (jj + block).min(n);
                    let ke = (kk + block).min(p);
                    for i in ii..ie {
                        for k in kk..ke {
                            let aik = a[i][k];
                            if aik == 0 { continue; }
                            self.total_mults += 1;
                            for j in jj..je {
                                c[i][j] += aik * b[k][j];
                                self.total_adds += 1;
                            }
                        }
                    }
                }
            }
        }
        c
    }

    pub fn add(a: &[Vec<i64>], b: &[Vec<i64>]) -> Vec<Vec<i64>> {
        let m = a.len();
        let n = a[0].len();
        let mut c = vec![vec![0i64; n]; m];
        for i in 0..m { for j in 0..n { c[i][j] = a[i][j] + b[i][j]; } }
        c
    }

    pub fn sub(a: &[Vec<i64>], b: &[Vec<i64>]) -> Vec<Vec<i64>> {
        let m = a.len();
        let n = a[0].len();
        let mut c = vec![vec![0i64; n]; m];
        for i in 0..m { for j in 0..n { c[i][j] = a[i][j] - b[i][j]; } }
        c
    }

    pub fn transpose(m: &[Vec<i64>]) -> Vec<Vec<i64>> {
        let rows = m.len();
        let cols = m[0].len();
        let mut t = vec![vec![0i64; rows]; cols];
        for i in 0..rows { for j in 0..cols { t[j][i] = m[i][j]; } }
        t
    }

    pub fn identity(n: usize) -> Vec<Vec<i64>> {
        let mut m = vec![vec![0i64; n]; n];
        for i in 0..n { m[i][i] = 1; }
        m
    }

    pub fn total_mults(&self) -> u64 { self.total_mults }
    pub fn total_adds(&self) -> u64 { self.total_adds }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mat(rows: usize, cols: usize, fill: i64) -> Vec<Vec<i64>> { vec![vec![fill; cols]; rows] }

    #[test]
    fn identity() {
        let mut sm = StrasMat::new();
        let i = StrasMat::identity(3);
        let a = mat(3, 3, 5);
        let c = sm.multiply(&a, &i);
        assert_eq!(c, a);
    }

    #[test]
    fn square() {
        let mut sm = StrasMat::new();
        let a = vec![vec![1, 2], vec![3, 4]];
        let b = vec![vec![5, 6], vec![7, 8]];
        let c = sm.multiply(&a, &b);
        assert_eq!(c[0][0], 19);
        assert_eq!(c[1][1], 50);
    }

    #[test]
    fn add_sub() {
        let a = vec![vec![1, 2], vec![3, 4]];
        let b = vec![vec![5, 6], vec![7, 8]];
        let s = StrasMat::add(&a, &b);
        let d = StrasMat::sub(&s, &b);
        assert_eq!(d, a);
    }

    #[test]
    fn transpose() {
        let a = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let t = StrasMat::transpose(&a);
        assert_eq!(t[0], vec![1, 4]);
        assert_eq!(t[2], vec![3, 6]);
    }

    #[test]
    fn rect() {
        let mut sm = StrasMat::new();
        let a = vec![vec![1, 2, 3]];
        let b = vec![vec![4], vec![5], vec![6]];
        let c = sm.multiply(&a, &b);
        assert_eq!(c, vec![vec![32]]);
    }

    #[test]
    fn stats() {
        let mut sm = StrasMat::new();
        sm.multiply(&StrasMat::identity(2), &StrasMat::identity(2));
        assert!(sm.total_mults() > 0);
    }
}

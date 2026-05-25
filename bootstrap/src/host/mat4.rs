pub struct Mat4;

impl Mat4 {
    pub fn identity() -> [[f64; 4]; 4] {
        [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]]
    }

    pub fn mul(a: &[[f64; 4]; 4], b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
        let mut c = [[0.0f64; 4]; 4];
        for i in 0..4 { for j in 0..4 { for k in 0..4 { c[i][j] += a[i][k] * b[k][j]; } } }
        c
    }

    fn det3(m: &[[f64; 3]; 3]) -> f64 {
        m[0][0]*(m[1][1]*m[2][2]-m[1][2]*m[2][1])
        - m[0][1]*(m[1][0]*m[2][2]-m[1][2]*m[2][0])
        + m[0][2]*(m[1][0]*m[2][1]-m[1][1]*m[2][0])
    }

    fn minor(m: &[[f64; 4]; 4], r: usize, c: usize) -> [[f64; 3]; 3] {
        let mut sub = [[0.0f64; 3]; 3];
        let mut si = 0;
        for i in 0..4 { if i == r { continue; } let mut sj = 0; for j in 0..4 { if j == c { continue; } sub[si][sj] = m[i][j]; sj += 1; } si += 1; }
        sub
    }

    pub fn det(m: &[[f64; 4]; 4]) -> f64 {
        let mut d = 0.0f64;
        for j in 0..4 {
            let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
            d += sign * m[0][j] * Self::det3(&Self::minor(m, 0, j));
        }
        d
    }

    pub fn transpose(m: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
        let mut t = [[0.0f64; 4]; 4];
        for i in 0..4 { for j in 0..4 { t[i][j] = m[j][i]; } }
        t
    }

    pub fn trace(m: &[[f64; 4]; 4]) -> f64 { m[0][0] + m[1][1] + m[2][2] + m[3][3] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_det() { assert!((Mat4::det(&Mat4::identity()) - 1.0).abs() < 1e-9); }

    #[test]
    fn mul_identity() {
        let a = Mat4::identity();
        let r = Mat4::mul(&a, &Mat4::identity());
        for i in 0..4 { for j in 0..4 { assert!((r[i][j] - a[i][j]).abs() < 1e-9); } }
    }

    #[test]
    fn det_known() {
        let mut m = Mat4::identity();
        m[0][0] = 2.0;
        assert!((Mat4::det(&m) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn transpose_twice() {
        let m = Mat4::identity();
        let tt = Mat4::transpose(&Mat4::transpose(&m));
        for i in 0..4 { for j in 0..4 { assert!((tt[i][j] - m[i][j]).abs() < 1e-9); } }
    }

    #[test]
    fn trace() { assert!((Mat4::trace(&Mat4::identity()) - 4.0).abs() < 1e-9); }

    #[test]
    fn det_singular() {
        let m = [[1.0,0.0,0.0,0.0],[2.0,0.0,0.0,0.0],[3.0,0.0,0.0,0.0],[4.0,0.0,0.0,0.0]];
        assert!(Mat4::det(&m).abs() < 1e-9);
    }
}

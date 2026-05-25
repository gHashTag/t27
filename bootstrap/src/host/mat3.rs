pub struct Mat3;

impl Mat3 {
    pub fn identity() -> [[f64; 3]; 3] { [[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]] }

    pub fn mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let mut c = [[0.0f64; 3]; 3];
        for i in 0..3 { for j in 0..3 { for k in 0..3 { c[i][j] += a[i][k] * b[k][j]; } } }
        c
    }

    pub fn det(m: &[[f64; 3]; 3]) -> f64 {
        m[0][0]*(m[1][1]*m[2][2]-m[1][2]*m[2][1])
        - m[0][1]*(m[1][0]*m[2][2]-m[1][2]*m[2][0])
        + m[0][2]*(m[1][0]*m[2][1]-m[1][1]*m[2][0])
    }

    pub fn transpose(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
        [[m[0][0],m[1][0],m[2][0]],[m[0][1],m[1][1],m[2][1]],[m[0][2],m[1][2],m[2][2]]]
    }

    pub fn scale(m: &[[f64; 3]; 3], s: f64) -> [[f64; 3]; 3] {
        let mut r = *m;
        for i in 0..3 { for j in 0..3 { r[i][j] *= s; } }
        r
    }

    pub fn add(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let mut c = [[0.0f64; 3]; 3];
        for i in 0..3 { for j in 0..3 { c[i][j] = a[i][j] + b[i][j]; } }
        c
    }

    pub fn trace(m: &[[f64; 3]; 3]) -> f64 { m[0][0] + m[1][1] + m[2][2] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_det() { assert!((Mat3::det(&Mat3::identity()) - 1.0).abs() < 1e-9); }

    #[test]
    fn mul_identity() {
        let a = [[1.0,2.0,3.0],[4.0,5.0,6.0],[7.0,8.0,9.0]];
        let r = Mat3::mul(&a, &Mat3::identity());
        for i in 0..3 { for j in 0..3 { assert!((r[i][j] - a[i][j]).abs() < 1e-9); } }
    }

    #[test]
    fn det_zero() {
        let m = [[1.0,2.0,3.0],[4.0,5.0,6.0],[7.0,8.0,9.0]];
        assert!(Mat3::det(&m).abs() < 1e-9);
    }

    #[test]
    fn transpose_twice() {
        let m = [[1.0,2.0,3.0],[4.0,5.0,6.0],[7.0,8.0,9.0]];
        let tt = Mat3::transpose(&Mat3::transpose(&m));
        for i in 0..3 { for j in 0..3 { assert!((tt[i][j] - m[i][j]).abs() < 1e-9); } }
    }

    #[test]
    fn trace() { let m = [[1.0,0.0,0.0],[0.0,2.0,0.0],[0.0,0.0,3.0]]; assert!((Mat3::trace(&m) - 6.0).abs() < 1e-9); }

    #[test]
    fn add() {
        let a = [[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]];
        let b = [[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]];
        let c = Mat3::add(&a, &b);
        assert!((c[0][0] - 2.0).abs() < 1e-9);
    }
}

pub struct Gf1024;

impl Gf1024 {
    const PRIM: u16 = 0x409;
    const GEN: u16 = 2;

    pub fn mul(a: u16, b: u16) -> u16 {
        let mut result = 0u16;
        let mut a = a & 0x3FF;
        let mut b = b & 0x3FF;
        while b > 0 {
            if b & 1 != 0 { result ^= a; }
            a <<= 1;
            if a & 0x400 != 0 { a ^= Self::PRIM; }
            b >>= 1;
        }
        result & 0x3FF
    }

    pub fn add(a: u16, b: u16) -> u16 { (a ^ b) & 0x3FF }
    pub fn sub(a: u16, b: u16) -> u16 { Self::add(a, b) }

    pub fn pow(mut base: u16, mut exp: u16) -> u16 {
        let mut result = 1u16;
        base &= 0x3FF;
        while exp > 0 {
            if exp & 1 != 0 { result = Self::mul(result, base); }
            base = Self::mul(base, base);
            exp >>= 1;
        }
        result
    }

    pub fn inv(a: u16) -> u16 { Self::pow(a, 1022) }

    pub fn div(a: u16, b: u16) -> u16 { Self::mul(a, Self::inv(b)) }

    pub fn is_zero(a: u16) -> bool { (a & 0x3FF) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = 123u16; let b = 456u16;
        assert_eq!(Gf1024::add(a, b), Gf1024::sub(a, b));
    }

    #[test]
    fn mul_identity() { assert_eq!(Gf1024::mul(42, 1), 42); }

    #[test]
    fn mul_commutative() { assert_eq!(Gf1024::mul(100, 200), Gf1024::mul(200, 100)); }

    #[test]
    fn inv_roundtrip() {
        for v in [1u16, 2, 5, 100, 500, 1023] {
            let inv = Gf1024::inv(v);
            assert_eq!(Gf1024::mul(v, inv), 1);
        }
    }

    #[test]
    fn div() { assert_eq!(Gf1024::div(6, 2), Gf1024::mul(6, Gf1024::inv(2))); }

    #[test]
    fn field_order() { assert_eq!(Gf1024::pow(2, 1023), 1); }
}

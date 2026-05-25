pub struct Gf2048;

impl Gf2048 {
    const PRIM: u16 = 0x805;
    const GEN: u16 = 2;

    pub fn mul(a: u16, b: u16) -> u16 {
        let mut result = 0u16;
        let mut a = a & 0x7FF;
        let mut b = b & 0x7FF;
        while b > 0 {
            if b & 1 != 0 { result ^= a; }
            a <<= 1;
            if a & 0x800 != 0 { a ^= Self::PRIM; }
            b >>= 1;
        }
        result & 0x7FF
    }

    pub fn add(a: u16, b: u16) -> u16 { (a ^ b) & 0x7FF }
    pub fn sub(a: u16, b: u16) -> u16 { Self::add(a, b) }

    pub fn pow(mut base: u16, mut exp: u16) -> u16 {
        let mut result = 1u16;
        base &= 0x7FF;
        while exp > 0 {
            if exp & 1 != 0 { result = Self::mul(result, base); }
            base = Self::mul(base, base);
            exp >>= 1;
        }
        result
    }

    pub fn inv(a: u16) -> u16 { Self::pow(a, 2046) }

    pub fn div(a: u16, b: u16) -> u16 { Self::mul(a, Self::inv(b)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() { let a = 123u16; let b = 456u16; assert_eq!(Gf2048::add(a, b), Gf2048::sub(a, b)); }

    #[test]
    fn mul_identity() { assert_eq!(Gf2048::mul(42, 1), 42); }

    #[test]
    fn mul_commutative() { assert_eq!(Gf2048::mul(100, 200), Gf2048::mul(200, 100)); }

    #[test]
    fn inv_roundtrip() {
        for v in [1u16, 2, 5, 100, 500, 2047] {
            assert_eq!(Gf2048::mul(v, Gf2048::inv(v)), 1);
        }
    }

    #[test]
    fn div() { assert_eq!(Gf2048::div(6, 2), Gf2048::mul(6, Gf2048::inv(2))); }

    #[test]
    fn field_order() { assert_eq!(Gf2048::pow(2, 2047), 1); }
}

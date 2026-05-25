#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Simd8 {
    lanes: [u8; 8],
}

impl Simd8 {
    pub fn new(vals: [u8; 8]) -> Self { Self { lanes: vals } }
    pub fn splat(v: u8) -> Self { Self { lanes: [v; 8] } }
    pub fn zero() -> Self { Self { lanes: [0; 8] } }

    pub fn add(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i].wrapping_add(other.lanes[i]); }
        Simd8::new(r)
    }

    pub fn sub(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i].wrapping_sub(other.lanes[i]); }
        Simd8::new(r)
    }

    pub fn mul_lo(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i].wrapping_mul(other.lanes[i]); }
        Simd8::new(r)
    }

    pub fn min(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i].min(other.lanes[i]); }
        Simd8::new(r)
    }

    pub fn max(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i].max(other.lanes[i]); }
        Simd8::new(r)
    }

    pub fn and(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i] & other.lanes[i]; }
        Simd8::new(r)
    }

    pub fn or(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i] | other.lanes[i]; }
        Simd8::new(r)
    }

    pub fn xor(&self, other: &Simd8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i] ^ other.lanes[i]; }
        Simd8::new(r)
    }

    pub fn shl(&self, bits: u8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i] << bits; }
        Simd8::new(r)
    }

    pub fn shr(&self, bits: u8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = self.lanes[i] >> bits; }
        Simd8::new(r)
    }

    pub fn eq_mask(&self, other: &Simd8) -> u8 {
        let mut m = 0u8;
        for i in 0..8 { if self.lanes[i] == other.lanes[i] { m |= 1 << i; } }
        m
    }

    pub fn gt_mask(&self, other: &Simd8) -> u8 {
        let mut m = 0u8;
        for i in 0..8 { if self.lanes[i] > other.lanes[i] { m |= 1 << i; } }
        m
    }

    pub fn blend(&self, other: &Simd8, mask: u8) -> Simd8 {
        let mut r = [0u8; 8];
        for i in 0..8 { r[i] = if mask & (1 << i) != 0 { other.lanes[i] } else { self.lanes[i] }; }
        Simd8::new(r)
    }

    pub fn sum(&self) -> u64 { self.lanes.iter().map(|&v| v as u64).sum() }
    pub fn horizontal_max(&self) -> u8 { *self.lanes.iter().max().unwrap() }
    pub fn horizontal_min(&self) -> u8 { *self.lanes.iter().min().unwrap() }
    pub fn lanes(&self) -> &[u8; 8] { &self.lanes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_splat() { let v = Simd8::splat(5); assert_eq!(v.lanes(), &[5; 8]); }

    #[test]
    fn add() {
        let a = Simd8::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let b = Simd8::new([10, 20, 30, 40, 50, 60, 70, 80]);
        let r = a.add(&b);
        assert_eq!(r.lanes(), &[11, 22, 33, 44, 55, 66, 77, 88]);
    }

    #[test]
    fn sub() {
        let a = Simd8::new([10, 20, 30, 40, 50, 60, 70, 80]);
        let b = Simd8::splat(5);
        let r = a.sub(&b);
        assert_eq!(r.lanes(), &[5, 15, 25, 35, 45, 55, 65, 75]);
    }

    #[test]
    fn mul_lo() {
        let a = Simd8::new([2, 3, 4, 5, 6, 7, 8, 9]);
        let b = Simd8::splat(10);
        let r = a.mul_lo(&b);
        assert_eq!(r.lanes()[0], 20);
    }

    #[test]
    fn min_max() {
        let a = Simd8::new([1, 9, 3, 7, 5, 2, 8, 4]);
        let b = Simd8::splat(5);
        assert_eq!(a.min(&b).lanes(), &[1, 5, 3, 5, 5, 2, 5, 4]);
        assert_eq!(a.max(&b).lanes(), &[5, 9, 5, 7, 5, 5, 8, 5]);
    }

    #[test]
    fn bitwise() {
        let a = Simd8::new([0xFF, 0xF0, 0x0F, 0x00, 0xFF, 0xAA, 0x55, 0x88]);
        let b = Simd8::splat(0x0F);
        let and = a.and(&b);
        assert_eq!(and.lanes()[0], 0x0F);
        assert_eq!(and.lanes()[1], 0x00);
    }

    #[test]
    fn eq_gt_mask() {
        let a = Simd8::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let b = Simd8::new([1, 0, 3, 0, 5, 0, 7, 0]);
        assert_eq!(a.eq_mask(&b), 0b01010101);
        assert_eq!(a.gt_mask(&b), 0b10101010);
    }

    #[test]
    fn blend() {
        let a = Simd8::splat(0);
        let b = Simd8::splat(255);
        let r = a.blend(&b, 0b10101010);
        assert_eq!(r.lanes(), &[0, 255, 0, 255, 0, 255, 0, 255]);
    }

    #[test]
    fn reduce() {
        let v = Simd8::new([1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(v.sum(), 36);
        assert_eq!(v.horizontal_max(), 8);
        assert_eq!(v.horizontal_min(), 1);
    }

    #[test]
    fn overflow() {
        let a = Simd8::splat(250);
        let b = Simd8::splat(10);
        let r = a.add(&b);
        assert_eq!(r.lanes()[0], 4);
    }
}

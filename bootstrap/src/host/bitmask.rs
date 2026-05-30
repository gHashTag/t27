pub const fn mask(width: u8) -> u32 {
    if width >= 32 { 0xFFFF_FFFF } else { (1u32 << width) - 1 }
}

pub const fn field_mask(high: u8, low: u8) -> u32 {
    mask(high - low + 1) << low
}

pub const fn extract(value: u32, high: u8, low: u8) -> u32 {
    (value >> low) & mask(high - low + 1)
}

pub const fn insert(value: u32, high: u8, low: u8, field: u32) -> u32 {
    let m = field_mask(high, low);
    (value & !m) | ((field << low) & m)
}

pub const fn is_set(value: u32, bit: u8) -> bool {
    (value & (1u32 << bit)) != 0
}

pub const fn set_bit(value: u32, bit: u8) -> u32 {
    value | (1u32 << bit)
}

pub const fn clear_bit(value: u32, bit: u8) -> u32 {
    value & !(1u32 << bit)
}

pub const fn toggle_bit(value: u32, bit: u8) -> u32 {
    value ^ (1u32 << bit)
}

pub fn popcount(value: u32) -> u8 {
    value.count_ones() as u8
}

pub fn trailing_zeros(value: u32) -> u8 {
    value.trailing_zeros() as u8
}

pub fn leading_zeros(value: u32) -> u8 {
    value.leading_zeros() as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDesc {
    pub name: &'static str,
    pub high: u8,
    pub low: u8,
}

impl FieldDesc {
    pub const fn new(name: &'static str, high: u8, low: u8) -> Self {
        Self { name, high, low }
    }

    pub const fn width(self) -> u8 {
        self.high - self.low + 1
    }

    pub const fn mask(self) -> u32 {
        field_mask(self.high, self.low)
    }

    pub const fn extract(self, value: u32) -> u32 {
        extract(value, self.high, self.low)
    }

    pub const fn insert(self, value: u32, field: u32) -> u32 {
        insert(value, self.high, self.low, field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_1bit() {
        assert_eq!(mask(1), 0x1);
    }

    #[test]
    fn mask_8bit() {
        assert_eq!(mask(8), 0xFF);
    }

    #[test]
    fn mask_32bit() {
        assert_eq!(mask(32), 0xFFFF_FFFF);
    }

    #[test]
    fn field_mask_bits_7_4() {
        assert_eq!(field_mask(7, 4), 0xF0);
    }

    #[test]
    fn extract_field() {
        let v = 0xABCD;
        assert_eq!(extract(v, 11, 8), 0xB);
        assert_eq!(extract(v, 3, 0), 0xD);
    }

    #[test]
    fn insert_field() {
        let v = 0x0000;
        assert_eq!(insert(v, 7, 4, 0xA), 0x00A0);
    }

    #[test]
    fn insert_preserves_other() {
        let v = 0x1234;
        let result = insert(v, 7, 4, 0x0);
        assert_eq!(extract(result, 15, 8), 0x12);
        assert_eq!(extract(result, 3, 0), 0x4);
    }

    #[test]
    fn is_set_clear() {
        assert!(is_set(0x04, 2));
        assert!(!is_set(0x04, 1));
    }

    #[test]
    fn set_clear_toggle() {
        assert_eq!(set_bit(0, 3), 0x08);
        assert_eq!(clear_bit(0xFF, 0), 0xFE);
        assert_eq!(toggle_bit(0x0F, 3), 0x07);
        assert_eq!(toggle_bit(0x07, 3), 0x0F);
    }

    #[test]
    fn popcount_test() {
        assert_eq!(popcount(0xFF), 8);
        assert_eq!(popcount(0), 0);
        assert_eq!(popcount(1), 1);
    }

    #[test]
    fn trailing_zeros_test() {
        assert_eq!(trailing_zeros(0x08), 3);
        assert_eq!(trailing_zeros(1), 0);
    }

    #[test]
    fn field_desc() {
        let f = FieldDesc::new("ctrl_start", 0, 0);
        assert_eq!(f.width(), 1);
        assert_eq!(f.mask(), 0x1);
        assert_eq!(f.extract(0x01), 1);
        assert_eq!(f.insert(0, 1), 1);
    }

    #[test]
    fn field_desc_wide() {
        let f = FieldDesc::new("neurons", 15, 0);
        assert_eq!(f.width(), 16);
        assert_eq!(f.mask(), 0xFFFF);
        assert_eq!(f.extract(0x00FF_0000), 0);
        assert_eq!(f.extract(0x0000_ABCD), 0xABCD);
    }

    #[test]
    fn insert_overwrite() {
        let v = insert(0x00F0, 7, 4, 0x5);
        assert_eq!(v, 0x0050);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BiError {
    DivisionByZero,
    NegativeSubtraction,
}

impl std::fmt::Display for BiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiError::DivisionByZero => write!(f, "division by zero"),
            BiError::NegativeSubtraction => write!(f, "negative result"),
        }
    }
}

impl std::error::Error for BiError {}

#[derive(Debug, Clone, PartialEq)]
pub struct BigInt2 {
    limbs: Vec<u32>,
}

impl BigInt2 {
    pub fn zero() -> Self { Self { limbs: vec![0] } }
    pub fn one() -> Self { Self { limbs: vec![1] } }
    pub fn from_u64(v: u64) -> Self { Self { limbs: vec![v as u32, (v >> 32) as u32] } }
    pub fn from_u32(v: u32) -> Self { Self { limbs: vec![v] } }

    pub fn is_zero(&self) -> bool { self.limbs.iter().all(|&l| l == 0) }

    fn trim(&mut self) {
        while self.limbs.len() > 1 && *self.limbs.last().unwrap() == 0 { self.limbs.pop(); }
    }

    pub fn add(&self, other: &BigInt2) -> BigInt2 {
        let n = self.limbs.len().max(other.limbs.len());
        let mut result = Vec::with_capacity(n + 1);
        let mut carry: u64 = 0;
        for i in 0..n {
            let a = *self.limbs.get(i).unwrap_or(&0) as u64;
            let b = *other.limbs.get(i).unwrap_or(&0) as u64;
            let sum = a + b + carry;
            result.push(sum as u32);
            carry = sum >> 32;
        }
        if carry > 0 { result.push(carry as u32); }
        let mut r = BigInt2 { limbs: result }; r.trim(); r
    }

    pub fn sub(&self, other: &BigInt2) -> Result<BigInt2, BiError> {
        if self.cmp(other) == std::cmp::Ordering::Less { return Err(BiError::NegativeSubtraction); }
        let n = self.limbs.len();
        let mut result = Vec::with_capacity(n);
        let mut borrow: u64 = 0;
        for i in 0..n {
            let a = *self.limbs.get(i).unwrap_or(&0) as u64;
            let b = *other.limbs.get(i).unwrap_or(&0) as u64;
            let diff = (1u64 << 32) + a - b - borrow;
            result.push(diff as u32);
            borrow = if diff >> 32 == 0 { 1 } else { 0 };
        }
        let mut r = BigInt2 { limbs: result }; r.trim(); Ok(r)
    }

    pub fn mul(&self, other: &BigInt2) -> BigInt2 {
        let n = self.limbs.len() + other.limbs.len();
        let mut result = vec![0u64; n];
        for i in 0..self.limbs.len() {
            let mut carry: u64 = 0;
            for j in 0..other.limbs.len() {
                let prod = (self.limbs[i] as u64) * (other.limbs[j] as u64) + result[i + j] + carry;
                result[i + j] = prod & 0xFFFFFFFF;
                carry = prod >> 32;
            }
            result[i + other.limbs.len()] += carry;
        }
        let mut r = BigInt2 { limbs: result.iter().map(|&v| v as u32).collect() }; r.trim(); r
    }

    pub fn shl(&self, bits: usize) -> BigInt2 {
        if bits == 0 || self.is_zero() { return self.clone(); }
        let word_shift = bits / 32;
        let bit_shift = bits % 32;
        let mut result = vec![0u32; word_shift + self.limbs.len() + 1];
        for i in 0..self.limbs.len() {
            let v = self.limbs[i] as u64;
            result[i + word_shift] |= (v << bit_shift) as u32;
            if bit_shift > 0 && i + word_shift + 1 < result.len() {
                result[i + word_shift + 1] |= (v >> (32 - bit_shift)) as u32;
            }
        }
        let mut r = BigInt2 { limbs: result }; r.trim(); r
    }

    pub fn cmp(&self, other: &BigInt2) -> std::cmp::Ordering {
        let a_len = self.limbs.iter().rposition(|&l| l != 0).map(|i| i + 1).unwrap_or(0);
        let b_len = other.limbs.iter().rposition(|&l| l != 0).map(|i| i + 1).unwrap_or(0);
        match a_len.cmp(&b_len) {
            std::cmp::Ordering::Equal => {
                for i in (0..a_len).rev() {
                    match self.limbs[i].cmp(&other.limbs[i]) {
                        std::cmp::Ordering::Equal => continue,
                        o => return o,
                    }
                }
                std::cmp::Ordering::Equal
            }
            o => o,
        }
    }

    pub fn to_u64(&self) -> u64 {
        let lo = *self.limbs.get(0).unwrap_or(&0) as u64;
        let hi = *self.limbs.get(1).unwrap_or(&0) as u64;
        lo | (hi << 32)
    }

    pub fn bit_len(&self) -> usize {
        if self.is_zero() { return 0; }
        let top = self.limbs.iter().rposition(|&l| l != 0).unwrap();
        top * 32 + (32 - self.limbs[top].leading_zeros() as usize)
    }

    pub fn limbs(&self) -> &[u32] { &self.limbs }
}

impl std::cmp::Ord for BigInt2 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.cmp(other) }
}
impl std::cmp::PartialOrd for BigInt2 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_one() { assert!(BigInt2::zero().is_zero()); assert!(!BigInt2::one().is_zero()); }

    #[test]
    fn add_simple() {
        let a = BigInt2::from_u32(100);
        let b = BigInt2::from_u32(200);
        assert_eq!(a.add(&b).to_u64(), 300);
    }

    #[test]
    fn add_overflow() {
        let a = BigInt2::from_u32(0xFFFFFFFF);
        let b = BigInt2::from_u32(1);
        let r = a.add(&b);
        assert_eq!(r.limbs()[0], 0);
        assert_eq!(r.limbs()[1], 1);
    }

    #[test]
    fn sub() {
        let a = BigInt2::from_u32(100);
        let b = BigInt2::from_u32(30);
        assert_eq!(a.sub(&b).unwrap().to_u64(), 70);
    }

    #[test]
    fn sub_negative() { assert!(BigInt2::from_u32(1).sub(&BigInt2::from_u32(2)).is_err()); }

    #[test]
    fn mul() {
        let a = BigInt2::from_u64(1000);
        let b = BigInt2::from_u64(2000);
        assert_eq!(a.mul(&b).to_u64(), 2_000_000);
    }

    #[test]
    fn mul_large() {
        let a = BigInt2::from_u64(0xFFFFFFFF);
        let b = BigInt2::from_u64(0xFFFFFFFF);
        let r = a.mul(&b);
        assert!(r.bit_len() > 32);
    }

    #[test]
    fn shl() {
        let a = BigInt2::from_u32(1);
        let r = a.shl(32);
        assert_eq!(r.limbs()[0], 0);
        assert_eq!(r.limbs()[1], 1);
    }

    #[test]
    fn cmp_ord() {
        let a = BigInt2::from_u32(10);
        let b = BigInt2::from_u32(20);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a);
    }

    #[test]
    fn bit_len() {
        assert_eq!(BigInt2::from_u32(255).bit_len(), 8);
        assert_eq!(BigInt2::from_u32(256).bit_len(), 9);
        assert_eq!(BigInt2::zero().bit_len(), 0);
    }

    #[test]
    fn error_display() { assert!(BiError::DivisionByZero.to_string().contains("zero")); }
}

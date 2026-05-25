#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigUint {
    digits: Vec<u64>,
}

impl BigUint {
    pub fn zero() -> Self { Self { digits: vec![0] } }
    pub fn one() -> Self { Self { digits: vec![1] } }

    pub fn from_u64(v: u64) -> Self { Self { digits: vec![v] } }

    pub fn from_bytes_be(bytes: &[u8]) -> Self {
        let mut digits = Vec::new();
        let mut i = bytes.len();
        while i >= 8 {
            digits.push(u64::from_be_bytes(bytes[i - 8..i].try_into().unwrap()));
            i -= 8;
        }
        if i > 0 {
            let mut arr = [0u8; 8];
            arr[8 - i..].copy_from_slice(&bytes[..i]);
            digits.push(u64::from_be_bytes(arr));
        }
        if digits.is_empty() { digits.push(0); }
        let mut r = Self { digits };
        r.normalize();
        r
    }

    fn normalize(&mut self) {
        while self.digits.len() > 1 && *self.digits.last().unwrap() == 0 {
            self.digits.pop();
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        let n = self.digits.len().max(other.digits.len());
        let mut result = Vec::with_capacity(n + 1);
        let mut carry = 0u64;
        for i in 0..n {
            let a = if i < self.digits.len() { self.digits[i] } else { 0 };
            let b = if i < other.digits.len() { other.digits[i] } else { 0 };
            let (sum, c1) = a.overflowing_add(b);
            let (sum, c2) = sum.overflowing_add(carry);
            result.push(sum);
            carry = c1 as u64 + c2 as u64;
        }
        if carry > 0 { result.push(carry); }
        let mut r = Self { digits: result };
        r.normalize();
        r
    }

    pub fn mul_u64(&self, v: u64) -> Self {
        let mut result = Vec::with_capacity(self.digits.len() + 1);
        let mut carry = 0u128;
        for &d in &self.digits {
            let prod = d as u128 * v as u128 + carry;
            result.push(prod as u64);
            carry = prod >> 64;
        }
        if carry > 0 { result.push(carry as u64); }
        let mut r = Self { digits: result };
        r.normalize();
        r
    }

    pub fn is_zero(&self) -> bool { self.digits.len() == 1 && self.digits[0] == 0 }

    pub fn to_u64(&self) -> u64 { self.digits[0] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_simple() {
        let a = BigUint::from_u64(100);
        let b = BigUint::from_u64(200);
        assert_eq!(a.add(&b).to_u64(), 300);
    }

    #[test]
    fn add_overflow() {
        let a = BigUint::from_u64(u64::MAX);
        let b = BigUint::from_u64(1);
        let r = a.add(&b);
        assert_eq!(r.digits.len(), 2);
        assert_eq!(r.digits[0], 0);
        assert_eq!(r.digits[1], 1);
    }

    #[test]
    fn mul_simple() {
        let a = BigUint::from_u64(12345);
        assert_eq!(a.mul_u64(100).to_u64(), 1_234_500);
    }

    #[test]
    fn from_bytes() {
        let b = BigUint::from_bytes_be(&[1, 0]);
        assert_eq!(b.to_u64(), 256);
    }

    #[test]
    fn zero_one() {
        assert!(BigUint::zero().is_zero());
        assert!(!BigUint::one().is_zero());
    }

    #[test]
    fn mul_overflow() {
        let a = BigUint::from_u64(u64::MAX);
        let r = a.mul_u64(2);
        assert_eq!(r.digits.len(), 2);
    }
}

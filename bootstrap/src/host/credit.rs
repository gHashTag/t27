#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreditError {
    Insufficient { requested: u64, available: u64 },
    Overflow,
}

impl std::fmt::Display for CreditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreditError::Insufficient { requested, available } => {
                write!(f, "need {requested} credits, have {available}")
            }
            CreditError::Overflow => write!(f, "credit overflow"),
        }
    }
}

impl std::error::Error for CreditError {}

#[derive(Debug, Clone)]
pub struct CreditCounter {
    balance: u64,
    max_balance: u64,
    refill_rate: u64,
    total_consumed: u64,
    total_refilled: u64,
    total_rejected: u64,
}

impl CreditCounter {
    pub fn new(max_balance: u64, refill_rate: u64) -> Self {
        Self {
            balance: max_balance,
            max_balance,
            refill_rate,
            total_consumed: 0,
            total_refilled: 0,
            total_rejected: 0,
        }
    }

    pub fn with_initial(mut self, initial: u64) -> Self {
        self.balance = initial.min(self.max_balance);
        self
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn max_balance(&self) -> u64 {
        self.max_balance
    }

    pub fn refill_rate(&self) -> u64 {
        self.refill_rate
    }

    pub fn available(&self) -> u64 {
        self.balance
    }

    pub fn try_consume(&mut self, amount: u64) -> Result<u64, CreditError> {
        if amount > self.balance {
            self.total_rejected += 1;
            return Err(CreditError::Insufficient {
                requested: amount,
                available: self.balance,
            });
        }
        self.balance -= amount;
        self.total_consumed += amount;
        Ok(self.balance)
    }

    pub fn consume_max(&mut self, amount: u64) -> u64 {
        let actual = amount.min(self.balance);
        self.balance -= actual;
        self.total_consumed += actual;
        actual
    }

    pub fn refill(&mut self) -> u64 {
        let added = self.refill_rate.min(self.max_balance - self.balance);
        self.balance += added;
        self.total_refilled += added;
        added
    }

    pub fn refill_by(&mut self, amount: u64) -> u64 {
        let added = amount.min(self.max_balance - self.balance);
        self.balance += added;
        self.total_refilled += added;
        added
    }

    pub fn set_balance(&mut self, value: u64) {
        self.balance = value.min(self.max_balance);
    }

    pub fn reset(&mut self) {
        self.balance = self.max_balance;
    }

    pub fn utilization(&self) -> f64 {
        if self.max_balance == 0 { 0.0 } else { self.balance as f64 / self.max_balance as f64 }
    }

    pub fn total_consumed(&self) -> u64 {
        self.total_consumed
    }

    pub fn total_refilled(&self) -> u64 {
        self.total_refilled
    }

    pub fn total_rejected(&self) -> u64 {
        self.total_rejected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_credit_counter() {
        let cc = CreditCounter::new(100, 10);
        assert_eq!(cc.balance(), 100);
        assert_eq!(cc.max_balance(), 100);
        assert_eq!(cc.refill_rate(), 10);
    }

    #[test]
    fn with_initial() {
        let cc = CreditCounter::new(100, 10).with_initial(50);
        assert_eq!(cc.balance(), 50);
    }

    #[test]
    fn with_initial_clamped() {
        let cc = CreditCounter::new(100, 10).with_initial(200);
        assert_eq!(cc.balance(), 100);
    }

    #[test]
    fn consume_success() {
        let mut cc = CreditCounter::new(100, 10);
        let remaining = cc.try_consume(30).unwrap();
        assert_eq!(remaining, 70);
        assert_eq!(cc.balance(), 70);
    }

    #[test]
    fn consume_insufficient() {
        let mut cc = CreditCounter::new(100, 10);
        let err = cc.try_consume(150).unwrap_err();
        assert!(matches!(err, CreditError::Insufficient { requested: 150, available: 100 }));
        assert_eq!(cc.total_rejected(), 1);
    }

    #[test]
    fn consume_max() {
        let mut cc = CreditCounter::new(50, 10);
        let actual = cc.consume_max(100);
        assert_eq!(actual, 50);
        assert_eq!(cc.balance(), 0);
    }

    #[test]
    fn refill_to_max() {
        let mut cc = CreditCounter::new(100, 30).with_initial(80);
        let added = cc.refill();
        assert_eq!(added, 20);
        assert_eq!(cc.balance(), 100);
    }

    #[test]
    fn refill_by() {
        let mut cc = CreditCounter::new(100, 10).with_initial(0);
        let added = cc.refill_by(25);
        assert_eq!(added, 25);
        assert_eq!(cc.balance(), 25);
    }

    #[test]
    fn refill_by_clamped() {
        let mut cc = CreditCounter::new(100, 10).with_initial(90);
        let added = cc.refill_by(50);
        assert_eq!(added, 10);
        assert_eq!(cc.balance(), 100);
    }

    #[test]
    fn stats() {
        let mut cc = CreditCounter::new(100, 10);
        cc.try_consume(40).unwrap();
        cc.refill();
        assert_eq!(cc.total_consumed(), 40);
        assert_eq!(cc.total_refilled(), 10);
        assert_eq!(cc.total_rejected(), 0);
    }

    #[test]
    fn reset() {
        let mut cc = CreditCounter::new(100, 10).with_initial(0);
        cc.reset();
        assert_eq!(cc.balance(), 100);
    }

    #[test]
    fn set_balance() {
        let mut cc = CreditCounter::new(100, 10);
        cc.set_balance(42);
        assert_eq!(cc.balance(), 42);
        cc.set_balance(999);
        assert_eq!(cc.balance(), 100);
    }

    #[test]
    fn utilization() {
        let cc = CreditCounter::new(200, 10).with_initial(50);
        assert!((cc.utilization() - 0.25).abs() < 0.001);
    }

    #[test]
    fn error_display() {
        let e = CreditError::Insufficient { requested: 10, available: 5 };
        assert!(e.to_string().contains("10"));
    }
}

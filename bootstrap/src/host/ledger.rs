use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LedgerError {
    AccountExists { id: u64 },
    AccountNotFound { id: u64 },
    InsufficientFunds { id: u64, balance: i64, amount: i64 },
    NegativeAmount { amount: i64 },
    SelfTransfer { id: u64 },
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerError::AccountExists { id } => write!(f, "account {id} exists"),
            LedgerError::AccountNotFound { id } => write!(f, "account {id} not found"),
            LedgerError::InsufficientFunds { id, balance, amount } => write!(f, "account {id}: {balance} < {amount}"),
            LedgerError::NegativeAmount { amount } => write!(f, "negative amount {amount}"),
            LedgerError::SelfTransfer { id } => write!(f, "self-transfer {id}"),
        }
    }
}

impl std::error::Error for LedgerError {}

struct Account {
    id: u64,
    balance: i64,
    credit_count: u64,
    debit_count: u64,
}

pub struct Ledger {
    accounts: BTreeMap<u64, Account>,
    total_transfers: u64,
    total_volume: i64,
}

impl Ledger {
    pub fn new() -> Self { Self { accounts: BTreeMap::new(), total_transfers: 0, total_volume: 0 } }

    pub fn create_account(&mut self, id: u64, initial: i64) -> Result<(), LedgerError> {
        if self.accounts.contains_key(&id) { return Err(LedgerError::AccountExists { id }); }
        self.accounts.insert(id, Account { id, balance: initial, credit_count: 0, debit_count: 0 });
        Ok(())
    }

    pub fn balance(&self, id: u64) -> Option<i64> { self.accounts.get(&id).map(|a| a.balance) }

    pub fn transfer(&mut self, from: u64, to: u64, amount: i64) -> Result<(), LedgerError> {
        if amount < 0 { return Err(LedgerError::NegativeAmount { amount }); }
        if amount == 0 { return Ok(()); }
        if from == to { return Err(LedgerError::SelfTransfer { id: from }); }
        let from_bal = self.accounts.get(&from).ok_or(LedgerError::AccountNotFound { id: from })?.balance;
        if from_bal < amount { return Err(LedgerError::InsufficientFunds { id: from, balance: from_bal, amount }); }
        if !self.accounts.contains_key(&to) { return Err(LedgerError::AccountNotFound { id: to }); }
        self.accounts.get_mut(&from).unwrap().balance -= amount;
        self.accounts.get_mut(&from).unwrap().debit_count += 1;
        self.accounts.get_mut(&to).unwrap().balance += amount;
        self.accounts.get_mut(&to).unwrap().credit_count += 1;
        self.total_transfers += 1;
        self.total_volume += amount;
        Ok(())
    }

    pub fn credit(&mut self, id: u64, amount: i64) -> Result<i64, LedgerError> {
        if amount < 0 { return Err(LedgerError::NegativeAmount { amount }); }
        let a = self.accounts.get_mut(&id).ok_or(LedgerError::AccountNotFound { id })?;
        a.balance += amount;
        a.credit_count += 1;
        Ok(a.balance)
    }

    pub fn debit(&mut self, id: u64, amount: i64) -> Result<i64, LedgerError> {
        if amount < 0 { return Err(LedgerError::NegativeAmount { amount }); }
        let a = self.accounts.get_mut(&id).ok_or(LedgerError::AccountNotFound { id })?;
        if a.balance < amount { return Err(LedgerError::InsufficientFunds { id, balance: a.balance, amount }); }
        a.balance -= amount;
        a.debit_count += 1;
        Ok(a.balance)
    }

    pub fn verify(&self) -> bool {
        let total: i64 = self.accounts.values().map(|a| a.balance).sum();
        total == 0
    }

    pub fn total_supply(&self) -> i64 { self.accounts.values().map(|a| a.balance).sum() }
    pub fn account_count(&self) -> usize { self.accounts.len() }
    pub fn total_transfers(&self) -> u64 { self.total_transfers }
    pub fn total_volume(&self) -> i64 { self.total_volume }
    pub fn credit_count(&self, id: u64) -> Option<u64> { self.accounts.get(&id).map(|a| a.credit_count) }
    pub fn debit_count(&self, id: u64) -> Option<u64> { self.accounts.get(&id).map(|a| a.debit_count) }
}

impl Default for Ledger {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ledger() { assert_eq!(Ledger::new().account_count(), 0); }

    #[test]
    fn create_balance() {
        let mut l = Ledger::new();
        l.create_account(1, 1000).unwrap();
        assert_eq!(l.balance(1), Some(1000));
    }

    #[test]
    fn transfer() {
        let mut l = Ledger::new();
        l.create_account(1, 1000).unwrap();
        l.create_account(2, 0).unwrap();
        l.transfer(1, 2, 300).unwrap();
        assert_eq!(l.balance(1), Some(700));
        assert_eq!(l.balance(2), Some(300));
    }

    #[test]
    fn insufficient() {
        let mut l = Ledger::new();
        l.create_account(1, 100).unwrap();
        l.create_account(2, 0).unwrap();
        let err = l.transfer(1, 2, 200).unwrap_err();
        assert!(matches!(err, LedgerError::InsufficientFunds { .. }));
    }

    #[test]
    fn not_found() {
        let mut l = Ledger::new();
        let err = l.transfer(1, 2, 10).unwrap_err();
        assert!(matches!(err, LedgerError::AccountNotFound { .. }));
    }

    #[test]
    fn self_transfer() {
        let mut l = Ledger::new();
        l.create_account(1, 100).unwrap();
        let err = l.transfer(1, 1, 10).unwrap_err();
        assert!(matches!(err, LedgerError::SelfTransfer { .. }));
    }

    #[test]
    fn credit_debit() {
        let mut l = Ledger::new();
        l.create_account(1, 0).unwrap();
        l.credit(1, 500).unwrap();
        l.debit(1, 200).unwrap();
        assert_eq!(l.balance(1), Some(300));
    }

    #[test]
    fn verify_zero() {
        let mut l = Ledger::new();
        l.create_account(1, 1000).unwrap();
        l.create_account(2, -1000).unwrap();
        assert!(l.verify());
    }

    #[test]
    fn stats() {
        let mut l = Ledger::new();
        l.create_account(1, 1000).unwrap();
        l.create_account(2, 0).unwrap();
        l.transfer(1, 2, 100).unwrap();
        assert_eq!(l.total_transfers(), 1);
        assert_eq!(l.total_volume(), 100);
        assert_eq!(l.credit_count(2), Some(1));
    }

    #[test]
    fn duplicate_account() {
        let mut l = Ledger::new();
        l.create_account(1, 0).unwrap();
        let err = l.create_account(1, 0).unwrap_err();
        assert!(matches!(err, LedgerError::AccountExists { .. }));
    }

    #[test]
    fn error_display() { assert!(LedgerError::AccountNotFound { id: 3 }.to_string().contains("3")); }
}

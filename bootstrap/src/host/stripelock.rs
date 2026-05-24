use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockState {
    Free,
    Shared { holders: u32 },
    Exclusive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StripeError {
    AlreadyHeld { stripe: usize, owner: u64 },
    NotHeld { stripe: usize, owner: u64 },
    Conflict { stripe: usize, state: LockState },
    OutOfRange { stripe: usize, count: usize },
}

impl std::fmt::Display for StripeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StripeError::AlreadyHeld { stripe, owner } => write!(f, "stripe {stripe} held by {owner}"),
            StripeError::NotHeld { stripe, owner } => write!(f, "{owner} does not hold stripe {stripe}"),
            StripeError::Conflict { stripe, state } => write!(f, "stripe {stripe} conflict: {:?}", state),
            StripeError::OutOfRange { stripe, count } => write!(f, "stripe {stripe} >= {count}"),
        }
    }
}

impl std::error::Error for StripeError {}

#[derive(Debug, Clone)]
struct StripeSlot {
    state: LockState,
    exclusive_owner: Option<u64>,
    shared_owners: Vec<u64>,
    total_locks: u64,
    total_unlocks: u64,
}

#[derive(Debug, Clone)]
pub struct StripeLockStats {
    pub stripe_count: usize,
    pub locked_stripes: usize,
    pub total_locks: u64,
    pub total_unlocks: u64,
    pub total_conflicts: u64,
}

#[derive(Debug, Clone)]
pub struct StripeLock {
    stripes: Vec<StripeSlot>,
    stripe_count: usize,
    total_conflicts: u64,
}

impl StripeLock {
    pub fn new(stripe_count: usize) -> Self {
        let stripes = (0..stripe_count).map(|_| StripeSlot {
            state: LockState::Free,
            exclusive_owner: None,
            shared_owners: Vec::new(),
            total_locks: 0,
            total_unlocks: 0,
        }).collect();
        Self { stripes, stripe_count, total_conflicts: 0 }
    }

    pub fn stripe_count(&self) -> usize {
        self.stripe_count
    }

    fn stripe_for(&self, key: u64) -> usize {
        (key as usize) % self.stripe_count.max(1)
    }

    pub fn lock_shared(&mut self, key: u64, owner: u64) -> Result<usize, StripeError> {
        let idx = self.stripe_for(key);
        let stripe = &mut self.stripes[idx];
        match stripe.state {
            LockState::Free => {
                stripe.state = LockState::Shared { holders: 1 };
                stripe.shared_owners.push(owner);
                stripe.total_locks += 1;
                Ok(idx)
            }
            LockState::Shared { .. } => {
                if stripe.shared_owners.contains(&owner) {
                    self.total_conflicts += 1;
                    return Err(StripeError::AlreadyHeld { stripe: idx, owner });
                }
                stripe.shared_owners.push(owner);
                stripe.state = LockState::Shared { holders: stripe.shared_owners.len() as u32 };
                stripe.total_locks += 1;
                Ok(idx)
            }
            LockState::Exclusive => {
                self.total_conflicts += 1;
                Err(StripeError::Conflict { stripe: idx, state: stripe.state.clone() })
            }
        }
    }

    pub fn lock_exclusive(&mut self, key: u64, owner: u64) -> Result<usize, StripeError> {
        let idx = self.stripe_for(key);
        let stripe = &mut self.stripes[idx];
        if stripe.state != LockState::Free {
            self.total_conflicts += 1;
            return Err(StripeError::Conflict { stripe: idx, state: stripe.state.clone() });
        }
        stripe.state = LockState::Exclusive;
        stripe.exclusive_owner = Some(owner);
        stripe.total_locks += 1;
        Ok(idx)
    }

    pub fn unlock(&mut self, key: u64, owner: u64) -> Result<usize, StripeError> {
        let idx = self.stripe_for(key);
        let stripe = &mut self.stripes[idx];
        match stripe.state {
            LockState::Free => Err(StripeError::NotHeld { stripe: idx, owner }),
            LockState::Exclusive => {
                if stripe.exclusive_owner != Some(owner) {
                    return Err(StripeError::NotHeld { stripe: idx, owner });
                }
                stripe.state = LockState::Free;
                stripe.exclusive_owner = None;
                stripe.total_unlocks += 1;
                Ok(idx)
            }
            LockState::Shared { .. } => {
                if let Some(pos) = stripe.shared_owners.iter().position(|&o| o == owner) {
                    stripe.shared_owners.remove(pos);
                    if stripe.shared_owners.is_empty() {
                        stripe.state = LockState::Free;
                    } else {
                        stripe.state = LockState::Shared { holders: stripe.shared_owners.len() as u32 };
                    }
                    stripe.total_unlocks += 1;
                    Ok(idx)
                } else {
                    Err(StripeError::NotHeld { stripe: idx, owner })
                }
            }
        }
    }

    pub fn state(&self, key: u64) -> LockState {
        let idx = self.stripe_for(key);
        self.stripes[idx].state.clone()
    }

    pub fn is_locked(&self, key: u64) -> bool {
        self.state(key) != LockState::Free
    }

    pub fn locked_stripes(&self) -> usize {
        self.stripes.iter().filter(|s| s.state != LockState::Free).count()
    }

    pub fn stats(&self) -> StripeLockStats {
        StripeLockStats {
            stripe_count: self.stripe_count,
            locked_stripes: self.locked_stripes(),
            total_locks: self.stripes.iter().map(|s| s.total_locks).sum(),
            total_unlocks: self.stripes.iter().map(|s| s.total_unlocks).sum(),
            total_conflicts: self.total_conflicts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lock() {
        let sl = StripeLock::new(16);
        assert_eq!(sl.stripe_count(), 16);
    }

    #[test]
    fn lock_exclusive() {
        let mut sl = StripeLock::new(16);
        let idx = sl.lock_exclusive(42, 1).unwrap();
        assert_eq!(sl.state(42), LockState::Exclusive);
        assert!(sl.is_locked(42));
    }

    #[test]
    fn unlock_exclusive() {
        let mut sl = StripeLock::new(16);
        sl.lock_exclusive(42, 1).unwrap();
        sl.unlock(42, 1).unwrap();
        assert_eq!(sl.state(42), LockState::Free);
    }

    #[test]
    fn lock_shared() {
        let mut sl = StripeLock::new(16);
        sl.lock_shared(42, 1).unwrap();
        sl.lock_shared(42, 2).unwrap();
        assert_eq!(sl.state(42), LockState::Shared { holders: 2 });
    }

    #[test]
    fn unlock_shared() {
        let mut sl = StripeLock::new(16);
        sl.lock_shared(42, 1).unwrap();
        sl.lock_shared(42, 2).unwrap();
        sl.unlock(42, 1).unwrap();
        assert_eq!(sl.state(42), LockState::Shared { holders: 1 });
        sl.unlock(42, 2).unwrap();
        assert_eq!(sl.state(42), LockState::Free);
    }

    #[test]
    fn exclusive_conflicts_shared() {
        let mut sl = StripeLock::new(16);
        sl.lock_shared(42, 1).unwrap();
        let err = sl.lock_exclusive(42, 2).unwrap_err();
        assert!(matches!(err, StripeError::Conflict { .. }));
    }

    #[test]
    fn shared_conflicts_exclusive() {
        let mut sl = StripeLock::new(16);
        sl.lock_exclusive(42, 1).unwrap();
        let err = sl.lock_shared(42, 2).unwrap_err();
        assert!(matches!(err, StripeError::Conflict { .. }));
    }

    #[test]
    fn unlock_not_held() {
        let mut sl = StripeLock::new(16);
        let err = sl.unlock(42, 1).unwrap_err();
        assert!(matches!(err, StripeError::NotHeld { .. }));
    }

    #[test]
    fn key_striping() {
        let mut sl = StripeLock::new(4);
        let a = sl.lock_exclusive(0, 1).unwrap();
        let err = sl.lock_exclusive(4, 2).unwrap_err();
        assert_eq!(a, 0);
        assert!(matches!(err, StripeError::Conflict { .. }));
    }

    #[test]
    fn stats() {
        let mut sl = StripeLock::new(16);
        sl.lock_exclusive(1, 1).unwrap();
        sl.lock_shared(2, 1).unwrap();
        let s = sl.stats();
        assert_eq!(s.total_locks, 2);
        assert_eq!(s.locked_stripes, 2);
    }

    #[test]
    fn error_display() {
        assert!(StripeError::OutOfRange { stripe: 20, count: 16 }.to_string().contains("20"));
    }
}

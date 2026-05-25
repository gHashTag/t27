pub struct RandPerm;

impl RandPerm {
    pub fn shuffle(data: &mut [usize], seed: u64) {
        let mut state = seed;
        let n = data.len();
        for i in (1..n).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = ((state >> 33) as usize) % (i + 1);
            data.swap(i, j);
        }
    }

    pub fn permutation(n: usize, seed: u64) -> Vec<usize> {
        let mut v: Vec<usize> = (0..n).collect();
        Self::shuffle(&mut v, seed);
        v
    }

    pub fn inverse_permutation(perm: &[usize]) -> Vec<usize> {
        let n = perm.len();
        let mut inv = vec![0; n];
        for (i, &p) in perm.iter().enumerate() { inv[p] = i; }
        inv
    }

    pub fn is_permutation(perm: &[usize]) -> bool {
        let n = perm.len();
        let mut seen = vec![false; n];
        for &p in perm {
            if p >= n || seen[p] { return false; }
            seen[p] = true;
        }
        true
    }

    pub fn compose(a: &[usize], b: &[usize]) -> Vec<usize> {
        a.iter().map(|&i| b[i]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permutation_valid() {
        let p = RandPerm::permutation(10, 42);
        assert!(RandPerm::is_permutation(&p));
    }

    #[test]
    fn deterministic() {
        let p1 = RandPerm::permutation(20, 12345);
        let p2 = RandPerm::permutation(20, 12345);
        assert_eq!(p1, p2);
    }

    #[test]
    fn different_seeds() {
        let p1 = RandPerm::permutation(20, 1);
        let p2 = RandPerm::permutation(20, 2);
        assert_ne!(p1, p2);
    }

    #[test]
    fn inverse() {
        let p = RandPerm::permutation(20, 99);
        let inv = RandPerm::inverse_permutation(&p);
        let id = RandPerm::compose(&p, &inv);
        assert_eq!(id, (0..20).collect::<Vec<_>>());
    }

    #[test]
    fn compose_identity() {
        let p = RandPerm::permutation(10, 7);
        let id: Vec<usize> = (0..10).collect();
        assert_eq!(RandPerm::compose(&id, &p), p);
    }

    #[test]
    fn empty() {
        let p = RandPerm::permutation(0, 42);
        assert!(p.is_empty());
        assert!(RandPerm::is_permutation(&p));
    }
}

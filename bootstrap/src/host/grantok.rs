use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub enum GtError {
    TokenNotFound { id: u64 },
    AlreadyRevoked { id: u64 },
    ScopeDenied { id: u64, scope: u64 },
    NotDelegable { id: u64 },
}

impl std::fmt::Display for GtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GtError::TokenNotFound { id } => write!(f, "token {id} not found"),
            GtError::AlreadyRevoked { id } => write!(f, "token {id} already revoked"),
            GtError::ScopeDenied { id, scope } => write!(f, "scope {scope} denied for token {id}"),
            GtError::NotDelegable { id } => write!(f, "token {id} not delegable"),
        }
    }
}

impl std::error::Error for GtError {}

struct Token {
    id: u64,
    scopes: BTreeSet<u64>,
    delegable: bool,
    revoked: bool,
    parent: Option<u64>,
    children: BTreeSet<u64>,
}

pub struct Grantok {
    tokens: BTreeMap<u64, Token>,
    next_id: u64,
    total_grants: u64,
    total_revocations: u64,
    total_delegations: u64,
}

impl Grantok {
    pub fn new() -> Self { Self { tokens: BTreeMap::new(), next_id: 1, total_grants: 0, total_revocations: 0, total_delegations: 0 } }

    pub fn grant(&mut self, scopes: BTreeSet<u64>, delegable: bool) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tokens.insert(id, Token { id, scopes, delegable, revoked: false, parent: None, children: BTreeSet::new() });
        self.total_grants += 1;
        id
    }

    pub fn delegate(&mut self, parent_id: u64, subset: BTreeSet<u64>) -> Result<u64, GtError> {
        let parent = self.tokens.get(&parent_id).ok_or(GtError::TokenNotFound { id: parent_id })?;
        if parent.revoked { return Err(GtError::AlreadyRevoked { id: parent_id }); }
        if !parent.delegable { return Err(GtError::NotDelegable { id: parent_id }); }
        for &s in &subset {
            if !parent.scopes.contains(&s) { return Err(GtError::ScopeDenied { id: parent_id, scope: s }); }
        }
        drop(parent);
        let id = self.next_id;
        self.next_id += 1;
        self.tokens.insert(id, Token { id, scopes: subset, delegable: true, revoked: false, parent: Some(parent_id), children: BTreeSet::new() });
        self.tokens.get_mut(&parent_id).unwrap().children.insert(id);
        self.total_delegations += 1;
        Ok(id)
    }

    pub fn revoke(&mut self, id: u64) -> Result<u64, GtError> {
        let tok = self.tokens.get_mut(&id).ok_or(GtError::TokenNotFound { id })?;
        if tok.revoked { return Err(GtError::AlreadyRevoked { id }); }
        tok.revoked = true;
        self.total_revocations += 1;
        let child_ids: Vec<u64> = tok.children.iter().copied().collect();
        drop(tok);
        let mut total = 1u64;
        for cid in child_ids { total += self.revoke(cid).unwrap_or(0); }
        Ok(total)
    }

    pub fn is_valid(&self, id: u64, scope: u64) -> bool {
        let tok = match self.tokens.get(&id) {
            Some(t) => t,
            None => return false,
        };
        if tok.revoked { return false; }
        if !tok.scopes.contains(&scope) { return false; }
        if let Some(pid) = tok.parent { self.is_valid(pid, scope) } else { true }
    }

    pub fn scopes(&self, id: u64) -> Option<&BTreeSet<u64>> { self.tokens.get(&id).map(|t| &t.scopes) }
    pub fn is_revoked(&self, id: u64) -> Option<bool> { self.tokens.get(&id).map(|t| t.revoked) }
    pub fn token_count(&self) -> usize { self.tokens.len() }
    pub fn total_grants(&self) -> u64 { self.total_grants }
    pub fn total_revocations(&self) -> u64 { self.total_revocations }
    pub fn total_delegations(&self) -> u64 { self.total_delegations }
}

impl Default for Grantok {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scopes(ids: &[u64]) -> BTreeSet<u64> { ids.iter().copied().collect() }

    #[test]
    fn new_system() { assert_eq!(Grantok::new().token_count(), 0); }

    #[test]
    fn grant_check() {
        let mut g = Grantok::new();
        let t = g.grant(scopes(&[1, 2, 3]), false);
        assert!(g.is_valid(t, 1));
        assert!(!g.is_valid(t, 4));
    }

    #[test]
    fn delegate_subset() {
        let mut g = Grantok::new();
        let p = g.grant(scopes(&[1, 2, 3]), true);
        let c = g.delegate(p, scopes(&[1, 2])).unwrap();
        assert!(g.is_valid(c, 1));
        assert!(!g.is_valid(c, 3));
    }

    #[test]
    fn delegate_scope_denied() {
        let mut g = Grantok::new();
        let p = g.grant(scopes(&[1, 2]), true);
        let err = g.delegate(p, scopes(&[3])).unwrap_err();
        assert!(matches!(err, GtError::ScopeDenied { .. }));
    }

    #[test]
    fn not_delegable() {
        let mut g = Grantok::new();
        let p = g.grant(scopes(&[1]), false);
        let err = g.delegate(p, scopes(&[1])).unwrap_err();
        assert!(matches!(err, GtError::NotDelegable { .. }));
    }

    #[test]
    fn revoke_cascades() {
        let mut g = Grantok::new();
        let p = g.grant(scopes(&[1]), true);
        let c = g.delegate(p, scopes(&[1])).unwrap();
        let count = g.revoke(p).unwrap();
        assert_eq!(count, 2);
        assert!(g.is_revoked(p).unwrap());
        assert!(g.is_revoked(c).unwrap());
        assert!(!g.is_valid(c, 1));
    }

    #[test]
    fn revoke_already() {
        let mut g = Grantok::new();
        let t = g.grant(scopes(&[1]), false);
        g.revoke(t).unwrap();
        let err = g.revoke(t).unwrap_err();
        assert!(matches!(err, GtError::AlreadyRevoked { .. }));
    }

    #[test]
    fn revoke_invalidates_delegate() {
        let mut g = Grantok::new();
        let p = g.grant(scopes(&[1, 2]), true);
        let c = g.delegate(p, scopes(&[1])).unwrap();
        g.revoke(p).unwrap();
        assert!(!g.is_valid(c, 1));
    }

    #[test]
    fn stats() {
        let mut g = Grantok::new();
        let p = g.grant(scopes(&[1]), true);
        g.delegate(p, scopes(&[1])).unwrap();
        g.revoke(p).unwrap();
        assert_eq!(g.total_grants(), 1);
        assert_eq!(g.total_delegations(), 1);
        assert_eq!(g.total_revocations(), 2);
    }

    #[test]
    fn error_display() { assert!(GtError::TokenNotFound { id: 1 }.to_string().contains("1")); }
}

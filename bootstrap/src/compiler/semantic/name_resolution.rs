//! Name resolution
//!
//! Resolves variable and function names to their declarations.
//! Part of modular compiler architecture (Ring-018).

use anyhow::Result;
use super::ast::{Expr, Decl};

/// Name resolution context
pub struct NameContext {
    pub scopes: Vec<Scope>,
}

impl NameContext {
    pub fn new() -> Self {
        NameContext {
            scopes: vec![Scope::new()],
        }
    }
}

/// Variable scope
pub struct Scope {
    pub bindings: Vec<(String, Option<Decl>)>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            bindings: Vec::new(),
        }
    }

    /// Add a binding to current scope
    pub fn bind(&mut self, name: String, decl: Option<Decl>) {
        self.bindings.push((name, decl));
    }

    /// Look up a binding
    pub fn lookup(&self, name: &str) -> Option<&Decl> {
        for (n, decl) in self.bindings.iter().rev() {
            if n == name {
                return decl.as_ref();
            }
        }
        None
    }
}

/// Resolve a name reference (stub)
///
/// TODO: Implement full name resolution in Ring-018/019.
pub fn resolve(_name: &str, _ctx: &mut NameContext) -> Result<Option<Decl>> {
    // Placeholder: Always returns None
    // This will be replaced with full name resolution in future rings

    Ok(None)
}

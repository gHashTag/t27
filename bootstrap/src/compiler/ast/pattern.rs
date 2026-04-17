//! AST Pattern nodes
//!
//! Defines pattern matching patterns (for match, let, function params).
//! Part of modular compiler architecture (Ring-018).

use serde::{Deserialize, Serialize};
use super::expr::{Expr, Literal, TritValue};

/// Pattern for match/let/function expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard pattern: _
    Wildcard,

    /// Literal pattern
    Literal(Literal),

    /// Variable binding: name
    Var(String),

    /// Tuple pattern: (pat1, pat2, ...)
    Tuple(Vec<Pattern>),

    /// Struct pattern: Type { field: pat, ... }
    Struct {
        type_name: String,
        fields: Vec<(String, Pattern)>,
    },

    /// Or pattern: pat1 | pat2
    Or(Box<Pattern>, Box<Pattern>),

    /// Range pattern (optional for K3)
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },

    /// As pattern: pattern as name
    As {
        pattern: Box<Pattern>,
        name: String,
    },
}

impl Pattern {
    /// Create a literal pattern
    pub fn lit(lit: impl Into<Literal>) -> Self {
        Pattern::Literal(lit.into())
    }

    /// Create a variable pattern
    pub fn var(name: impl Into<String>) -> Self {
        Pattern::Var(name.into())
    }
}

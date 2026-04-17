//! Type checker
//!
//! Performs type checking on AST expressions and declarations.
//! Part of modular compiler architecture (Ring-018).

use anyhow::{bail, Result};
use super::ast::{Expr, Type, Decl};

/// Type checking context
pub struct TypeContext {
    pub type_env: Vec<(String, Type)>,
    pub errors: Vec<TypeError>,
}

impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            type_env: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Look up a type in the environment
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.type_env.iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t)
    }
}

/// Type check errors
#[derive(Debug, Clone)]
pub enum TypeError {
    UndefinedVariable(String),
    TypeMismatch { expected: Type, found: Type },
    NotCallable(String),
    ArgCountMismatch { expected: usize, found: usize },
    FieldNotFound { typ: Type, field: String },
}

/// Type check an expression (stub)
///
/// TODO: Implement full type checking in Ring-018/019.
pub fn typecheck(_expr: &Expr, _ctx: &mut TypeContext) -> Result<Type> {
    // Placeholder: Returns I32 for all expressions
    // This will be replaced with full type checking in future rings

    Ok(Type::Primitive(super::ast::PrimitiveType::I32))
}

/// Type check a declaration (stub)
pub fn typecheck_decl(_decl: &Decl, _ctx: &mut TypeContext) -> Result<()> {
    // Placeholder
    Ok(())
}

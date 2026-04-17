//! AST Type system
//!
//! Defines type annotations and type expressions.
//! Part of modular compiler architecture (Ring-018).

use serde::{Deserialize, Serialize};

/// Type in Trinity language
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive type
    Primitive(PrimitiveType),

    /// Tuple type: (T1, T2, ...)
    Tuple(Vec<Type>),

    /// List type: [T]
    List(Box<Type>),

    /// Function type: (T1, T2) -> T
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    /// Struct type: Name { fields... }
    Struct {
        name: String,
        fields: Vec<(String, Type)>,
    },

    /// Type variable or reference
    Var(String),

    /// Generic type with variance
    Generic {
        name: String,
        args: Vec<Type>,
        base: Box<Type>,
    },

    /// Type application with args
    Apply {
        func: Box<Type>,
        args: Vec<Type>,
    },
}

impl Type {
    /// Create a primitive type
    pub fn prim(prim: PrimitiveType) -> Self {
        Type::Primitive(prim)
    }

    /// Create a tuple type
    pub fn tuple(types: Vec<Type>) -> Self {
        Type::Tuple(types)
    }

    /// Create a variable reference
    pub fn var(name: impl Into<String>) -> Self {
        Type::Var(name.into())
    }

    /// Create a function type
    pub fn func(params: Vec<Type>, ret: Type) -> Self {
        Type::Function {
            params,
            return_type: Box::new(ret),
        }
    }
}

//! AST Declaration nodes
//!
//! Defines top-level declarations: modules, functions, constants.
//! Part of modular compiler architecture (Ring-018).

use serde::{Deserialize, Serialize};

/// Declaration in Trinity language
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Decl {
    /// Module declaration: module name { ...exports }
    Module {
        name: String,
        exports: Vec<Export>,
    },

    /// Function declaration: func name(params): type [= expr]
    Function {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Box<Expr>,
    },

    /// Constant declaration: const name: type = value
    Const {
        name: String,
        type_annotation: Option<Type>,
        value: Expr,
    },

    /// Type alias: type NewName = OldType
    TypeAlias {
        name: String,
        target: Type,
    },

    /// Import declaration: from spec ...
    Import {
        path: String,
        items: Vec<ImportItem>,
    },
}

/// Export from module
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Export {
    pub name: String,
    pub spec_name: Option<String>,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub default_value: Option<Expr>,
}

/// Import item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportItem {
    Name(String),
    All,
}

//! Declaration parser
//!
//! Parses top-level declarations: modules, functions, constants.
//! Part of modular compiler architecture (Ring-018).

use anyhow::{bail, Result};
use super::ast::{Decl, Export, Param, Type, ImportItem};

/// Parse a module declaration (stub)
pub fn parse_module(_name: &str, _exports: &[Export]) -> Result<Decl> {
    // Placeholder: Returns a stub module declaration
    // This will be replaced with full parsing in future rings

    Ok(Decl::Module {
        name: "stub".into(),
        exports: Vec::new(),
    })
}

/// Parse a function declaration (stub)
pub fn parse_function(
    _name: &str,
    _params: &[Param],
    _return_type: Option<&Type>,
    _body: &str,
) -> Result<Decl> {
    // Placeholder: Returns a stub function declaration
    // This will be replaced with full parsing in future rings

    Ok(Decl::Function {
        name: "stub".into(),
        params: Vec::new(),
        return_type: None,
        body: Box::new(Expr::Literal(Literal::Integer(0))),
    })
}

/// Parse a constant declaration (stub)
pub fn parse_const(_name: &str, _type: Option<&Type>, _value: &str) -> Result<Decl> {
    // Placeholder: Returns a stub constant declaration
    // This will be replaced with full parsing in future rings

    Ok(Decl::Const {
        name: "stub".into(),
        type_annotation: None,
        value: Expr::Literal(Literal::Integer(0)),
    })
}

/// Parse an import declaration (stub)
pub fn parse_import(_path: &str, _items: &[ImportItem]) -> Result<Decl> {
    // Placeholder: Returns a stub import declaration
    // This will be replaced with full parsing in future rings

    Ok(Decl::Import {
        path: "stub".into(),
        items: Vec::new(),
    })
}

//! Type parser
//!
//! Parses type annotations and type expressions.
//! Part of modular compiler architecture (Ring-018).

use anyhow::Result;
use super::ast::Type;

/// Parse a primitive type (stub)
pub fn parse_primitive(_input: &str) -> Result<Type> {
    // Placeholder: Returns a stub primitive type
    // This will be replaced with full parsing in future rings

    Ok(Type::Primitive(super::ast::PrimitiveType::I32))
}

/// Parse a type name (stub)
pub fn parse_type_name(_input: &str) -> Result<String> {
    // Placeholder
    Ok("stub".to_string())
}

/// Parse a function type (stub)
pub fn parse_function_type(
    _params: &[Type],
    _return_type: &Type,
) -> Result<Type> {
    // Placeholder: Returns a stub function type
    // This will be replaced with full parsing in future rings

    Ok(Type::Function {
        params: Vec::new(),
        return_type: Box::new(Type::Primitive(super::ast::PrimitiveType::I32)),
    })
}

/// Parse a tuple type (stub)
pub fn parse_tuple(_types: &[Type]) -> Result<Type> {
    // Placeholder: Returns a stub tuple type
    // This will be replaced with full parsing in future rings

    Ok(Type::Tuple(Vec::new()))
}

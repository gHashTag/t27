// bootstrap/src/compiler/parser/expr.rs
// Expression parsing submodules
//
// This module contains expression parsing logic.
// Provides parse_expr() and related functions that work with ExprNode types.

use crate::compiler::token::{Token, TokenKind, Span};
use crate::compiler::ast::{ExprNode, LiteralExpr, IdentifierExpr, BinaryExpr, UnaryExpr, CallExpr, FieldAccessExpr, IndexExpr};
use crate::compiler::parser::ParseError;

// ============================================================================
// Expression Parsing (precedence levels)
// ============================================================================

/// Parse a primary expression (literal, identifier, or parenthesized expr)
/// Returns ParseResult<ExprNode, ParseError>
pub fn parse_expr(lexer: &mut Token) -> ParseResult<ExprNode, ParseError> {
    todo!()
}

/// Parse an expression (handles operators with precedence)
/// Returns ParseResult<ExprNode, ParseError>
pub fn parse_expr_internal(lexer: &mut Token) -> ParseResult<ExprNode, ParseError> {
    todo!()
}

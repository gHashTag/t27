//! Tuple lowering for t27b.
//!
//! This module handles lowering of tuple constructs from the AST to the IR,
//! following the same pattern as the reference compiler t27c.

use super::*;
use crate::ir::{self, *};

/// Lower tuple types and expressions according to the reference compiler's rules.
pub struct TupleLowering;

impl TupleLowering {
    /// Check if a type is a tuple type in a function return position.
    pub fn is_return_tuple_type(n: &Node) -> bool {
        matches!(n.kind, NodeKind::TypeTuple) && n.name.starts_with("fn")
    }

    /// Check if a type is a tuple type that should be rejected.
    pub fn is_rejected_tuple_type(n: &Node) -> bool {
        matches!(n.kind, NodeKind::TypeTuple) && !n.name.starts_with("fn")
    }

    /// Check if an expression is a tuple literal that should be rejected.
    pub fn is_rejected_tuple_expr(n: &Node) -> bool {
        matches!(n.kind, NodeKind::ExprTuple) && n.name.is_empty()
    }

    /// Check if an assignment is a tuple assignment that should be rejected.
    pub fn is_rejected_tuple_assign(n: &Node) -> bool {
        matches!(n.kind, NodeKind::StmtAssign) && n.children.len() >= 2 && {
            let lhs = &n.children[0];
            matches!(lhs.kind, NodeKind::ExprTuple)
        }
    }

    /// Check if an index expression is a tuple index that should be rejected.
    pub fn is_rejected_tuple_index(n: &Node) -> bool {
        matches!(n.kind, NodeKind::ExprIndex) && {
            let base = &n.children[0];
            matches!(base.kind, NodeKind::ExprTuple)
        }
    }

    /// Lower a tuple type in function return position to an array type.
    pub fn lower_return_tuple_type(n: &Node, ctx: &mut LowerCtx) -> Option<ir::Type> {
        if !Self::is_return_tuple_type(n) {
            return None;
        }

        // Extract the tuple elements and create an array type
        let mut element_types = Vec::new();
        for child in &n.children {
            if child.kind == NodeKind::TypeIdent {
                element_types.push(child.name.clone());
            }
        }

        if element_types.len() != 2 {
            return None; // Only handle 2-element tuples for now
        }

        Some(ir::Type::Array {
            element: Box::new(element_types[0].clone()),
            size: element_types[1].clone(),
        })
    }

    /// Lower a tuple literal in a valid position to an array literal.
    pub fn lower_tuple_literal(n: &Node, ctx: &mut LowerCtx) -> Option<ir::Expr> {
        if !matches!(n.kind, NodeKind::ExprTuple) {
            return None;
        }

        // Convert tuple literal to array literal
        let mut elements = Vec::new();
        for child in &n.children {
            if child.kind == NodeKind::ExprIdent || child.kind == NodeKind::ExprInt {
                elements.push(child.name.clone());
            }
        }

        if elements.len() != 2 {
            return None; // Only handle 2-element tuples for now
        }

        Some(ir::Expr::ArrayLiteral {
            elements,
            extra_type: String::new(),
            extra_size: String::new(),
        })
    }

    /// Lower a tuple field access (t.0) to an array index.
    pub fn lower_tuple_field_access(n: &Node, ctx: &mut LowerCtx) -> Option<ir::Expr> {
        if !matches!(n.kind, NodeKind::ExprFieldAccess) {
            return None;
        }

        // Check if this is a tuple field access like t.0
        if n.name == "0" || n.name == "1" {
            let base = &n.children[0];
            // Create an array index expression
            Some(ir::Expr::Index {
                base: Box::new(ir::Expr::Ident(base.name.clone())),
                index: Box::new(ir::Expr::Int(n.name.clone())),
                extra_type: String::new(),
            })
        } else {
            None
        }
    }
}

/// Hook function to check for rejected tuple types in parameters, locals, and fields.
pub fn check_rejected_tuple_types(n: &Node, ctx: &mut LowerCtx, rejects: &mut Vec<Reject>) {
    if TupleLowering::is_rejected_tuple_type(n) {
        rejects.push(Reject {
            line: n.line,
            kind: "type (tuple)",
            message: format!("tuple type in parameter, local, or field: {}", n.name),
        });
    }
}

/// Hook function to check for rejected tuple expressions.
pub fn check_rejected_tuple_exprs(n: &Node, ctx: &mut LowerCtx, rejects: &mut Vec<Reject>) {
    if TupleLowering::is_rejected_tuple_expr(n) {
        rejects.push(Reject {
            line: n.line,
            kind: "ExprTuple",
            message: "untyped tuple local".to_string(),
        });
    }
}

/// Hook function to check for rejected tuple assignments.
pub fn check_rejected_tuple_assigns(n: &Node, ctx: &mut LowerCtx, rejects: &mut Vec<Reject>) {
    if TupleLowering::is_rejected_tuple_assign(n) {
        rejects.push(Reject {
            line: n.line,
            kind: "StmtAssign(tuple)",
            message: "tuple assignment in function body".to_string(),
        });
    }
}

/// Hook function to check for rejected tuple indexes.
pub fn check_rejected_tuple_indexes(n: &Node, ctx: &mut LowerCtx, rejects: &mut Vec<Reject>) {
    if TupleLowering::is_rejected_tuple_index(n) {
        rejects.push(Reject {
            line: n.line,
            kind: "ExprIndex(tuple)",
            message: "runtime or out-of-bounds tuple index".to_string(),
        });
    }
}

/// Hook function to lower valid tuple types in return positions.
pub fn lower_return_tuple_types(n: &Node, ctx: &mut LowerCtx) -> Option<ir::Type> {
    TupleLowering::lower_return_tuple_type(n, ctx)
}

/// Hook function to lower valid tuple literals.
pub fn lower_tuple_literals(n: &Node, ctx: &mut LowerCtx) -> Option<ir::Expr> {
    TupleLowering::lower_tuple_literal(n, ctx)
}

/// Hook function to lower tuple field accesses.
pub fn lower_tuple_field_accesses(n: &Node, ctx: &mut LowerCtx) -> Option<ir::Expr> {
    TupleLowering::lower_tuple_field_access(n, ctx)
}
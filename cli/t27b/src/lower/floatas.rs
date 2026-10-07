//! `x as T` with a float operand, as t27c's Zig backend prints it.
//!
//! gen-zig picks the cast builtin from the operand's spelling, not its type
//! (`is_float_expr`): a literal with a decimal point, a name in
//! `float_names`, or a binary expression with such a side is a float, and
//! anything else is taken for an integer. So `y as i64` is
//! `@as(i64, @intFromFloat(y))` when `y` is a parameter or local declared
//! f64, and `@as(i64, @intCast(y))` -- which Zig refuses -- when `y` is, say,
//! a call returning f64. Only the first shape is lowered here; a float the
//! backend would not recognize stays refused as `ExprCast(f64)`.
//!
//! `float_names`, as gen-zig keeps it:
//! - every struct field declared f16 / f32 / f64 / float / double, for the
//!   whole file;
//! - inside a fn, that fn's parameters and its locals so declared, at any
//!   depth of its body (not scoped). Test, invariant and bench bodies add
//!   nothing.
//!
//! A fn's float locals are removed again when the fn ends, even when the
//! name was a struct field's, so a field sharing a name with some fn's float
//! local is in the set only before that fn. Such a name is never taken for a
//! float here.

use super::{float::float_cast, Lower, R, Val};
use crate::compiler::{Node, NodeKind};
use crate::ir::Ty;
use std::collections::HashSet;

const FLOAT_TYPES: [&str; 5] = ["f16", "f32", "f64", "float", "double"];

fn float_locals(stmts: &[Node], out: &mut HashSet<String>) {
    for s in stmts {
        if s.kind == NodeKind::StmtLocal && !s.name.is_empty() && FLOAT_TYPES.contains(&s.extra_type.trim()) {
            out.insert(s.name.clone());
        }
        float_locals(&s.children, out);
    }
}

impl<'a> Lower<'a> {
    /// The struct-field half of `float_names`, less any name some fn also
    /// declares as a float local.
    pub(super) fn float_field_names(&mut self, items: &[&Node]) {
        let mut locals = HashSet::new();
        for item in items {
            if item.kind == NodeKind::FnDecl {
                float_locals(&item.children, &mut locals);
            }
            if item.kind == NodeKind::StructDecl {
                for f in &item.children {
                    if !f.name.is_empty() && FLOAT_TYPES.contains(&f.extra_type.trim()) {
                        self.float_fields.insert(f.name.clone());
                    }
                }
            }
        }
        self.float_fields.retain(|n| !locals.contains(n));
    }

    /// The fn half of `float_names` while fn `n`'s body is lowered.
    pub(super) fn enter_float_names(&mut self, n: &Node) {
        for (p, t) in &n.params {
            if FLOAT_TYPES.contains(&t.trim()) {
                self.float_locals.insert(p.trim().to_string());
            }
        }
        let mut locals = HashSet::new();
        float_locals(&n.children, &mut locals);
        self.float_locals.extend(locals);
    }

    /// gen-zig's `is_float_expr`.
    fn spelled_float(&self, n: &Node) -> bool {
        match n.kind {
            NodeKind::ExprLiteral => n.extra_kind != "string" && n.value.contains('.'),
            NodeKind::ExprIdentifier | NodeKind::ExprFieldAccess => {
                let last = if n.kind == NodeKind::ExprIdentifier {
                    n.name.rsplit(['.', ':']).next().unwrap_or("")
                } else {
                    n.name.as_str()
                };
                !last.is_empty() && (self.float_locals.contains(last) || self.float_fields.contains(last))
            }
            NodeKind::ExprBinary => n.children.iter().any(|c| self.spelled_float(c)),
            _ => false,
        }
    }

    /// `operand as to` where `v`, the lowered operand, is a typed f32 / f64
    /// that gen-zig also spells as a float: `@intFromFloat` to an integer,
    /// `@floatCast` to a float. None for every other cast.
    pub(super) fn float_as(&mut self, operand: &Node, v: &Val, to: Ty) -> R<Option<Val>> {
        let Val::E(e) = v else { return Ok(None) };
        if !e.ty.is_float() || !self.spelled_float(operand) {
            return Ok(None);
        }
        if to.is_float() {
            return Ok(Some(Val::E(float_cast(e.clone(), to))));
        }
        if to.is_int() {
            let what = format!("ExprCast({})", e.ty.name());
            return self.from_float(e.clone(), to, &what).map(Some);
        }
        Ok(None)
    }
}

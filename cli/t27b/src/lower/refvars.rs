//! Array and struct names the reference makes a `var` (spec:
//! `specs/tri/t27b/conformance/value_params.t27`).
//!
//! t27c's Zig backend collects, per fn and per test body, the names a
//! statement assigns: the name itself, an element `name[i]` or a field
//! `name.f`, at the top of the body or inside if / while / for
//! (`collect_mutable_names` in `bootstrap/src/compiler.rs`). Each such name
//! is a `var`:
//!   - a parameter is renamed `<name>_arg` and the body opens with
//!     `var <name> = <name>_arg;`, so the callee writes its own copy;
//!   - a `let` / `const` local is declared `var`.
//! t27b passes an array or struct argument as a pointer to the caller's
//! memory, so such a parameter is copied into a slot of the callee at entry;
//! such a local is simply writable. Scalars are left as they were.
//!
//! A name written only deeper (`name.f[i] = ...`) is not collected, stays a
//! constant in the reference, which refuses the write; it is refused here
//! too. Invariant bodies (t27c does not collect for them) and bench bodies
//! are left alone.

use super::*;

impl<'a> Lower<'a> {
    /// Called from `begin_body`.
    pub(super) fn collect_ref_vars(&mut self, body: &[Node]) {
        self.ref_vars.clear();
        collect(body, &mut self.ref_vars);
    }

    /// Whether a value of type `t` named `name` is a writable `var` in the
    /// reference: an array or struct the body assigns.
    pub(super) fn ref_var_agg(&self, t: &LTy, name: &str) -> bool {
        matches!(t, LTy::Arr(..) | LTy::Struct(_)) && self.ref_vars.contains(name)
    }

    /// The place parameter `pname` is bound to: `src` (the caller's memory),
    /// or a writable copy of it when the reference makes it a `var`.
    pub(super) fn param_place(&mut self, pname: &str, src: Place, out: &mut Vec<Stmt>) -> R<Place> {
        if pname == "self" || !self.ref_var_agg(&src.ty, pname) {
            return Ok(src);
        }
        let k = self.new_slot(&src.ty)?;
        let dst = Place { addr: slot_expr(k), off: 0, ty: src.ty.clone(), mutable: true, temp: None };
        self.copy(&dst, src, out)?;
        Ok(dst)
    }
}

/// t27c's `collect_mutable_names`.
fn collect(stmts: &[Node], set: &mut HashSet<String>) {
    for s in stmts {
        collect_one(s, set);
    }
}

fn collect_one(s: &Node, set: &mut HashSet<String>) {
    match s.kind {
        NodeKind::StmtAssign => {
            let Some(lhs) = s.children.first() else { return };
            let base = match lhs.kind {
                NodeKind::ExprIdentifier => Some(lhs),
                NodeKind::ExprIndex | NodeKind::ExprFieldAccess => lhs.children.first(),
                _ => None,
            };
            if let Some(b) = base.filter(|b| b.kind == NodeKind::ExprIdentifier) {
                set.insert(b.name.clone());
            }
        }
        NodeKind::StmtIf | NodeKind::StmtWhile | NodeKind::StmtFor | NodeKind::StmtForRange => {
            for c in &s.children {
                if c.kind == NodeKind::Module {
                    collect(&c.children, set);
                } else {
                    collect_one(c, set);
                }
            }
        }
        _ => {}
    }
}

//! Signatures of fns the reference never analyzes.
//!
//! Zig analyzes a fn lazily: one that no test, invariant or constant reaches
//! (`analyzed_fns`) is parsed but never type-checked, so its return type is
//! never resolved either. t27c prints a port's `-> undefined` stub as
//! `fn f() undefined { undefined; }`; Zig would refuse that type ("expected
//! type 'type', found '@TypeOf(undefined)'") only once something calls `f`.

use super::{Lower, Reject, R};
use crate::compiler::{Node, NodeKind};
use std::collections::HashSet;

impl<'a> Lower<'a> {
    /// `-> undefined` on a fn nothing analyzed reaches: lowered as a fn with
    /// no result, whose `undefined;` body is the stub trap no test reaches.
    /// On a reached fn the type stays refused (`type undefined`), as the
    /// reference's Zig does not compile it.
    pub(super) fn unanalyzed_undefined_ret(&self, n: &Node) -> bool {
        n.extra_return_type.trim() == "undefined" && !self.analyzed.contains(&n.name)
    }

    /// `signature(n)` was just refused, and `nerr` errors were recorded
    /// before it. Zig resolves a fn's parameter and result types only when
    /// the fn is analyzed, so on a fn nothing analyzed reaches, a struct
    /// whose layout fails (`Node` holding `[N]Node`, a field of a type t27b
    /// does not lower) is never laid out and refuses nothing. Such a fn has
    /// no body here, and what its signature reported is withdrawn: true then.
    ///
    /// An undeclared name is still refused: Zig's AstGen resolves every name
    /// in the file, analyzed or not.
    pub(super) fn unresolved_sig(&mut self, n: &Node, nerr: usize) -> bool {
        if self.analyzed.contains(&n.name) || !self.sig_layout_only || self.errors.len() == nerr {
            return false;
        }
        if self.errors[nerr..].iter().any(|e| e.construct.contains("undeclared")) {
            return false;
        }
        if let Some(u) = self.undeclared_in(n) {
            let _: R<()> = self.reject("ExprIdentifier(undeclared)", format!("`{}` in `{}`, a body Zig's AstGen still resolves", u, n.name));
            return false;
        }
        let dropped: Vec<Reject> = self.errors.drain(nerr..).collect();
        // A struct whose failure was withdrawn reports it again where an
        // analyzed use lays it out (recovery mode names a cached failure
        // only once).
        for sd in self.structs.iter_mut() {
            let withdrawn = sd
                .fail
                .as_ref()
                .is_some_and(|f| dropped.iter().any(|d| d.construct == f.construct && d.detail == f.detail));
            if withdrawn {
                sd.fail = None;
            }
        }
        self.unresolved.insert(n.name.clone());
        true
    }

    /// A refused signature type Zig resolves only when the fn is analyzed, as `layout` failures are
    /// (specs/tri/t27b/lazy_sig_plan.t27): `anytype` on a parameter, `[*]T` with T a type Zig reads here.
    pub(super) fn lazy_sig(&self, t: &str, param: bool) -> bool {
        let t = t.trim();
        let k = super::ls::many_item_elem(t.as_bytes());
        super::ls::plan(t.as_bytes(), param, k > 0 && self.zig_spelled(&t[k..])) == super::ls::UNRESOLVED
    }

    /// A name the body of `f` uses that neither `f` nor the file declares and Zig does not know (`ls::zig_knows`).
    fn undeclared_in(&self, f: &Node) -> Option<String> {
        fn walk(ns: &[Node], local: &mut HashSet<String>, used: &mut Vec<String>) {
            for n in ns {
                local.extend(n.params.iter().map(|(p, _)| p.trim().to_string()));
                match n.kind {
                    NodeKind::ExprIdentifier | NodeKind::ExprCall | NodeKind::ExprStructLit => used.push(n.name.split(['.', ':']).next().unwrap_or("").trim().to_string()),
                    NodeKind::StmtLocal | NodeKind::StmtForRange => { local.insert(n.name.trim().to_string()); }
                    _ => {}
                }
                walk(&n.children, local, used);
            }
        }
        let (mut local, mut used) = (HashSet::new(), Vec::new());
        walk(std::slice::from_ref(f), &mut local, &mut used);
        used.into_iter().find(|u| !u.is_empty() && !local.contains(u) && !self.fns.contains_key(u) && !self.type_decls.contains_key(u) && !super::ls::zig_knows(u.as_bytes()))
    }

    /// A call to a fn `unresolved_sig` withdrew. Only a fn nothing analyzed
    /// reaches can make one, but the callee's parameter types are unknown
    /// here, so the call is refused rather than lowered.
    pub(super) fn unresolved_call<T>(&mut self, c: &Node) -> R<T> {
        self.reject(
            "ExprCall(unresolved fn)",
            format!(
                "call to `{}`, a fn no test reaches whose signature names a struct t27b cannot lay out",
                c.name
            ),
        )
    }
}

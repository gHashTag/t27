//! Signatures of fns the reference never analyzes.
//!
//! Zig analyzes a fn lazily: one that no test, invariant or constant reaches
//! (`analyzed_fns`) is parsed but never type-checked, so its return type is
//! never resolved either. t27c prints a port's `-> undefined` stub as
//! `fn f() undefined { undefined; }`; Zig would refuse that type ("expected
//! type 'type', found '@TypeOf(undefined)'") only once something calls `f`.

use super::{Lower, Reject, R};
use crate::compiler::Node;

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

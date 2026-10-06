//! Signatures of fns the reference never analyzes.
//!
//! Zig analyzes a fn lazily: one that no test, invariant or constant reaches
//! (`analyzed_fns`) is parsed but never type-checked, so its return type is
//! never resolved either. t27c prints a port's `-> undefined` stub as
//! `fn f() undefined { undefined; }`; Zig would refuse that type ("expected
//! type 'type', found '@TypeOf(undefined)'") only once something calls `f`.

use super::Lower;
use crate::compiler::Node;

impl<'a> Lower<'a> {
    /// `-> undefined` on a fn nothing analyzed reaches: lowered as a fn with
    /// no result, whose `undefined;` body is the stub trap no test reaches.
    /// On a reached fn the type stays refused (`type undefined`), as the
    /// reference's Zig does not compile it.
    pub(super) fn unanalyzed_undefined_ret(&self, n: &Node) -> bool {
        n.extra_return_type.trim() == "undefined" && !self.analyzed.contains(&n.name)
    }
}

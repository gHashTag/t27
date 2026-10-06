//! `x.len()` and `len(x)`: the length FIELD, as t27c's Zig backend prints
//! both (W570; Zig has no `len` method on a slice or an array):
//!
//! - a call with no arguments whose name is a dotted path ending in `.len`
//!   (`a.b.len()`) prints the path, `a.b.len`;
//! - a method call `len` the parser kept with its receiver as the one child
//!   prints `<receiver>.len`;
//! - a free `len(x)`, when no fn named `len` is declared, prints `x.len`.
//!
//! Each lowers as that field access, which decides what has a length.

use super::{Lower, Val, R};
use crate::compiler::{Node, NodeKind};

impl<'a> Lower<'a> {
    /// The `.len` field access a call to `len` stands for; None for any
    /// other call.
    pub(super) fn len_call(&mut self, c: &Node) -> R<Option<Val>> {
        let Some(recv) = self.len_receiver(c) else { return Ok(None) };
        let field = Node {
            kind: NodeKind::ExprFieldAccess,
            name: "len".into(),
            line: c.line,
            children: vec![recv],
            ..Node::default()
        };
        self.expr(&field).map(Some)
    }

    fn len_receiver(&self, c: &Node) -> Option<Node> {
        if c.extra_kind == "method" && !c.children.is_empty() {
            return (c.name == "len" && c.children.len() == 1).then(|| c.children[0].clone());
        }
        if c.children.is_empty() && c.name.len() > 4 {
            let path = c.name.strip_suffix(".len")?;
            let mut segs = path.split('.');
            let first = segs.next()?;
            let ok = |s: &str| !s.is_empty() && s.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
            if !ok(first) || first.starts_with(|ch: char| ch.is_ascii_digit()) {
                return None;
            }
            let mut n = Node { kind: NodeKind::ExprIdentifier, name: first.into(), line: c.line, ..Node::default() };
            for s in segs {
                if !ok(s) {
                    return None;
                }
                n = Node { kind: NodeKind::ExprFieldAccess, name: s.into(), line: c.line, children: vec![n], ..Node::default() };
            }
            return Some(n);
        }
        if c.name == "len" && c.children.len() == 1 && !self.sigs.contains_key("len") && !self.poison_names.contains("len") {
            return Some(c.children[0].clone());
        }
        None
    }
}

//! Array literals the reference takes by address or returns as a slice
//! (specs: `specs/tri/t27b/conformance/array_literal.t27` and
//! `array_literal_value.t27`).
//!
//! `const xs = &[_]T{ a, b }` is `const xs = &.{ a, b }` in t27c's Zig
//! backend: a pointer to a tuple, which coerces to `[]const E` where a
//! callee declares one -- E is the callee's element type, T is dropped.
//! Taken only when every use of `xs` is such an argument (one E for all) or
//! `xs.len`: then it is an `[N]E` built once at the declaration, exactly what
//! the existing slice-local path (`slice_local`) builds. Indexing it is
//! refused (its elements are comptime values in the reference).
//!
//! `const x = [N]T{ a, b }` (or `[_]T{...}`) is `const x = .{ a, b }`: a
//! tuple, N and T dropped, so its length is its element count. When every
//! use is `&x` as such an argument (one E for all) or `x.len`, it takes the
//! same slice-local path, and `&x` passes the slice of it (`arg_as`).
//!
//! An untyped module constant bound to a list is `.{ ... }`, a tuple; one
//! nothing names is never analyzed by the reference, so it is skipped
//! (`unreferenced_tuple_const`), not evaluated.
//!
//! `return [ ... ];` in a fn returning a slice is `@constCast(&[_]E{ ... })`:
//! an empty slice, or a slice of a constant array (`slice_literal_return`).

use super::*;

impl<'a> Lower<'a> {
    /// An untyped module constant whose value is an array literal and whose
    /// name no other node carries: Zig never analyzes it.
    pub(super) fn unreferenced_tuple_const(&self, items: &[&Node], name: &str) -> bool {
        let Some(node) = self.const_nodes.get(name) else { return false };
        let untyped_list = node.extra_type.trim().is_empty()
            && node.children.len() == 1
            && node.children[0].kind == NodeKind::ExprArrayLiteral;
        let named: usize = items.iter().map(|n| name_count(std::slice::from_ref(*n), name)).sum();
        untyped_list && named == 1
    }

    /// Called from `begin_body`: every `const xs = &[_]T{ ... }`, and every
    /// `const x = [N]T{ ... }` used as `&x`, whose uses all fit becomes a
    /// slice local of the callees' element type.
    pub(super) fn addr_lit_locals(&mut self, body: &[Node]) {
        let mut names = Vec::new();
        addr_lit_names(body, &mut names);
        for name in names {
            if self.slice_locals.contains_key(&name) {
                continue;
            }
            let mut decls = Vec::new();
            decls_of(body, &name, &mut decls);
            let [d] = decls[..] else { continue };
            if d.kind != NodeKind::StmtLocal || d.extra_mutable || !d.extra_type.trim().is_empty() || mutated(body, &name) {
                continue;
            }
            let init = &d.children[0];
            let by_addr = !is_addr_lit(init);
            let lit = if by_addr { init } else { &init.children[0] };
            if lit.children.is_empty() || !by_addr && lit.extra_size.trim() != "_" || lit.extra_size.contains(';') {
                continue;
            }
            let mut elem = None;
            if self.addr_lit_uses(body, &name, by_addr, &mut elem) {
                if let Some(e) = elem {
                    self.slice_locals.insert(name, e);
                }
            }
        }
    }

    /// Whether every node naming `name` (but its declaration) is an argument
    /// where the callee declares `[]const E` (one E for all) -- `name`
    /// itself, or `&name` when `by_addr` -- or `name.len`.
    fn addr_lit_uses(&self, ns: &[Node], name: &str, by_addr: bool, elem: &mut Option<LTy>) -> bool {
        for n in ns {
            if n.kind == NodeKind::StmtLocal && n.name == name {
                if !self.addr_lit_uses(&n.children, name, by_addr, elem) {
                    return false;
                }
                continue;
            }
            if n.kind == NodeKind::ExprFieldAccess
                && n.name == "len"
                && n.children.len() == 1
                && n.children[0].kind == NodeKind::ExprIdentifier
                && n.children[0].name == name
            {
                continue;
            }
            if n.kind == NodeKind::ExprCall {
                if let Some(sig) = self.sigs.get(&n.name) {
                    for (i, a) in n.children.iter().enumerate() {
                        let arg = if by_addr { addr_of_name(a) } else { Some(a) };
                        if arg.is_some_and(|a| a.kind == NodeKind::ExprIdentifier && a.name == name) {
                            let e = match sig.params.get(i) {
                                Some(LTy::Slice(e, false)) => (**e).clone(),
                                Some(LTy::Str) => LTy::S(Ty::U8),
                                _ => return false,
                            };
                            if !(e == LTy::Str || !has_brackets(&e)) {
                                return false;
                            }
                            match elem {
                                None => *elem = Some(e),
                                Some(u) if *u != e => return false,
                                _ => {}
                            }
                        } else if !self.addr_lit_uses(std::slice::from_ref(a), name, by_addr, elem) {
                            return false;
                        }
                    }
                    continue;
                }
            }
            if n.name == name || n.name.strip_prefix(name).is_some_and(|r| r.starts_with('.')) {
                return false;
            }
            if !self.addr_lit_uses(&n.children, name, by_addr, elem) {
                return false;
            }
        }
        true
    }

    /// `const xs = &[_]T{ ... }` taken by `addr_lit_locals`: the literal
    /// itself goes down the slice-local path.
    pub(super) fn addr_lit_local(&mut self, init: &Node, name: &str, out: &mut Vec<Stmt>) -> R<Option<()>> {
        if !is_addr_lit(init) {
            return Ok(None);
        }
        let Some(elem) = self.slice_locals.get(name).cloned() else { return Ok(None) };
        self.see(init);
        self.slice_local(&init.children[0], name.to_string(), elem, out)?;
        Ok(Some(()))
    }

    /// `return [ ... ];` (or `[_]T{ ... }`, `[]T{}`) where the fn returns a
    /// slice, written into `dst`. The reference writes
    /// `@constCast(&[_]E{ ... })` with E the return element type: empty, an
    /// empty slice; otherwise a slice of a constant array, which outlives the
    /// call because its elements are comptime values -- so here it is
    /// read-only data. Elements known only at run time are refused (the
    /// reference returns a pointer into its own frame), and so is a mutable
    /// slice from a fn some test reaches (a write through it faults there).
    pub(super) fn slice_literal_return(&mut self, c: &Node, dst: &Place, out: &mut Vec<Stmt>) -> R<bool> {
        if c.kind != NodeKind::ExprArrayLiteral || c.extra_size.contains(';') {
            return Ok(false);
        }
        let elem = match &dst.ty {
            LTy::Slice(e, _) => (**e).clone(),
            LTy::Str => LTy::S(Ty::U8),
            _ => return Ok(false),
        };
        if !(elem == LTy::Str || !has_brackets(&elem)) {
            return Ok(false);
        }
        if c.children.is_empty() && c.extra_size.trim().is_empty() {
            self.see(c);
            let Val::M(arr) = self.struct_temp(c, LTy::Arr(Box::new(elem), 0))? else {
                return Err(());
            };
            let Val::M(src) = self.slice_of(addr_of(&arr), 0, dst.ty.clone())? else {
                return Err(());
            };
            self.copy(dst, src, out)?;
            return Ok(true);
        }
        let text = self.text_lit(c)?;
        let lit = text.as_ref().unwrap_or(c);
        if lit.children.is_empty() {
            return Ok(false);
        }
        self.see(c);
        if matches!(dst.ty, LTy::Slice(_, true)) && !self.unanalyzed_fn {
            return self.reject(
                "ExprArrayLiteral(constant to mutable slice)",
                "an array literal returned as a mutable slice from a fn a test reaches".into(),
            );
        }
        let len = lit.children.len() as u32;
        let seen = self.errors.len();
        let arr = match self.rodata(lit, LTy::Arr(Box::new(elem), len)) {
            Ok(Val::M(arr)) => arr,
            Ok(_) => return Err(()),
            Err(()) => {
                if let Some(e) = self.errors.get_mut(seen).filter(|e| e.construct == "ConstDecl") {
                    e.construct = "ExprArrayLiteral(run-time slice return)".into();
                }
                return Err(());
            }
        };
        let Val::M(src) = self.slice_of(addr_of(&arr), len, dst.ty.clone())? else {
            return Err(());
        };
        self.copy(dst, src, out)?;
        Ok(true)
    }
}

fn is_addr_lit(n: &Node) -> bool {
    n.kind == NodeKind::ExprUnary
        && n.extra_op == "&"
        && n.children.len() == 1
        && n.children[0].kind == NodeKind::ExprArrayLiteral
}

/// `&x` with `x` a name: the name node.
pub(super) fn addr_of_name(n: &Node) -> Option<&Node> {
    (n.kind == NodeKind::ExprUnary && n.extra_op == "&" && n.children.len() == 1)
        .then(|| &n.children[0])
        .filter(|c| c.kind == NodeKind::ExprIdentifier)
}

fn addr_lit_names(ns: &[Node], out: &mut Vec<String>) {
    for n in ns {
        let lit = |c: &Node| is_addr_lit(c) || c.kind == NodeKind::ExprArrayLiteral;
        if n.kind == NodeKind::StmtLocal && !n.name.is_empty() && n.children.first().is_some_and(lit) {
            out.push(n.name.clone());
        }
        addr_lit_names(&n.children, out);
    }
}

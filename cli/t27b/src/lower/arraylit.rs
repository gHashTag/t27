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
//! `return [ ... ];` in a fn returning a slice, or a slice field's literal, is
//! `@constCast(&[_]E{ ... })` (`slice_lit`, plan `specs/tri/t27b/slice_lit_plan.t27`).
//! A write through it is undefined behaviour in Zig, so it is refused
//! (`slice_write`, `literal_writes`, #7765).

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

    /// `const x = [_]T{ ... }` with no annotation, which the reference prints
    /// `.{ ... }`: a tuple whose fields have type T, read at constant indices
    /// outside an invariant (plan `specs/tri/t27b/tuple_local_plan.t27`,
    /// #8050). None leaves it to the old refusal.
    pub(super) fn tuple_array_local(&mut self, init: &Node, name: &str) -> R<Option<()>> {
        let tt = init.extra_type.trim();
        let same = init.children.iter().all(|c| match c.kind {
            NodeKind::ExprStructLit => c.name == tt,
            NodeKind::ExprIdentifier => self.const_nodes.get(c.name.as_str()).is_some_and(|d| d.extra_type.trim() == tt),
            _ => false,
        });
        let n = init.children.len();
        if tl::storage(!tt.is_empty(), same, n, false) == tl::NOT_MINE {
            return Ok(None);
        }
        let t = LTy::Arr(Box::new(self.lty(tt)?), n as u32);
        let seen = self.errors.len();
        let agg = tl::storage(true, true, n, self.holds_str(&t)?) == tl::CONST_AGG;
        match if agg { self.const_agg(init, &t) } else { self.rodata(init, t) } {
            Ok(Val::M(p)) => self.bind(name, Binding::Mem(p)),
            Ok(v @ Val::A(..)) => self.bind(name, Binding::Const(v)),
            _ => {
                self.errors.truncate(seen);
                return Ok(None);
            }
        }
        self.see(init);
        self.tuple_names.insert(name.to_string());
        Ok(Some(()))
    }

    /// An array literal the reference prints `@constCast(&[_]E{ ... })`, E the
    /// element type of slice `dst`: a field's value or a return's
    /// (`return [ ... ];`, `[_]T{ ... }`, `[]T{}`). As
    /// `specs/tri/t27b/slice_lit_plan.t27` decides: a zero-length slice, or a
    /// slice of the one static of its type and value (Zig interns it), which
    /// only reads (`literal_writes`). False for NOT_MINE.
    pub(super) fn slice_lit(&mut self, at: u8, named: bool, c: &Node, dst: &Place, out: &mut Vec<Stmt>) -> R<bool> {
        if c.kind != NodeKind::ExprArrayLiteral || !matches!(dst.ty, LTy::Slice(..) | LTy::Str) {
            return Ok(false);
        }
        let elem = if let LTy::Slice(e, _) = &dst.ty { (**e).clone() } else { LTy::S(Ty::U8) };
        let flat = elem == LTy::Str || !has_brackets(&elem);
        let (repeat, typed, strings) = (c.extra_size.contains(';'), !c.extra_type.trim().is_empty(), self.holds_str(&elem)?);
        let ask = |n, text| sl::plan(at, named, flat, repeat, n, text, typed, strings);
        let (mut act, mut text) = (ask(c.children.len(), !c.extra_size.trim().is_empty()), None);
        if act == sl::PARSE {
            text = self.text_lit(c)?;
            act = ask(text.as_ref().map_or(0, |l| l.children.len()), false);
        }
        if act == sl::NOT_MINE {
            return Ok(false);
        }
        self.see(c);
        if sl::refuses(act) {
            return self.reject(sl::what(at, act), sl::why(at, act).into());
        }
        let lit = text.as_ref().unwrap_or(c);
        let len = lit.children.len() as u32;
        if sl::logs(act, matches!(dst.ty, LTy::Slice(_, true)), !self.unanalyzed_fn) {
            self.lit_log.push((act, format!("{:?}", elem), self.line));
        }
        let t = LTy::Arr(Box::new(elem), len);
        let arr = if sl::is_static(act) {
            let (size, _) = self.size_align(&t)?;
            let (mut buf, seen) = (vec![0u8; size as usize], self.errors.len());
            if self.const_fill(lit, &t, &mut buf, 0).is_err() {
                if self.errors.get(seen).is_some_and(|e| e.construct == "ConstDecl") {
                    self.errors.truncate(seen);
                    return self.reject(sl::what(at, sl::REFUSE_RUN_TIME), sl::why(at, sl::REFUSE_RUN_TIME).into());
                }
                return Err(());
            }
            let n = self.globals_init.len() as u32;
            let k = *self.statics.entry((format!("{:?}", t), buf.clone())).or_insert(n);
            if k == n {
                self.globals_init.push(buf);
            }
            Place { addr: Expr { ty: Ty::Ptr, kind: ExprKind::Global(k) }, off: 0, ty: t, mutable: true, temp: None }
        } else {
            let Val::M(p) = self.struct_temp(c, t)? else { return Err(()) };
            p
        };
        let Val::M(src) = self.slice_of(addr_of(&arr), len, dst.ty.clone())? else {
            return Err(());
        };
        self.copy(dst, src, out)?;
        Ok(true)
    }

    /// `slice_index` of a slice of `elem`: a WRITE of the write check when
    /// `lvalue` builds the place (a store, or `&xs[i]`).
    pub(super) fn slice_write(&mut self, elem: &LTy, mutable: bool) {
        if self.writing && sl::logs(sl::WRITE, mutable, !self.unanalyzed_fn) {
            self.lit_log.push((sl::WRITE, format!("{:?}", elem), self.line));
        }
    }

    /// After every body: each WRITE whose element type a STATIC literal
    /// backs is refused at the write, as the plan's `refuses_write` decides.
    pub(super) fn literal_writes(&mut self) {
        let log = std::mem::take(&mut self.lit_log);
        for (act, elem, line) in &log {
            if sl::refuses_write(*act, log.iter().any(|(a, e, _)| *a == sl::STATIC && e == elem)) {
                self.line = *line;
                let _: R<()> = self.reject(sl::what(sl::AT_WRITE, sl::REFUSE_WRITE), sl::why(sl::AT_WRITE, sl::REFUSE_WRITE).into());
            }
        }
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

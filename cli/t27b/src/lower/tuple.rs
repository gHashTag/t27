//! Tuples, the way t27c's Zig backend writes them.
//!
//! The reference turns a tuple type into Zig only where it is a function's
//! return type: `-> (u32, bool)` is `struct { u32, bool }`, and the named form
//! `-> (lo: u32, hi: u32)` is `struct { lo: u32, hi: u32 }`. Anywhere else
//! (a parameter, a local's annotation, a field) it prints the t27 spelling
//! verbatim, which Zig cannot parse, so those stay `type (tuple)`.
//!
//! Here a tuple type is a struct whose fields are named `0`, `1`, ... (or by
//! the names the type gives), interned by its spelling, so every function
//! returning `(u32, u32)` returns the same type. Values of it come from:
//!
//! - `return (a, b)` (Zig `.{ a, b }`; for a named tuple `.{ .lo = a, .hi = b }`),
//!   and a tuple-returning call;
//! - `t.0`, `t.lo`: an ordinary field access;
//! - `let (a, b) = f()` / `var (a, b) = f()`: Zig's `const a, const b = f()`.
//!   A named tuple is read field by field (the reference's `__tN.lo`), always
//!   as constants, and only from a direct call, as the reference does;
//! - `(a, b) = f()` at the top of a test: fresh constants, like the
//!   reference's `const a, const b = f()` there. In a function body the
//!   reference writes `.{ a, b } = f()`, which Zig refuses.
//!
//! A `(a, b)` / `.{ a, b }` literal also initialises an array (`[N]T`) where one
//! is the result type, which is what Zig's anonymous list literal does.
//! Without a result type (`const t = (a, 2)`) it is still `ExprTuple`.

use super::*;

impl<'a> Lower<'a> {
    /// A function's return type: a tuple is taken here, everything else is
    /// `lty`.
    pub(super) fn ret_lty(&mut self, rt: &str) -> R<LTy> {
        let t = rt.trim();
        if !(t.starts_with('(') && t.ends_with(')') && t.contains(',')) {
            return self.lty(t);
        }
        let inner = &t[1..t.len() - 1];
        let mut elems: Vec<(Option<String>, String)> = Vec::new();
        for e in inner.split(',') {
            let e = e.trim();
            // The reference splits on every comma, so a nested tuple or a
            // generic argument list is torn apart there.
            if e.is_empty() || e.contains('(') || e.contains(')') {
                return self.reject("type (tuple)", format!("`{}`: element `{}`", t, e));
            }
            match e.split_once(':') {
                Some((n, ty))
                    if !n.trim().is_empty()
                        && n.trim().chars().all(|c| c.is_alphanumeric() || c == '_')
                        && !n.trim().starts_with(|c: char| c.is_ascii_digit())
                        && !ty.trim().is_empty() =>
                {
                    elems.push((Some(n.trim().to_string()), ty.trim().to_string()))
                }
                _ => elems.push((None, e.to_string())),
            }
        }
        let named = elems.iter().filter(|(n, _)| n.is_some()).count();
        if named != 0 && named != elems.len() {
            // `tuple_field_names` takes all names or none; the rest is
            // printed as a type.
            return self.reject("type (tuple)", format!("`{}`: some elements named, some not", t));
        }
        let key = format!(
            "({})",
            elems
                .iter()
                .map(|(n, ty)| match n {
                    Some(n) => format!("{}: {}", n, ty),
                    None => ty.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
        if let Some(&id) = self.struct_ids.get(&key) {
            self.layout(id)?;
            return Ok(LTy::Struct(id));
        }
        let mut fields: Vec<Field<'a>> = Vec::new();
        let (mut size, mut align) = (0u32, 1u32);
        for (i, (n, ety)) in elems.iter().enumerate() {
            let name = n.clone().unwrap_or_else(|| i.to_string());
            if fields.iter().any(|f| f.name == name) {
                return self.reject("type (tuple)", format!("`{}` names `{}` twice", t, name));
            }
            let ty = self.lty(ety)?;
            let (fs, fa) = self.size_align(&ty)?;
            let off = size.div_ceil(fa) * fa;
            size = off + fs;
            align = align.max(fa);
            fields.push(Field { name, ty, off, default: None });
        }
        let id = self.structs.len() as u32;
        self.structs.push(StructDef {
            name: key.clone(),
            fields,
            size: Some(size.div_ceil(align) * align),
            align,
            fail: None,
        });
        self.struct_ids.insert(key, id);
        Ok(LTy::Struct(id))
    }

    /// Whether struct `id` is a tuple type (see `ret_lty`).
    fn is_tuple(&self, id: u32) -> bool {
        self.structs[id as usize].name.starts_with('(')
    }

    /// `(a, b)` / `.{ a, b }` written into `dst`: a tuple's fields in order,
    /// or an array's elements.
    pub(super) fn init_tuple_lit(&mut self, n: &Node, dst: Place, fresh: bool, out: &mut Vec<Stmt>) -> R<()> {
        let t = dst.ty.clone();
        match &t {
            LTy::Arr(..) => {
                if fresh && pure_addr(&dst.addr) {
                    return self.init_array(n, &dst, out);
                }
                // The literal may read `dst`: build it aside first.
                let k = self.new_slot(&t)?;
                let tmp = Place { addr: slot_expr(k), off: 0, ty: t, mutable: true, temp: None };
                self.init_array(n, &tmp, out)?;
                self.copy(&dst, tmp, out)
            }
            LTy::Struct(id) if self.is_tuple(*id) => {
                let id = *id;
                let fields = self.fields(id)?;
                if n.children.len() != fields.len() {
                    let tn = self.type_name(&t);
                    return self.reject("ExprTuple", format!("{} values for `{}`", n.children.len(), tn));
                }
                let aside = !(fresh && pure_addr(&dst.addr));
                let target = if aside {
                    // The literal may read `dst`: build it aside first.
                    let k = self.new_slot(&t)?;
                    Place { addr: slot_expr(k), off: 0, ty: t.clone(), mutable: true, temp: None }
                } else {
                    dst.clone()
                };
                for (c, f) in n.children.iter().zip(fields.iter()) {
                    self.init(c, field_place(&target, f), true, out)?;
                }
                if aside {
                    self.copy(&dst, target, out)?;
                }
                Ok(())
            }
            _ => {
                let tn = self.type_name(&t);
                self.reject("ExprTuple", format!("a tuple literal where `{}` is expected", tn))
            }
        }
    }

    /// `let (a, b) = init` / `var (a, b) = init` (a `StmtLocal` with no name
    /// and the names in `extra_field`).
    pub(super) fn destructure_local(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        let names: Vec<String> =
            n.extra_field.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let Some(init) = n.children.first() else {
            return self.reject("StmtLocal(destructure)", format!("`({})` has no value", n.extra_field));
        };
        if !n.extra_type.trim().is_empty() {
            return self.reject("StmtLocal(destructure)", format!("`({})` with a type annotation", n.extra_field));
        }
        self.destructure(&names, init, n.extra_mutable, out)
    }

    /// `(a, b) = init` (a `StmtAssign` whose target is a tuple). The reference
    /// declares fresh constants only at the top of a test.
    pub(super) fn tuple_assign(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        let target = &n.children[0];
        let top_of_test = self.in_test && self.scopes.len() == 1;
        if !top_of_test || !(n.extra_op.is_empty() || n.extra_op == "=") {
            return self.reject(
                "StmtAssign(tuple)",
                "a tuple assignment outside the top of a test (the reference writes `.{ a, b } = ..`, which Zig refuses)".into(),
            );
        }
        let mut names = Vec::new();
        for e in &target.children {
            if e.kind != NodeKind::ExprIdentifier || e.name.is_empty() {
                let k = kind_name(e);
                return self.reject("StmtAssign(tuple)", format!("{} in a tuple target", k));
            }
            if e.name != "_" && self.lookup(&e.name).is_some() {
                return self.reject(
                    "StmtAssign(tuple)",
                    format!("`{}` is already bound (the reference declares it again)", e.name),
                );
            }
            names.push(e.name.clone());
        }
        self.destructure(&names, &n.children[1], false, out)
    }

    fn destructure(&mut self, names: &[String], init: &Node, mutable: bool, out: &mut Vec<Stmt>) -> R<()> {
        for name in names {
            if name.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                return self.reject("StmtLocal", format!("binding `{}`", name));
            }
            if name != "_" {
                self.no_var_shadow(name)?;
            }
        }
        if init.kind == NodeKind::ExprTuple {
            return self.destructure_lit(names, init, mutable, out);
        }
        let v = self.expr(init)?;
        let src = match v {
            Val::M(p) => p,
            Val::Poison => return Err(()),
            _ => return self.reject("StmtLocal(destructure)", "a value that is not a tuple or an array".into()),
        };
        // Each element's offset and type, and whether the reference reads it
        // field by field (a named tuple) rather than destructuring.
        let (elems, by_field): (Vec<(u32, LTy)>, bool) = match src.ty.clone() {
            LTy::Struct(id) if self.is_tuple(id) => {
                let fields = self.fields(id)?;
                let named = fields.first().is_some_and(|f| f.name != "0");
                if named && init.kind != NodeKind::ExprCall {
                    return self.reject(
                        "StmtLocal(destructure)",
                        "a named tuple not returned by a direct call (Zig cannot destructure a struct)".into(),
                    );
                }
                (fields.iter().map(|f| (f.off, f.ty.clone())).collect(), named)
            }
            LTy::Arr(elem, len) => {
                let (esize, _) = self.size_align(&elem)?;
                ((0..len).map(|i| (i * esize, (*elem).clone())).collect(), false)
            }
            other => {
                let tn = self.type_name(&other);
                return self.reject("StmtLocal(destructure)", format!("`{}` is not a tuple or an array", tn));
            }
        };
        if elems.len() != names.len() {
            return self.reject(
                "StmtLocal(destructure)",
                format!("{} names for {} elements", names.len(), elems.len()),
            );
        }
        // The elements live in memory no other name sees: a temporary (a
        // call's result) as it is, anything else copied out once, since the
        // source may be a variable that changes later.
        let slot = match src.temp {
            Some(k) if src.off == 0 => {
                match src.addr.kind {
                    ExprKind::Slot(_) => {}
                    ExprKind::Seq { stmts, .. } => out.extend(stmts),
                    _ => out.push(Stmt::Eval(src.addr)),
                }
                k
            }
            _ => {
                let k = self.new_slot(&src.ty)?;
                let own = Place { addr: slot_expr(k), off: 0, ty: src.ty.clone(), mutable: true, temp: None };
                self.copy(&own, src, out)?;
                k
            }
        };
        // A named tuple is read into `const` bindings whatever the spec said.
        let mutable = mutable && !by_field;
        for (name, (off, ty)) in names.iter().zip(elems) {
            if name == "_" {
                continue;
            }
            self.bind(name, Binding::Mem(Place { addr: slot_expr(slot), off, ty, mutable, temp: None }));
        }
        Ok(())
    }

    /// `let (a, b) = (x, y)`: Zig's `const a, const b = .{ x, y }`, each name
    /// bound to its element as `const a = x` would bind it. Every element is
    /// evaluated before any name is bound.
    fn destructure_lit(&mut self, names: &[String], init: &Node, mutable: bool, out: &mut Vec<Stmt>) -> R<()> {
        self.see(init);
        if init.children.len() != names.len() {
            return self.reject(
                "StmtLocal(destructure)",
                format!("{} names for {} elements", names.len(), init.children.len()),
            );
        }
        let mut vals = Vec::new();
        for c in &init.children {
            vals.push(self.expr(c)?);
        }
        for (name, v) in names.iter().zip(vals) {
            if name == "_" {
                // `_` still evaluates its element.
                match v {
                    Val::E(e) | Val::P(e, _) => out.push(Stmt::Eval(e)),
                    Val::M(p) if !pure_addr(&p.addr) => out.push(Stmt::Eval(p.addr)),
                    _ => {}
                }
                continue;
            }
            match v {
                // A compile-time number stays one, as in `const a = 2`; a
                // `var` of one is refused there.
                Val::Ct(_) | Val::Cf(..) if !mutable => self.bind(name, Binding::Const(v)),
                v => self.bind_value(name, v, mutable, out)?,
            }
        }
        Ok(())
    }

    /// `t[i]` for a tuple `t`: Zig takes a compile-time index there and reads
    /// field `i`. Anything else comes back as it went in.
    pub(super) fn tuple_index(&mut self, base: Val, idx: &Val) -> R<Result<Place, Val>> {
        let p = match base {
            Val::M(p) if matches!(p.ty, LTy::Struct(id) if self.is_tuple(id)) => p,
            v => return Ok(Err(v)),
        };
        let LTy::Struct(id) = p.ty else { unreachable!() };
        let fields = self.fields(id)?;
        let tn = self.structs[id as usize].name.clone();
        if fields.first().is_some_and(|f| f.name != "0") {
            return self.reject("ExprIndex(tuple)", format!("index of the named tuple `{}`", tn));
        }
        let c = match idx {
            Val::Ct(c) => *c,
            Val::E(Expr { kind: ExprKind::Const(c), ty }) if ty.is_int() => *c,
            _ => return self.reject("ExprIndex(tuple)", format!("index of `{}` not known at compile time", tn)),
        };
        if !(0..fields.len() as i128).contains(&c) {
            return self.reject("ExprIndex(tuple)", format!("index {} out of bounds for `{}`", c, tn));
        }
        let mut q = field_place(&p, &fields[c as usize]);
        // A field of a temporary is read from it only once.
        q.mutable = p.mutable && p.temp.is_none();
        Ok(Ok(q))
    }
}

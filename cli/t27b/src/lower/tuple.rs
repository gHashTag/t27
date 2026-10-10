//! Tuples, the way t27c's Zig backend writes them: what the reference does, the spelling of a tuple type and
//! every refusal here are `specs/tri/t27b/tuple_local_plan.t27` (module `tl`); this glue walks the nodes.

use super::*;

impl<'a> Lower<'a> {
    /// Refusal `r` of the plan, the `{}` holes in its words filled from `args` in order.
    pub(super) fn tuple_refuse<T>(&mut self, r: u8, args: &[&str]) -> R<T> {
        let mut words = tl::why(r).split("{}");
        let mut s = words.next().unwrap_or("").to_string();
        s.extend(args.iter().zip(words).flat_map(|(a, w)| [*a, w]));
        self.reject(tl::what(r), s)
    }

    /// A function's return type: a tuple is taken here, everything else is
    /// `lty`.
    pub(super) fn ret_lty(&mut self, rt: &str) -> R<LTy> {
        let t = rt.trim();
        if !tl::is_tuple(t.as_bytes()) {
            return self.lty(t);
        }
        // Each element's name and type as byte ranges of `t`, and the key the type is interned by.
        let (mut at, mut kb) = (vec![0usize; 4 * t.len()], vec![0u8; 3 * t.len()]);
        let n = tl::split(t.as_bytes(), &mut at);
        if n < 2 {
            return self.tuple_refuse(n as u8, &[t, &t[at[0]..at[1]]]);
        }
        let k = tl::key(t.as_bytes(), at.clone(), n, &mut kb);
        let key = String::from_utf8_lossy(&kb[..k]).into_owned();
        let elems: Vec<_> = at.chunks(4).take(n).map(|r| ((r[0] < r[1]).then(|| t[r[0]..r[1]].to_string()), &t[r[2]..r[3]])).collect();
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

    /// `const t = .{ a, b }` / `const t = (a, b)` with no type: the
    /// reference prints `const t = .{ a, b };`, a Zig tuple whose fields
    /// have the types of the run-time values in it, read as `t[0]` at a
    /// compile-time index (`tuple_index`). Only run-time scalars: a literal
    /// element is a `comptime` field of type `comptime_int` /
    /// `comptime_float`, which no layout here models, and an aggregate or
    /// pointer element is left out until something needs it.
    pub(super) fn tuple_value_local(&mut self, init: &Node, name: &str, out: &mut Vec<Stmt>) -> R<()> {
        self.see(init);
        if init.children.len() < 2 {
            return self.reject("ExprTuple", format!("`{}` = a tuple of {} values", name, init.children.len()));
        }
        let mut vals = Vec::new();
        for c in &init.children {
            match self.expr(c)? {
                Val::E(e) => vals.push(e),
                Val::Poison => return Err(()),
                v => {
                    let d = self.val_desc(&v);
                    return self.tuple_refuse(tl::R_HOLDING, &[name, &d]);
                }
            }
        }
        let spelled = vals.iter().map(|e| e.ty.name().to_string()).collect::<Vec<_>>().join(", ");
        let t = self.ret_lty(&format!("({})", spelled))?;
        let LTy::Struct(id) = t else { return Err(()) };
        let fields = self.fields(id)?;
        let k = self.new_slot(&t)?;
        let dst = Place { addr: slot_expr(k), off: 0, ty: t, mutable: false, temp: None };
        for (e, f) in vals.into_iter().zip(fields.iter()) {
            out.push(Stmt::Store { addr: dst.addr.clone(), off: dst.off + f.off, value: e });
        }
        self.bind(name, Binding::Mem(dst));
        Ok(())
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
            // `.{}` for a declared struct: every field's default (plan `empty_lit_plan.t27`, #7735).
            LTy::Struct(id) if el::plan(el::WANT_STRUCT, false, n.children.len()) == el::DEFAULTS => {
                let id = *id;
                if fresh && pure_addr(&dst.addr) {
                    return self.init_struct(n, id, &dst, out);
                }
                let k = self.new_slot(&t)?;
                let tmp = Place { addr: slot_expr(k), off: 0, ty: t.clone(), mutable: true, temp: None };
                self.init_struct(n, id, &tmp, out)?;
                self.copy(&dst, tmp, out)
            }
            _ => self.tuple_refuse(tl::R_EXPECTED, &[&self.type_name(&t)]),
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
        if !tl::assigns(self.in_test && self.scopes.len() == 1, n.extra_op.as_bytes()) {
            return self.tuple_refuse(tl::R_OUTSIDE, &[]);
        }
        let mut names = Vec::new();
        for e in &target.children {
            if e.kind != NodeKind::ExprIdentifier || e.name.is_empty() {
                let k = kind_name(e);
                return self.reject("StmtAssign(tuple)", format!("{} in a tuple target", k));
            }
            if e.name != "_" && self.lookup(&e.name).is_some() {
                return self.tuple_refuse(tl::R_BOUND, &[&e.name]);
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
                    return self.tuple_refuse(tl::R_NAMED_CALL, &[]);
                }
                (fields.iter().map(|f| (f.off, f.ty.clone())).collect(), named)
            }
            LTy::Arr(elem, len) => {
                let (esize, _) = self.size_align(&elem)?;
                ((0..len).map(|i| (i * esize, (*elem).clone())).collect(), false)
            }
            other => return self.tuple_refuse(tl::R_NOT_TUPLE, &[&self.type_name(&other)]),
        };
        if elems.len() != names.len() {
            return self.tuple_refuse(tl::R_NAMES, &[&names.len().to_string(), &elems.len().to_string()]);
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
            return self.tuple_refuse(tl::R_NAMES, &[&names.len().to_string(), &init.children.len().to_string()]);
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

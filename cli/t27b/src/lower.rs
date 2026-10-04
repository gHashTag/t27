//! Lowering: t27c AST -> t27b IR.
//!
//! This is where the supported subset is enforced. Anything outside it is
//! reported as a `Reject` naming the AST construct and the nearest known source
//! line; nothing outside the subset reaches the back end.
//!
//! Typing follows Zig, which is what the t27 reference backend emits:
//! integer literals are comptime integers (exact, arbitrary sign) that take the
//! type of the typed operand they meet and must fit in it; two typed operands
//! must agree up to lossless widening (same signedness and not narrower, or
//! unsigned into a strictly wider signed type); `if`/`while` conditions are
//! `bool`; `bool` and integers never mix.

use crate::compiler::{Node, NodeKind};
use crate::ir::*;
use std::collections::{HashMap, HashSet};

/// A construct outside the supported subset (or a type error inside it).
#[derive(Clone, Debug)]
pub struct Reject {
    /// The AST node kind or a short category such as `type f32`.
    pub construct: String,
    pub line: u32,
    pub detail: String,
}

impl Reject {
    pub fn message(&self) -> String {
        let mut s = format!(
            "t27b: unsupported construct {} at line {}",
            self.construct, self.line
        );
        if !self.detail.is_empty() {
            s.push_str(&format!(" ({})", self.detail));
        }
        s
    }
}

type R<T> = Result<T, ()>;

/// A lowered expression: either a comptime integer (exact, untyped) or a
/// typed IR expression.
#[derive(Clone, Debug)]
enum Val {
    Ct(i128),
    E(Expr),
}

#[derive(Clone, Debug)]
enum Binding {
    Var { id: VarId, mutable: bool },
    Const(Val),
}

struct Sig {
    id: FuncId,
    params: Vec<Ty>,
    ret: Option<Ty>,
}

struct Lower<'a> {
    mode: OverflowMode,
    sites: Vec<Site>,
    errors: Vec<Reject>,
    sigs: HashMap<String, Sig>,
    globals: HashMap<String, Val>,
    const_nodes: HashMap<String, &'a Node>,
    resolving: HashSet<String>,
    // Per-function state.
    vars: Vec<Var>,
    scopes: Vec<HashMap<String, Binding>>,
    loop_depth: u32,
    line: u32,
    ret: Option<Ty>,
    in_test: bool,
    test_assigns: HashMap<String, u32>,
}

/// Lower a parsed module. All rejected constructs are returned (at most one per
/// top-level item, since lowering of an item stops at its first rejection).
pub fn lower(ast: &Node, mode: OverflowMode) -> Result<Program, Vec<Reject>> {
    let mut l = Lower {
        mode,
        sites: vec![Site {
            kind: TrapKind::Overflow,
            line: 0,
            what: "none".into(),
            ty: Ty::Bool,
        }],
        errors: Vec::new(),
        sigs: HashMap::new(),
        globals: HashMap::new(),
        const_nodes: HashMap::new(),
        resolving: HashSet::new(),
        vars: Vec::new(),
        scopes: Vec::new(),
        loop_depth: 0,
        line: ast.line,
        ret: None,
        in_test: false,
        test_assigns: HashMap::new(),
    };
    let module = if ast.kind == NodeKind::Module {
        ast.name.clone()
    } else {
        String::new()
    };
    let items: Vec<&Node> = if ast.kind == NodeKind::Module {
        ast.children.iter().collect()
    } else {
        vec![ast]
    };

    // Pass 1: signatures and constant declarations.
    let mut fn_nodes: Vec<&Node> = Vec::new();
    let mut next_id: FuncId = 0;
    for item in &items {
        l.see(item);
        match item.kind {
            NodeKind::FnDecl => {
                if l.sigs.contains_key(&item.name) {
                    let _: R<()> = l.reject(
                        "FnDecl",
                        format!("duplicate function `{}`", item.name),
                    );
                    continue;
                }
                match l.signature(item) {
                    Ok((params, ret)) => {
                        l.sigs.insert(
                            item.name.clone(),
                            Sig {
                                id: next_id,
                                params,
                                ret,
                            },
                        );
                        next_id += 1;
                        fn_nodes.push(item);
                    }
                    Err(()) => {}
                }
            }
            NodeKind::TestBlock => {}
            NodeKind::ConstDecl => {
                l.const_nodes.insert(item.name.clone(), item);
            }
            // `use` declarations were already resolved by splicing the
            // imported declarations into the source (front::parse).
            NodeKind::UseDecl => {}
            // The t27c parser reads a dotted `module a.b;` or `use a.b;` as
            // `a` followed by a stray top-level expression `.b`, with no line.
            NodeKind::StmtExpr
                if matches!(item.children.first(), Some(c) if c.kind == NodeKind::ExprEnumValue) =>
            {
                let name = &item.children[0].name;
                let _: R<()> = l.reject(
                    "StmtExpr",
                    format!(
                        "stray `.{}` at top level; t27c parses a dotted `module a.b;` or `use a.b;` as `a` followed by `.b`",
                        name
                    ),
                );
            }
            _ => {
                let k = kind_name(item);
                let detail = if k.starts_with("Stmt") || k.starts_with("Expr") {
                    "statement at top level, outside any fn or test".to_string()
                } else {
                    String::new()
                };
                let _: R<()> = l.reject(&k, detail);
            }
        }
    }
    let mut const_names: Vec<String> = l.const_nodes.keys().cloned().collect();
    const_names.sort();
    for name in const_names {
        let _ = l.global(&name);
    }

    // Pass 2: bodies.
    let mut funcs: Vec<Func> = Vec::new();
    for item in &fn_nodes {
        if let Ok(f) = l.function(item) {
            funcs.push(f);
        }
    }
    for item in &items {
        if item.kind == NodeKind::TestBlock {
            if let Ok(f) = l.test(item) {
                funcs.push(f);
            }
        }
    }
    if !l.errors.is_empty() {
        return Err(l.errors);
    }
    Ok(Program {
        module,
        funcs,
        sites: l.sites,
        mode,
    })
}

fn kind_name(n: &Node) -> String {
    format!("{:?}", n.kind)
}

/// Parse an integer literal: decimal, 0x, 0o, 0b, with `_` separators.
fn parse_int(s: &str) -> Option<i128> {
    let t: String = s.chars().filter(|c| *c != '_').collect();
    let (digits, radix) = if let Some(r) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        (r.to_string(), 16)
    } else if let Some(r) = t.strip_prefix("0o") {
        (r.to_string(), 8)
    } else if let Some(r) = t.strip_prefix("0b") {
        (r.to_string(), 2)
    } else {
        (t.clone(), 10)
    };
    if digits.is_empty() {
        return None;
    }
    let v = u128::from_str_radix(&digits, radix).ok()?;
    if v > i128::MAX as u128 {
        return None;
    }
    Some(v as i128)
}

impl<'a> Lower<'a> {
    fn see(&mut self, n: &Node) {
        if n.line != 0 {
            self.line = n.line;
        }
    }

    fn reject<T>(&mut self, construct: &str, detail: String) -> R<T> {
        self.errors.push(Reject {
            construct: construct.to_string(),
            line: self.line,
            detail,
        });
        Err(())
    }

    fn site(&mut self, kind: TrapKind, what: String, ty: Ty) -> SiteId {
        self.sites.push(Site {
            kind,
            line: self.line,
            what,
            ty,
        });
        (self.sites.len() - 1) as SiteId
    }

    fn ty(&mut self, name: &str) -> R<Ty> {
        let t = name.trim();
        match Ty::from_name(t) {
            Some(ty) => Ok(ty),
            None => self.reject(&format!("type {}", t), String::new()),
        }
    }

    fn signature(&mut self, n: &Node) -> R<(Vec<Ty>, Option<Ty>)> {
        if n.params.len() > 8 {
            return self.reject(
                "FnDecl",
                format!("`{}` has {} parameters, at most 8 are supported", n.name, n.params.len()),
            );
        }
        let mut params = Vec::new();
        for (pname, pty) in &n.params {
            if pname.starts_with("comptime ") || pty.is_empty() {
                return self.reject("FnDecl", format!("parameter `{}` of `{}`", pname, n.name));
            }
            params.push(self.ty(pty)?);
        }
        let rt = n.extra_return_type.trim();
        let ret = if rt.is_empty() || rt == "void" {
            None
        } else {
            Some(self.ty(rt)?)
        };
        Ok((params, ret))
    }

    // ---------------------------------------------------------------- scopes

    fn lookup(&self, name: &str) -> Option<Binding> {
        for s in self.scopes.iter().rev() {
            if let Some(b) = s.get(name) {
                return Some(b.clone());
            }
        }
        None
    }

    fn bind(&mut self, name: &str, b: Binding) {
        if let Some(s) = self.scopes.last_mut() {
            s.insert(name.to_string(), b);
        }
    }

    fn new_var(&mut self, name: &str, ty: Ty, mutable: bool) -> VarId {
        let id = self.vars.len() as VarId;
        self.vars.push(Var {
            name: name.to_string(),
            ty,
        });
        self.bind(name, Binding::Var { id, mutable });
        id
    }

    /// Resolve a module-level `const`, evaluating it on first use.
    fn global(&mut self, name: &str) -> R<Option<Val>> {
        if let Some(v) = self.globals.get(name) {
            return Ok(Some(v.clone()));
        }
        let node = match self.const_nodes.get(name) {
            Some(n) => *n,
            None => return Ok(None),
        };
        if !self.resolving.insert(name.to_string()) {
            return self.reject("ConstDecl", format!("`{}` refers to itself", name));
        }
        let saved_line = self.line;
        let saved_scopes = std::mem::take(&mut self.scopes);
        self.see(node);
        let r = self.global_value(node);
        self.scopes = saved_scopes;
        self.line = saved_line;
        self.resolving.remove(name);
        let v = r?;
        self.globals.insert(name.to_string(), v.clone());
        Ok(Some(v))
    }

    fn global_value(&mut self, node: &Node) -> R<Val> {
        let init = match node.children.first() {
            Some(c) => c,
            None => return self.reject("ConstDecl", format!("`{}` has no value", node.name)),
        };
        let v = self.expr(init)?;
        let v = if node.extra_type.trim().is_empty() {
            v
        } else {
            let ty = self.ty(&node.extra_type)?;
            Val::E(self.coerce(v, ty)?)
        };
        match &v {
            Val::Ct(_) => Ok(v),
            Val::E(e) if matches!(e.kind, ExprKind::Const(_)) => Ok(v),
            _ => self.reject(
                "ConstDecl",
                format!("`{}` is not a compile-time integer or bool", node.name),
            ),
        }
    }

    // ------------------------------------------------------------- functions

    fn begin_body(&mut self) {
        self.vars.clear();
        self.scopes.clear();
        self.scopes.push(HashMap::new());
        self.loop_depth = 0;
    }

    fn function(&mut self, n: &Node) -> R<Func> {
        self.see(n);
        self.begin_body();
        self.in_test = false;
        let (params, ret) = {
            let s = &self.sigs[&n.name];
            (s.params.clone(), s.ret)
        };
        self.ret = ret;
        for (i, (pname, _)) in n.params.iter().enumerate() {
            self.new_var(pname, params[i], true);
        }
        let body = self.stmts(&n.children)?;
        let noreturn_site = if ret.is_some() {
            self.site(TrapKind::NoReturn, format!("end of fn {}", n.name), Ty::Bool)
        } else {
            0
        };
        Ok(Func {
            name: n.name.clone(),
            nparams: params.len(),
            ret,
            vars: std::mem::take(&mut self.vars),
            body,
            line: n.line,
            is_test: false,
            noreturn_site,
        })
    }

    fn test(&mut self, n: &Node) -> R<Func> {
        self.see(n);
        if n.extra_field == "partial" {
            return self.reject(
                "TestBlock",
                format!("test `{}` was only partially parsed by the front-end", n.name),
            );
        }
        self.begin_body();
        self.in_test = true;
        self.ret = None;
        self.test_assigns.clear();
        count_assigns(&n.children, &mut self.test_assigns);
        let body = self.stmts(&n.children)?;
        self.in_test = false;
        Ok(Func {
            name: n.name.clone(),
            nparams: 0,
            ret: None,
            vars: std::mem::take(&mut self.vars),
            body,
            line: n.line,
            is_test: true,
            noreturn_site: 0,
        })
    }

    // ------------------------------------------------------------ statements

    /// A nested block: its own scope.
    fn block(&mut self, n: &Node) -> R<Vec<Stmt>> {
        self.scopes.push(HashMap::new());
        let r = if n.kind == NodeKind::Module {
            self.stmts(&n.children)
        } else {
            self.stmts(std::slice::from_ref(n))
        };
        self.scopes.pop();
        r
    }

    fn stmts(&mut self, ns: &[Node]) -> R<Vec<Stmt>> {
        let mut out = Vec::new();
        for n in ns {
            self.stmt(n, &mut out)?;
        }
        Ok(out)
    }

    fn stmt(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        self.see(n);
        match n.kind {
            NodeKind::StmtLocal => self.local(n, out),
            NodeKind::StmtAssign => self.assign(n, out),
            NodeKind::StmtIf => {
                if n.children.len() < 2 || n.children.len() > 3 {
                    return self.reject("StmtIf", "unexpected shape".into());
                }
                let cond = self.cond(&n.children[0])?;
                let then = self.block(&n.children[1])?;
                let els = match n.children.get(2) {
                    Some(e) => self.block(e)?,
                    None => Vec::new(),
                };
                out.push(Stmt::If { cond, then, els });
                Ok(())
            }
            NodeKind::StmtWhile => {
                if !n.name.is_empty() {
                    return self.reject("StmtWhile", format!("labelled loop `{}`", n.name));
                }
                let k = n.children.len();
                if !(2..=3).contains(&k) {
                    return self.reject("StmtWhile", "unexpected shape".into());
                }
                let cond = self.cond(&n.children[0])?;
                let body_node = &n.children[k - 1];
                if body_node.kind != NodeKind::Module || body_node.name != "body" {
                    return self.reject("StmtWhile", "unexpected body shape".into());
                }
                self.loop_depth += 1;
                let body = self.block(body_node);
                self.loop_depth -= 1;
                let body = body?;
                let step = if k == 3 {
                    let c = &n.children[1];
                    if c.kind != NodeKind::Module || c.name != "continue_expr" {
                        return self.reject("StmtWhile", "unexpected continue expression".into());
                    }
                    self.block(c)?
                } else {
                    Vec::new()
                };
                out.push(Stmt::While { cond, body, step });
                Ok(())
            }
            NodeKind::StmtBreak | NodeKind::StmtContinue => {
                let k = kind_name(n);
                if !n.name.is_empty() || !n.children.is_empty() {
                    return self.reject(&k, "labelled or valued break/continue".into());
                }
                if self.loop_depth == 0 {
                    return self.reject(&k, "outside a loop".into());
                }
                out.push(if n.kind == NodeKind::StmtBreak {
                    Stmt::Break
                } else {
                    Stmt::Continue
                });
                Ok(())
            }
            NodeKind::ExprReturn => {
                let v = match n.children.first() {
                    None => None,
                    Some(c) => Some(c),
                };
                match (self.ret, v) {
                    (None, None) => out.push(Stmt::Return(None)),
                    (Some(t), Some(c)) => {
                        let v = self.expr(c)?;
                        let e = self.coerce(v, t)?;
                        out.push(Stmt::Return(Some(e)));
                    }
                    (None, Some(_)) => {
                        return self.reject("ExprReturn", "value returned from a void fn or test".into())
                    }
                    (Some(_), None) => {
                        return self.reject("ExprReturn", "missing return value".into())
                    }
                }
                Ok(())
            }
            NodeKind::StmtExpr => match n.children.first() {
                Some(c) if c.kind == NodeKind::ExprCall => self.call_stmt(c, out),
                Some(c) if c.kind == NodeKind::ExprReturn => self.stmt(c, out),
                Some(c) => {
                    self.see(c);
                    let k = kind_name(c);
                    self.reject(&k, "expression statement".into())
                }
                None => self.reject("StmtExpr", "empty statement".into()),
            },
            NodeKind::ExprCall => self.call_stmt(n, out),
            NodeKind::Module if n.name.is_empty() || n.name == "block" => {
                let b = self.block(n)?;
                out.extend(b);
                Ok(())
            }
            _ => {
                let k = kind_name(n);
                self.reject(&k, String::new())
            }
        }
    }

    fn local(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        let name = n.name.clone();
        if name.is_empty() || name.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            return self.reject("StmtLocal", format!("binding `{}`", name));
        }
        let ann = n.extra_type.trim().to_string();
        let mutable = n.extra_mutable;
        if !ann.is_empty() {
            let ty = self.ty(&ann)?;
            let value = match n.children.first() {
                Some(init) => {
                    let v = self.expr(init)?;
                    self.coerce(v, ty)?
                }
                None => Expr {
                    ty,
                    kind: ExprKind::Const(0),
                },
            };
            let id = self.new_var(&name, ty, mutable);
            out.push(Stmt::Assign { var: id, value });
            return Ok(());
        }
        let init = match n.children.first() {
            Some(i) => i,
            None => return self.reject("StmtLocal", format!("`{}` has neither type nor value", name)),
        };
        let v = self.expr(init)?;
        match v {
            Val::Ct(c) => {
                if mutable {
                    return self.reject(
                        "StmtLocal",
                        format!("`var {}` initialised with an untyped integer needs a type", name),
                    );
                }
                self.bind(&name, Binding::Const(Val::Ct(c)));
            }
            Val::E(e) => {
                let id = self.new_var(&name, e.ty, mutable);
                out.push(Stmt::Assign { var: id, value: e });
            }
        }
        Ok(())
    }

    fn assign(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        if n.children.len() != 2 {
            return self.reject("StmtAssign", "unexpected shape".into());
        }
        let target = &n.children[0];
        if target.kind != NodeKind::ExprIdentifier {
            let k = kind_name(target);
            return self.reject(&k, "assignment target".into());
        }
        let name = target.name.clone();
        let op = n.extra_op.as_str();
        match self.lookup(&name) {
            Some(Binding::Var { id, mutable }) => {
                if !mutable {
                    return self.reject("StmtAssign", format!("assignment to constant `{}`", name));
                }
                let ty = self.vars[id as usize].ty;
                let rhs = self.expr(&n.children[1])?;
                let v = if op.is_empty() || op == "=" {
                    rhs
                } else {
                    let bin = match op.strip_suffix('=') {
                        Some(b) if !b.is_empty() => b.to_string(),
                        _ => return self.reject("StmtAssign", format!("operator `{}`", op)),
                    };
                    let cur = Val::E(Expr {
                        ty,
                        kind: ExprKind::Var(id),
                    });
                    self.binary(&bin, cur, rhs)?
                };
                let value = self.coerce(v, ty)?;
                out.push(Stmt::Assign { var: id, value });
                Ok(())
            }
            Some(Binding::Const(_)) => {
                self.reject("StmtAssign", format!("assignment to constant `{}`", name))
            }
            None if self.in_test && (op.is_empty() || op == "=") && self.scopes.len() == 1 => {
                // Test-block binding form: the first plain assignment to a
                // fresh name declares it (t27c's Zig backend emits `const`, or
                // `var` when the test assigns the name again).
                let assigned = self.test_assigns.get(&name).copied().unwrap_or(0);
                let v = self.expr(&n.children[1])?;
                match v {
                    Val::Ct(c) if assigned <= 1 => {
                        self.bind(&name, Binding::Const(Val::Ct(c)));
                    }
                    Val::Ct(_) => {
                        return self.reject(
                            "StmtAssign",
                            format!("test binding `{}` reassigned but its type is unknown", name),
                        )
                    }
                    Val::E(e) => {
                        let id = self.new_var(&name, e.ty, assigned > 1);
                        out.push(Stmt::Assign { var: id, value: e });
                    }
                }
                Ok(())
            }
            None => self.reject("StmtAssign", format!("assignment to undeclared `{}`", name)),
        }
    }

    fn call_stmt(&mut self, c: &Node, out: &mut Vec<Stmt>) -> R<()> {
        self.see(c);
        match c.name.as_str() {
            "assert" => {
                if c.children.len() != 1 {
                    return self.reject("ExprCall", format!("assert with {} arguments", c.children.len()));
                }
                let cond = self.cond(&c.children[0])?;
                let site = self.site(TrapKind::Assert, "assert".into(), Ty::Bool);
                out.push(Stmt::Assert { cond, site });
                Ok(())
            }
            "assert_eq" => {
                if c.children.len() != 2 {
                    return self.reject(
                        "ExprCall",
                        format!("assert_eq with {} arguments", c.children.len()),
                    );
                }
                let a = self.expr(&c.children[0])?;
                let b = self.expr(&c.children[1])?;
                let (lhs, rhs) = match (a, b) {
                    (Val::Ct(x), Val::Ct(y)) => {
                        let ty = if Ty::I64.fits(x) && Ty::I64.fits(y) {
                            Ty::I64
                        } else if Ty::U64.fits(x) && Ty::U64.fits(y) {
                            Ty::U64
                        } else {
                            return self.reject("ExprCall", "assert_eq on out-of-range literals".into());
                        };
                        (
                            Expr { ty, kind: ExprKind::Const(x) },
                            Expr { ty, kind: ExprKind::Const(y) },
                        )
                    }
                    (a, b) => self.peer(a, b, "assert_eq")?,
                };
                let site = self.site(TrapKind::AssertEq, "assert_eq".into(), lhs.ty);
                out.push(Stmt::AssertEq { lhs, rhs, site });
                Ok(())
            }
            _ => {
                let (func, args, _ret) = self.call(c)?;
                out.push(Stmt::Eval(Expr {
                    ty: Ty::Bool,
                    kind: ExprKind::Call { func, args },
                }));
                Ok(())
            }
        }
    }

    fn call(&mut self, c: &Node) -> R<(FuncId, Vec<Expr>, Option<Ty>)> {
        self.see(c);
        let (id, params, ret) = match self.sigs.get(&c.name) {
            Some(s) => (s.id, s.params.clone(), s.ret),
            None => {
                let what = if c.name.starts_with('@') {
                    format!("ExprCall({})", c.name)
                } else if c.name == "assert" || c.name == "assert_eq" {
                    "ExprCall(assert in expression)".to_string()
                } else {
                    "ExprCall(unresolved fn)".to_string()
                };
                return self.reject(&what, format!("call to `{}`", c.name));
            }
        };
        if c.children.len() != params.len() {
            return self.reject(
                "ExprCall",
                format!(
                    "`{}` takes {} arguments, {} given",
                    c.name,
                    params.len(),
                    c.children.len()
                ),
            );
        }
        let mut args = Vec::new();
        for (i, a) in c.children.iter().enumerate() {
            let v = self.expr(a)?;
            args.push(self.coerce(v, params[i])?);
        }
        Ok((id, args, ret))
    }

    // ----------------------------------------------------------- expressions

    fn cond(&mut self, n: &Node) -> R<Expr> {
        let v = self.expr(n)?;
        match v {
            Val::E(e) if e.ty == Ty::Bool => Ok(e),
            Val::E(e) => self.reject("condition", format!("expected bool, found {}", e.ty.name())),
            Val::Ct(_) => self.reject("condition", "expected bool, found an integer literal".into()),
        }
    }

    fn coerce(&mut self, v: Val, to: Ty) -> R<Expr> {
        match v {
            Val::Ct(c) => {
                if !to.is_int() {
                    return self.reject("type mismatch", "integer literal where bool is expected".into());
                }
                if !to.fits(c) {
                    return self.reject(
                        "literal out of range",
                        format!("{} does not fit in {}", c, to.name()),
                    );
                }
                Ok(Expr {
                    ty: to,
                    kind: ExprKind::Const(c),
                })
            }
            Val::E(e) => {
                if e.ty == to {
                    return Ok(e);
                }
                if to.can_widen_from(e.ty) {
                    if let ExprKind::Const(c) = e.kind {
                        return Ok(Expr {
                            ty: to,
                            kind: ExprKind::Const(c),
                        });
                    }
                    return Ok(Expr {
                        ty: to,
                        kind: ExprKind::Widen(Box::new(e)),
                    });
                }
                self.reject(
                    "type mismatch",
                    format!("expected {}, found {}", to.name(), e.ty.name()),
                )
            }
        }
    }

    /// Bring two operands to one type (Zig peer type resolution, integers).
    fn peer(&mut self, a: Val, b: Val, what: &str) -> R<(Expr, Expr)> {
        match (a, b) {
            (Val::E(x), Val::Ct(c)) => {
                let t = x.ty;
                let y = self.coerce(Val::Ct(c), t)?;
                Ok((x, y))
            }
            (Val::Ct(c), Val::E(y)) => {
                let t = y.ty;
                let x = self.coerce(Val::Ct(c), t)?;
                Ok((x, y))
            }
            (Val::E(x), Val::E(y)) => {
                if x.ty == y.ty {
                    return Ok((x, y));
                }
                if x.ty.can_widen_from(y.ty) {
                    let t = x.ty;
                    let y = self.coerce(Val::E(y), t)?;
                    return Ok((x, y));
                }
                if y.ty.can_widen_from(x.ty) {
                    let t = y.ty;
                    let x = self.coerce(Val::E(x), t)?;
                    return Ok((x, y));
                }
                self.reject(
                    "type mismatch",
                    format!("`{}` on {} and {}", what, x.ty.name(), y.ty.name()),
                )
            }
            (Val::Ct(_), Val::Ct(_)) => self.reject("type mismatch", "internal: two literals".into()),
        }
    }

    fn expr(&mut self, n: &Node) -> R<Val> {
        self.see(n);
        match n.kind {
            NodeKind::ExprLiteral => self.literal(n),
            NodeKind::ExprIdentifier => {
                let name = n.name.as_str();
                if name == "true" || name == "false" {
                    return Ok(Val::E(Expr {
                        ty: Ty::Bool,
                        kind: ExprKind::Const((name == "true") as i128),
                    }));
                }
                match self.lookup(name) {
                    Some(Binding::Var { id, .. }) => Ok(Val::E(Expr {
                        ty: self.vars[id as usize].ty,
                        kind: ExprKind::Var(id),
                    })),
                    Some(Binding::Const(v)) => Ok(v),
                    None => match self.global(name)? {
                        Some(v) => Ok(v),
                        None => self.reject(
                            "ExprIdentifier",
                            format!("unknown name `{}`", name),
                        ),
                    },
                }
            }
            NodeKind::ExprBinary => {
                if n.children.len() != 2 {
                    return self.reject("ExprBinary", "unexpected shape".into());
                }
                let op = n.extra_op.clone();
                if op == "and" || op == "or" || op == "&&" || op == "||" {
                    let a = self.cond(&n.children[0])?;
                    let b = self.cond(&n.children[1])?;
                    let and = op == "and" || op == "&&";
                    if let (ExprKind::Const(x), ExprKind::Const(y)) = (&a.kind, &b.kind) {
                        let r = if and { *x & *y } else { *x | *y };
                        return Ok(Val::E(Expr { ty: Ty::Bool, kind: ExprKind::Const(r) }));
                    }
                    let kind = if and {
                        ExprKind::And(Box::new(a), Box::new(b))
                    } else {
                        ExprKind::Or(Box::new(a), Box::new(b))
                    };
                    return Ok(Val::E(Expr { ty: Ty::Bool, kind }));
                }
                let a = self.expr(&n.children[0])?;
                let b = self.expr(&n.children[1])?;
                self.binary(&op, a, b)
            }
            NodeKind::ExprUnary => {
                if n.children.len() != 1 {
                    return self.reject("ExprUnary", "unexpected shape".into());
                }
                let op = n.extra_op.clone();
                let v = self.expr(&n.children[0])?;
                self.unary(&op, v)
            }
            NodeKind::ExprCall => {
                let (func, args, ret) = self.call(n)?;
                match ret {
                    Some(ty) => Ok(Val::E(Expr {
                        ty,
                        kind: ExprKind::Call { func, args },
                    })),
                    None => self.reject("ExprCall", format!("void fn `{}` used as a value", n.name)),
                }
            }
            _ => {
                let k = kind_name(n);
                self.reject(&k, String::new())
            }
        }
    }

    fn literal(&mut self, n: &Node) -> R<Val> {
        let s = n.value.trim();
        if s == "true" || s == "false" {
            return Ok(Val::E(Expr {
                ty: Ty::Bool,
                kind: ExprKind::Const((s == "true") as i128),
            }));
        }
        let v = match parse_int(s) {
            Some(v) => v,
            None => {
                // The t27c parser strips a string literal's quotes and marks
                // the node with extra_kind "string" instead.
                let what = if n.extra_kind == "string" || s.starts_with('"') {
                    "string literal"
                } else if s.starts_with('\'') {
                    "char literal"
                } else if s.contains('.') || (s.contains(['e', 'E']) && !s.starts_with("0x")) {
                    "float literal"
                } else {
                    "literal"
                };
                return self.reject(&format!("ExprLiteral({})", what), format!("`{}`", s));
            }
        };
        let suffix = n.extra_type.trim();
        if suffix.is_empty() {
            Ok(Val::Ct(v))
        } else {
            let ty = self.ty(suffix)?;
            Ok(Val::E(self.coerce(Val::Ct(v), ty)?))
        }
    }

    fn unary(&mut self, op: &str, v: Val) -> R<Val> {
        match (op, v) {
            ("-", Val::Ct(c)) => Ok(Val::Ct(-c)),
            ("-", Val::E(e)) => {
                if !e.ty.is_int() || !e.ty.signed() {
                    return self.reject(
                        "ExprUnary(-)",
                        format!("negation of {}", e.ty.name()),
                    );
                }
                let ty = e.ty;
                let zero = Expr { ty, kind: ExprKind::Const(0) };
                let op = if self.mode == OverflowMode::Trap { ArithOp::Sub } else { ArithOp::SubW };
                let site = if op == ArithOp::Sub {
                    self.site(TrapKind::Overflow, format!("- on {}", ty.name()), ty)
                } else {
                    0
                };
                Ok(Val::E(Expr {
                    ty,
                    kind: ExprKind::Arith { op, lhs: Box::new(zero), rhs: Box::new(e), site },
                }))
            }
            ("!", Val::E(e)) if e.ty == Ty::Bool => {
                if let ExprKind::Const(c) = e.kind {
                    return Ok(Val::E(Expr { ty: Ty::Bool, kind: ExprKind::Const(1 - c) }));
                }
                Ok(Val::E(Expr { ty: Ty::Bool, kind: ExprKind::Not(Box::new(e)) }))
            }
            ("~", Val::E(e)) if e.ty.is_int() => {
                let ty = e.ty;
                Ok(Val::E(Expr { ty, kind: ExprKind::BitNot(Box::new(e)) }))
            }
            (op, Val::E(e)) => self.reject(
                &format!("ExprUnary({})", op),
                format!("operand of type {}", e.ty.name()),
            ),
            (op, Val::Ct(_)) => self.reject(
                &format!("ExprUnary({})", op),
                "on an untyped integer literal".into(),
            ),
        }
    }

    fn binary(&mut self, op: &str, a: Val, b: Val) -> R<Val> {
        let cmp = match op {
            "==" => Some(CmpOp::Eq),
            "!=" => Some(CmpOp::Ne),
            "<" => Some(CmpOp::Lt),
            "<=" => Some(CmpOp::Le),
            ">" => Some(CmpOp::Gt),
            ">=" => Some(CmpOp::Ge),
            _ => None,
        };
        if let Some(c) = cmp {
            return self.compare(c, a, b);
        }
        let trap = self.mode == OverflowMode::Trap;
        let aop = match op {
            "+" => if trap { ArithOp::Add } else { ArithOp::AddW },
            "-" => if trap { ArithOp::Sub } else { ArithOp::SubW },
            "*" => if trap { ArithOp::Mul } else { ArithOp::MulW },
            "+%" => ArithOp::AddW,
            "-%" => ArithOp::SubW,
            "*%" => ArithOp::MulW,
            "/" => if trap { ArithOp::Div } else { ArithOp::DivW },
            "%" => ArithOp::Rem,
            "&" => ArithOp::And,
            "|" => ArithOp::Or,
            "^" => ArithOp::Xor,
            "<<" => if trap { ArithOp::Shl } else { ArithOp::ShlW },
            ">>" => if trap { ArithOp::Shr } else { ArithOp::ShrW },
            _ => return self.reject(&format!("ExprBinary({})", op), String::new()),
        };
        if aop.is_shift() {
            return self.shift(aop, a, b);
        }
        if let (Val::Ct(x), Val::Ct(y)) = (&a, &b) {
            return self.fold(aop, *x, *y).map(Val::Ct);
        }
        let (x, y) = self.peer(a, b, op)?;
        let ty = x.ty;
        let bitwise = matches!(aop, ArithOp::And | ArithOp::Or | ArithOp::Xor);
        if !ty.is_int() && !bitwise {
            return self.reject(&format!("ExprBinary({})", op), "on bool".into());
        }
        let site = match aop {
            ArithOp::Add | ArithOp::Sub | ArithOp::Mul => {
                self.site(TrapKind::Overflow, format!("{} on {}", op, ty.name()), ty)
            }
            ArithOp::Div => {
                // Two consecutive sites: divisor zero, then MIN / -1.
                let s = self.site(TrapKind::DivZero, format!("/ on {}", ty.name()), ty);
                self.site(TrapKind::Overflow, format!("/ on {}", ty.name()), ty);
                s
            }
            ArithOp::DivW | ArithOp::Rem => {
                self.site(TrapKind::DivZero, format!("{} on {}", op, ty.name()), ty)
            }
            _ => 0,
        };
        Ok(Val::E(Expr {
            ty,
            kind: ExprKind::Arith { op: aop, lhs: Box::new(x), rhs: Box::new(y), site },
        }))
    }

    fn fold(&mut self, op: ArithOp, x: i128, y: i128) -> R<i128> {
        let r = match op {
            ArithOp::Add | ArithOp::AddW => x.checked_add(y),
            ArithOp::Sub | ArithOp::SubW => x.checked_sub(y),
            ArithOp::Mul | ArithOp::MulW => x.checked_mul(y),
            ArithOp::Div | ArithOp::DivW | ArithOp::Rem => {
                if y == 0 {
                    return self.reject("ExprBinary", "constant division by zero".into());
                }
                if op == ArithOp::Rem { x.checked_rem(y) } else { x.checked_div(y) }
            }
            ArithOp::And => Some(x & y),
            ArithOp::Or => Some(x | y),
            ArithOp::Xor => Some(x ^ y),
            _ => None,
        };
        match r {
            Some(v) => Ok(v),
            None => self.reject("ExprBinary", "constant expression overflows 128 bits".into()),
        }
    }

    fn shift(&mut self, op: ArithOp, a: Val, b: Val) -> R<Val> {
        let left = matches!(op, ArithOp::Shl | ArithOp::ShlW);
        match (a, b) {
            (Val::Ct(x), Val::Ct(y)) => {
                if !(0..127).contains(&y) {
                    return self.reject("ExprBinary", format!("constant shift by {}", y));
                }
                if left {
                    let r = x << y;
                    if (r >> y) != x {
                        return self.reject("ExprBinary", "constant shift overflows 128 bits".into());
                    }
                    Ok(Val::Ct(r))
                } else {
                    Ok(Val::Ct(x >> y))
                }
            }
            (Val::Ct(_), Val::E(_)) => self.reject(
                "ExprBinary(<< >>)",
                "untyped literal shifted by a runtime amount".into(),
            ),
            (Val::E(x), amt) => {
                let ty = x.ty;
                if !ty.is_int() {
                    return self.reject("ExprBinary(<< >>)", format!("shift of {}", ty.name()));
                }
                let bits = ty.bits() as i128;
                match amt {
                    Val::Ct(c) => {
                        let c = if (0..bits).contains(&c) {
                            c
                        } else if self.mode == OverflowMode::Wrap {
                            c & (bits - 1)
                        } else {
                            return self.reject(
                                "ExprBinary(<< >>)",
                                format!("shift amount {} out of range for {}", c, ty.name()),
                            );
                        };
                        let wop = if left { ArithOp::ShlW } else { ArithOp::ShrW };
                        let amt = Expr { ty: Ty::U32, kind: ExprKind::Const(c) };
                        Ok(Val::E(Expr {
                            ty,
                            kind: ExprKind::Arith { op: wop, lhs: Box::new(x), rhs: Box::new(amt), site: 0 },
                        }))
                    }
                    Val::E(y) => {
                        if !y.ty.is_int() {
                            return self.reject("ExprBinary(<< >>)", "bool shift amount".into());
                        }
                        let site = if matches!(op, ArithOp::Shl | ArithOp::Shr) {
                            self.site(TrapKind::ShiftRange, format!("{} on {}", op.symbol(), ty.name()), ty)
                        } else {
                            0
                        };
                        Ok(Val::E(Expr {
                            ty,
                            kind: ExprKind::Arith { op, lhs: Box::new(x), rhs: Box::new(y), site },
                        }))
                    }
                }
            }
        }
    }

    fn compare(&mut self, op: CmpOp, a: Val, b: Val) -> R<Val> {
        if let (Val::Ct(x), Val::Ct(y)) = (&a, &b) {
            return Ok(Val::E(Expr {
                ty: Ty::Bool,
                kind: ExprKind::Const(op.holds(*x, *y) as i128),
            }));
        }
        let (x, y) = self.peer(a, b, op.symbol())?;
        if x.ty == Ty::Bool && !matches!(op, CmpOp::Eq | CmpOp::Ne) {
            return self.reject(&format!("ExprBinary({})", op.symbol()), "ordering on bool".into());
        }
        Ok(Val::E(Expr {
            ty: Ty::Bool,
            kind: ExprKind::Cmp { op, lhs: Box::new(x), rhs: Box::new(y) },
        }))
    }
}

/// Count plain assignments per identifier in a test body (all nesting levels).
fn count_assigns(ns: &[Node], counts: &mut HashMap<String, u32>) {
    for n in ns {
        if n.kind == NodeKind::StmtAssign {
            if let Some(t) = n.children.first() {
                if t.kind == NodeKind::ExprIdentifier {
                    *counts.entry(t.name.clone()).or_insert(0) += 1;
                }
            }
        }
        count_assigns(&n.children, counts);
    }
}

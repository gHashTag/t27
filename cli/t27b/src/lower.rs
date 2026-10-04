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
//!
//! Memory: a struct value lives in memory, never in a register -- a frame
//! slot for a local or a temporary, the caller's memory for a parameter (passed
//! as a pointer, never written), read-only data for a module constant. A
//! function returning a struct takes a hidden last pointer parameter to the
//! caller's result slot, builds the value there and returns that pointer.
//! Struct layout is C's: fields in order, each at its own alignment.

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

/// A lowered expression: a comptime integer (exact, untyped), a typed
/// scalar IR expression, a pointer (`Ty::Ptr` expression and its `LTy::Ptr`
/// type), or a struct in memory.
#[derive(Clone, Debug)]
enum Val {
    Ct(i128),
    E(Expr),
    P(Expr, LTy),
    M(Place),
    /// Recovery mode only (`blockers`): the value of an expression that was
    /// already rejected. Anything built from it is dropped without a second
    /// report, so one unsupported construct is named once, not once per use.
    Poison,
}

impl Val {
    fn is_poison(&self) -> bool {
        matches!(self, Val::Poison)
    }
}

#[derive(Clone, Debug)]
enum Binding {
    Var { id: VarId, mutable: bool },
    Const(Val),
    /// A variable that lives in memory: a struct, or a scalar whose address
    /// is taken.
    Mem(Place),
}

/// A source type: a scalar or a pointer (one register), or a struct.
#[derive(Clone, Debug, PartialEq, Eq)]
enum LTy {
    S(Ty),
    /// `*T` (true: mutable) or `*const T` (false).
    Ptr(Box<LTy>, bool),
    Struct(u32),
}

/// `ty` at `addr + off`. `addr` is evaluated exactly once by whatever
/// consumes the place, so it may be a call or a `Seq`.
#[derive(Clone, Debug)]
struct Place {
    addr: Expr,
    off: u32,
    ty: LTy,
    mutable: bool,
    /// The frame slot this place is the whole of, when it is a temporary
    /// nothing else refers to (a struct literal, a call's result).
    temp: Option<u32>,
}

#[derive(Clone, Debug)]
struct Field<'a> {
    name: String,
    ty: LTy,
    off: u32,
    default: Option<&'a Node>,
}

struct StructDef<'a> {
    name: String,
    fields: Vec<Field<'a>>,
    /// None while the layout is being computed (or after it failed).
    size: Option<u32>,
    align: u32,
    /// Why the layout failed, to report again at every later use.
    fail: Option<Reject>,
}

struct Sig {
    id: FuncId,
    params: Vec<LTy>,
    ret: Option<LTy>,
    /// Recovery mode only: the signature named a type outside the subset.
    /// Calls to it are dropped silently; its body is still lowered.
    poisoned: bool,
}

struct Lower<'a> {
    mode: OverflowMode,
    sites: Vec<Site>,
    errors: Vec<Reject>,
    sigs: HashMap<String, Sig>,
    globals: HashMap<String, Val>,
    const_nodes: HashMap<String, &'a Node>,
    resolving: HashSet<String>,
    struct_nodes: HashMap<String, &'a Node>,
    structs: Vec<StructDef<'a>>,
    struct_ids: HashMap<String, u32>,
    data: Vec<Vec<u8>>,
    internal_abi: Vec<FuncId>,
    // Per-function state.
    vars: Vec<Var>,
    /// The source type of each variable (parallel to `vars`).
    ltys: Vec<LTy>,
    slots: Vec<SlotInfo>,
    /// Names whose address is taken somewhere in the body (`&x`): such a
    /// scalar lives in a frame slot instead of a register.
    addr_taken: HashSet<String>,
    /// The hidden result pointer of a function returning a struct.
    sret: Option<VarId>,
    scopes: Vec<HashMap<String, Binding>>,
    loop_depth: u32,
    line: u32,
    ret: Option<LTy>,
    in_test: bool,
    test_assigns: HashMap<String, u32>,
    /// The source text, when known: a clause-form `invariant` (and a
    /// braceless `test`) reaches lowering with no line on any of its nodes, so
    /// its header line is looked up here instead.
    src: Option<&'a str>,
    // Recovery mode (`blockers`): keep lowering after a rejection so every
    // unsupported construct in the file is named, not only the first per item.
    recover: bool,
    /// Names declared by a rejected top-level item (a struct, an enum, a const
    /// outside the subset): a use of one is not reported again.
    poison_names: HashSet<String>,
    /// Names of top-level struct and enum declarations, so a type that names
    /// one is reported as `type (struct)` / `type (enum)`.
    type_decls: HashMap<String, &'static str>,
    /// The current fn's return type was rejected.
    ret_poison: bool,
}

/// Lower a parsed module. All rejected constructs are returned (at most one per
/// top-level item, since lowering of an item stops at its first rejection).
pub fn lower(ast: &Node, mode: OverflowMode) -> Result<Program, Vec<Reject>> {
    lower_src(ast, mode, None)
}

/// `lower`, with the source text the AST was parsed from, for line numbers.
pub fn lower_src<'a>(ast: &'a Node, mode: OverflowMode, src: Option<&'a str>) -> Result<Program, Vec<Reject>> {
    lower_mode(ast, mode, src, false)
}

/// Every construct outside the subset in a module, not only the first per
/// item: lowering continues past each rejection (statement by statement, and
/// operand by operand inside an expression), and a value built from a rejected
/// one is dropped without a second report. Empty when the module lowers.
///
/// A rejected top-level declaration (a `struct`, an `enum`) is still one
/// entry: its members are not looked into, so a file's list is a lower bound.
pub fn blockers(ast: &Node, mode: OverflowMode) -> Vec<Reject> {
    blockers_src(ast, mode, None)
}

/// `blockers`, with the source text the AST was parsed from, for line numbers.
pub fn blockers_src<'a>(ast: &'a Node, mode: OverflowMode, src: Option<&'a str>) -> Vec<Reject> {
    match lower_mode(ast, mode, src, true) {
        Ok(_) => Vec::new(),
        Err(r) => r,
    }
}

fn lower_mode<'a>(
    ast: &'a Node,
    mode: OverflowMode,
    src: Option<&'a str>,
    recover: bool,
) -> Result<Program, Vec<Reject>> {
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
        struct_nodes: HashMap::new(),
        structs: Vec::new(),
        struct_ids: HashMap::new(),
        data: Vec::new(),
        internal_abi: Vec::new(),
        vars: Vec::new(),
        ltys: Vec::new(),
        slots: Vec::new(),
        addr_taken: HashSet::new(),
        sret: None,
        scopes: Vec::new(),
        loop_depth: 0,
        line: ast.line,
        ret: None,
        in_test: false,
        test_assigns: HashMap::new(),
        src,
        recover,
        poison_names: HashSet::new(),
        type_decls: HashMap::new(),
        ret_poison: false,
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

    // Struct declarations are laid out on first use, like Zig's lazy
    // analysis: an unused struct with an unsupported member rejects nothing.
    for item in &items {
        if item.kind == NodeKind::StructDecl {
            l.struct_nodes.insert(item.name.clone(), item);
        }
    }

    // Pass 1: signatures and constant declarations.
    let mut fn_nodes: Vec<&Node> = Vec::new();
    let mut next_id: FuncId = 0;
    for item in &items {
        match item.kind {
            NodeKind::StructDecl => {
                l.type_decls.insert(item.name.clone(), "struct");
            }
            NodeKind::EnumDecl => {
                l.type_decls.insert(item.name.clone(), "enum");
            }
            // A constant used where a type goes is a type alias.
            NodeKind::ConstDecl => {
                l.type_decls.insert(item.name.clone(), "alias");
            }
            _ => {}
        }
    }
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
                        let agg = |t: &LTy| matches!(t, LTy::Struct(_));
                        if params.iter().any(agg) || ret.as_ref().is_some_and(agg) {
                            l.internal_abi.push(next_id);
                        }
                        l.sigs.insert(
                            item.name.clone(),
                            Sig {
                                id: next_id,
                                params,
                                ret,
                                poisoned: false,
                            },
                        );
                        next_id += 1;
                        fn_nodes.push(item);
                    }
                    Err(()) if l.recover => {
                        // Keep the body: what it contains is reported too.
                        let n = item.params.len();
                        l.sigs.insert(
                            item.name.clone(),
                            Sig {
                                id: next_id,
                                params: vec![LTy::S(Ty::Bool); n],
                                ret: None,
                                poisoned: true,
                            },
                        );
                        next_id += 1;
                        fn_nodes.push(item);
                    }
                    Err(()) => {
                        l.poison_names.insert(item.name.clone());
                    }
                }
            }
            NodeKind::TestBlock | NodeKind::InvariantBlock | NodeKind::StructDecl => {}
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
                if !item.name.is_empty() {
                    l.poison_names.insert(item.name.clone());
                }
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
    let mut unchecked = Vec::new();
    for item in &items {
        match item.kind {
            NodeKind::TestBlock => {
                if let Ok(f) = l.test(item, false) {
                    funcs.push(f);
                }
            }
            // An invariant with no lowered body is one whose clause the
            // front-end discarded (t27c's Zig backend writes `NOT CHECKED`
            // for exactly this case): there is nothing to run, and running
            // nothing must not be reported as the invariant holding.
            NodeKind::InvariantBlock if item.children.is_empty() && item.extra_field != "partial" => {
                unchecked.push(item.name.clone());
            }
            NodeKind::InvariantBlock => {
                if let Ok(f) = l.test(item, true) {
                    funcs.push(f);
                }
            }
            _ => {}
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
        unchecked,
        data: l.data,
        internal_abi: l.internal_abi,
    })
}

/// Line (1-based) of the first `<keyword> <name>` header in `src`, the name
/// optionally quoted.
fn header_line(src: &str, keyword: &str, name: &str) -> Option<u32> {
    for (i, line) in src.lines().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix(keyword) else { continue };
        let Some(rest) = rest.strip_prefix(|c: char| c == ' ' || c == '\t') else { continue };
        let rest = rest.trim_start();
        let rest = rest.strip_prefix('"').unwrap_or(rest);
        if let Some(after) = rest.strip_prefix(name) {
            if !after.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
                return Some(i as u32 + 1);
            }
        }
    }
    None
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
            None => {
                let (construct, detail) = self.type_construct(t);
                self.reject(&construct, detail)
            }
        }
    }

    /// The construct a type outside the subset is reported as. Types of one
    /// shape share a name (`type [N]T`, `type []T`, `type (struct)`), so the
    /// count says how many files need that shape; the detail names the type.
    fn type_construct(&self, t: &str) -> (String, String) {
        let shape = if let Some(k) = self.type_decls.get(t) {
            format!("type ({})", k)
        } else if t.starts_with("[]") {
            "type []T".to_string()
        } else if t.starts_with('[') {
            "type [N]T".to_string()
        } else if t.starts_with('(') {
            "type (tuple)".to_string()
        } else if t.starts_with('?') {
            "type ?T".to_string()
        } else if t.starts_with('*') {
            "type *T".to_string()
        } else if t.contains('!') {
            "type E!T".to_string()
        } else if t.starts_with("struct") {
            "type (anonymous struct)".to_string()
        } else if t.contains('(') {
            "type (generic)".to_string()
        } else if t.starts_with(|c: char| c.is_ascii_uppercase()) && !matches!(t, "Result" | "Option") {
            // Declared nowhere this file can see: an import `use` did not
            // splice, or a type of a sibling spec.
            "type (undeclared)".to_string()
        } else {
            return (format!("type {}", t), String::new());
        };
        (shape, format!("`{}`", t))
    }

    fn signature(&mut self, n: &Node) -> R<(Vec<LTy>, Option<LTy>)> {
        let mut bad = false;
        let mut params = Vec::new();
        for (pname, pty) in &n.params {
            let r = if pname.starts_with("comptime ") || pty.is_empty() {
                self.reject("FnDecl", format!("parameter `{}` of `{}`", pname, n.name))
            } else {
                self.lty(pty)
            };
            match r {
                Ok(t) => params.push(t),
                // Recovery mode reports every parameter and the return type.
                Err(()) if self.recover => bad = true,
                Err(()) => return Err(()),
            }
        }
        let rt = n.extra_return_type.trim();
        let ret = if rt.is_empty() || rt == "void" {
            None
        } else {
            Some(self.lty(rt)?)
        };
        // A struct result is returned through a hidden pointer parameter.
        let total = n.params.len() + matches!(ret, Some(LTy::Struct(_))) as usize;
        if total > 8 {
            return self.reject(
                "FnDecl",
                format!("`{}` has {} parameters, at most 8 are supported", n.name, total),
            );
        }
        if bad {
            return Err(());
        }
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

    /// A register variable of a scalar or pointer type, bound to `name`.
    fn new_lvar(&mut self, name: &str, t: LTy, mutable: bool) -> VarId {
        let id = self.hidden_var(name, t);
        self.bind(name, Binding::Var { id, mutable });
        id
    }

    /// A register variable no source name refers to.
    fn hidden_var(&mut self, name: &str, t: LTy) -> VarId {
        let id = self.vars.len() as VarId;
        let ty = reg_ty(&t).unwrap_or(Ty::Ptr);
        self.vars.push(Var {
            name: name.to_string(),
            ty,
        });
        self.ltys.push(t);
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
        let v = match r {
            Err(()) if self.recover => Val::Poison,
            r => r?,
        };
        self.globals.insert(name.to_string(), v.clone());
        Ok(Some(v))
    }

    fn global_value(&mut self, node: &Node) -> R<Val> {
        let init = match node.children.first() {
            Some(c) => c,
            None => return self.reject("ConstDecl", format!("`{}` has no value", node.name)),
        };
        let ann = node.extra_type.trim();
        let st = if !ann.is_empty() {
            match self.lty(ann)? {
                t @ LTy::Struct(_) => Some(t),
                LTy::Ptr(..) => {
                    return self.reject("ConstDecl", format!("`{}` is a module-level pointer", node.name))
                }
                LTy::S(_) => None,
            }
        } else if init.kind == NodeKind::ExprStructLit && !init.name.is_empty() {
            Some(self.lty(&init.name)?)
        } else {
            None
        };
        if let Some(t) = st {
            return self.rodata(init, t);
        }
        let v = self.expr(init)?;
        let v = if node.extra_type.trim().is_empty() {
            v
        } else {
            let ty = self.ty(&node.extra_type)?;
            Val::E(self.coerce(v, ty)?)
        };
        match &v {
            Val::Ct(_) | Val::Poison => Ok(v),
            Val::E(e) if matches!(e.kind, ExprKind::Const(_)) => Ok(v),
            // Another struct constant: the same read-only bytes.
            Val::M(p) if matches!(p.addr.kind, ExprKind::Data(_)) => Ok(v),
            _ => self.reject(
                "ConstDecl",
                format!("`{}` is not a compile-time integer or bool", node.name),
            ),
        }
    }

    // ------------------------------------------------------------- functions

    fn begin_body(&mut self, body: &[Node]) {
        self.vars.clear();
        self.ltys.clear();
        self.slots.clear();
        self.sret = None;
        self.addr_taken.clear();
        scan_addr_taken(body, &mut self.addr_taken);
        self.scopes.clear();
        self.scopes.push(HashMap::new());
        self.loop_depth = 0;
    }

    fn function(&mut self, n: &Node) -> R<Func> {
        self.see(n);
        self.begin_body(&n.children);
        self.in_test = false;
        let (params, ret, poisoned) = {
            let s = &self.sigs[&n.name];
            (s.params.clone(), s.ret.clone(), s.poisoned)
        };
        self.ret = ret.clone();
        self.ret_poison = false;
        if poisoned {
            return self.poisoned_function(n);
        }
        // Parameters first (vars 0..), then the hidden result pointer, so
        // that they are exactly the first `nparams` variables.
        let mut ids = Vec::new();
        for (i, (pname, _)) in n.params.iter().enumerate() {
            let t = match &params[i] {
                LTy::Struct(_) => LTy::Ptr(Box::new(params[i].clone()), false),
                t => t.clone(),
            };
            ids.push(self.hidden_var(pname, t));
        }
        if matches!(ret, Some(LTy::Struct(_))) {
            self.sret = Some(self.hidden_var("%sret", LTy::Ptr(Box::new(ret.clone().unwrap()), true)));
        }
        let nparams = self.vars.len();
        let mut body = Vec::new();
        for (i, (pname, _)) in n.params.iter().enumerate() {
            let var = Expr { ty: Ty::Ptr, kind: ExprKind::Var(ids[i]) };
            match &params[i] {
                // A struct argument is the caller's memory, read-only.
                LTy::Struct(_) => {
                    let p = Place { addr: var, off: 0, ty: params[i].clone(), mutable: false, temp: None };
                    self.bind(pname, Binding::Mem(p));
                }
                t if self.addr_taken.contains(pname) => {
                    let ty = reg_ty(t).unwrap();
                    let k = self.new_slot(t)?;
                    let value = Expr { ty, kind: ExprKind::Var(ids[i]) };
                    body.push(Stmt::Store { addr: slot_expr(k), off: 0, value });
                    let p = Place { addr: slot_expr(k), off: 0, ty: t.clone(), mutable: false, temp: None };
                    self.bind(pname, Binding::Mem(p));
                }
                _ => self.bind(pname, Binding::Var { id: ids[i], mutable: true }),
            }
        }
        body.extend(self.stmts(&n.children)?);
        let ret = ret.map(|t| reg_ty(&t).unwrap_or(Ty::Ptr));
        let noreturn_site = if ret.is_some() {
            self.site(TrapKind::NoReturn, format!("end of fn {}", n.name), Ty::Bool)
        } else {
            0
        };
        Ok(Func {
            name: n.name.clone(),
            nparams,
            ret,
            vars: std::mem::take(&mut self.vars),
            slots: std::mem::take(&mut self.slots),
            body,
            line: n.line,
            is_test: false,
            is_invariant: false,
            noreturn_site,
        })
    }

    /// Recovery mode: the body of a fn whose signature was rejected, lowered
    /// only for what it reports. A parameter whose type is a scalar of the
    /// subset is a real variable; any other is poison, already reported.
    fn poisoned_function(&mut self, n: &Node) -> R<Func> {
        for (pname, pty) in &n.params {
            match Ty::from_name(pty.trim()) {
                Some(t) if !pname.starts_with("comptime ") => {
                    self.new_lvar(pname, LTy::S(t), true);
                }
                _ => self.bind(pname, Binding::Const(Val::Poison)),
            }
        }
        let rt = n.extra_return_type.trim();
        if !rt.is_empty() && rt != "void" {
            self.ret = Ty::from_name(rt).map(LTy::S);
            self.ret_poison = self.ret.is_none();
        }
        let _ = self.stmts(&n.children)?;
        Err(())
    }

    /// A `test` block, or an `invariant` block: both are a parameterless body
    /// of statements run once, and both use the test binding rule.
    fn test(&mut self, n: &Node, invariant: bool) -> R<Func> {
        self.see(n);
        if n.line == 0 {
            if let Some(l) = self.src.and_then(|s| header_line(s, if invariant { "invariant" } else { "test" }, &n.name)) {
                self.line = l;
            }
        }
        if n.extra_field == "partial" {
            let (k, what) = if invariant { ("InvariantBlock", "invariant") } else { ("TestBlock", "test") };
            return self.reject(
                k,
                format!("{} `{}` was only partially parsed by the front-end", what, n.name),
            );
        }
        self.begin_body(&n.children);
        self.in_test = true;
        self.ret = None;
        self.ret_poison = false;
        self.test_assigns.clear();
        count_assigns(&n.children, &mut self.test_assigns);
        let body = self.stmts(&n.children)?;
        self.in_test = false;
        Ok(Func {
            name: n.name.clone(),
            nparams: 0,
            ret: None,
            vars: std::mem::take(&mut self.vars),
            slots: std::mem::take(&mut self.slots),
            body,
            line: n.line,
            is_test: true,
            is_invariant: invariant,
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
            if self.stmt(n, &mut out).is_err() {
                if !self.recover {
                    return Err(());
                }
                // Recovery mode: a name this statement would have declared is
                // poison from here on, so its uses are not reported again.
                self.poison_declared(n);
            }
        }
        Ok(out)
    }

    fn poison_declared(&mut self, n: &Node) {
        let name = match n.kind {
            NodeKind::StmtLocal => n.name.clone(),
            NodeKind::StmtAssign => match n.children.first() {
                Some(t) if t.kind == NodeKind::ExprIdentifier => t.name.clone(),
                _ => return,
            },
            _ => return,
        };
        let known = if n.kind == NodeKind::StmtLocal {
            self.scopes.last().map_or(false, |s| s.contains_key(&name))
        } else {
            self.lookup(&name).is_some()
        };
        if !name.is_empty() && !known {
            self.bind(&name, Binding::Const(Val::Poison));
        }
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
                if self.ret_poison {
                    // Recovery mode: the return type was already rejected.
                    if let Some(c) = v {
                        let _ = self.expr(c)?;
                    }
                    return Ok(());
                }
                match (self.ret.clone(), v) {
                    (None, None) => out.push(Stmt::Return(None)),
                    (Some(t @ LTy::Struct(_)), Some(c)) => {
                        // Build the result in the caller's memory.
                        let sret = Expr { ty: Ty::Ptr, kind: ExprKind::Var(self.sret.unwrap()) };
                        let dst = Place { addr: sret.clone(), off: 0, ty: t, mutable: true, temp: None };
                        self.init(c, dst, true, out)?;
                        out.push(Stmt::Return(Some(sret)));
                    }
                    (Some(t), Some(c)) => {
                        let v = self.expr_as(c, &t)?;
                        let e = self.reg(v)?;
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
                    let k = if c.kind == NodeKind::ExprUnary && !c.extra_op.is_empty() {
                        format!("ExprUnary({}) statement", c.extra_op.trim())
                    } else {
                        format!("{} statement", kind_name(c))
                    };
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
        let init = n.children.first().filter(|i| !is_undefined(i));
        if !ann.is_empty() {
            let t = self.lty(&ann)?;
            if matches!(t, LTy::Struct(_)) || self.addr_taken.contains(&name) {
                // In memory; the name is bound only after its initializer.
                let k = self.new_slot(&t)?;
                let dst = Place { addr: slot_expr(k), off: 0, ty: t, mutable, temp: None };
                match init {
                    Some(i) => self.init(i, dst.clone(), true, out)?,
                    // A scalar with no value reads as zero, like a register.
                    None if !matches!(dst.ty, LTy::Struct(_)) => {
                        let ty = reg_ty(&dst.ty).unwrap();
                        out.push(Stmt::Store { addr: slot_expr(k), off: 0, value: Expr { ty, kind: ExprKind::Const(0) } });
                    }
                    None => {}
                }
                self.bind(&name, Binding::Mem(dst));
                return Ok(());
            }
            let ty = reg_ty(&t).unwrap();
            let value = match init {
                Some(init) => {
                    let v = self.expr_as(init, &t)?;
                    if v.is_poison() {
                        // Recovery mode: the type is known, so keep the name.
                        self.new_lvar(&name, t, mutable);
                        return Err(());
                    }
                    self.reg(v)?
                }
                None => Expr {
                    ty,
                    kind: ExprKind::Const(0),
                },
            };
            let id = self.new_lvar(&name, t, mutable);
            out.push(Stmt::Assign { var: id, value });
            return Ok(());
        }
        let init = match init {
            Some(i) => i,
            None => return self.reject("StmtLocal", format!("`{}` has neither type nor value", name)),
        };
        let v = self.expr(init)?;
        match v {
            Val::Poison => self.bind(&name, Binding::Const(Val::Poison)),
            Val::Ct(c) => {
                if mutable {
                    return self.reject(
                        "StmtLocal",
                        format!("`var {}` initialised with an untyped integer needs a type", name),
                    );
                }
                self.bind(&name, Binding::Const(Val::Ct(c)));
            }
            v => self.bind_value(&name, v, mutable, out)?,
        }
        Ok(())
    }

    /// Bind `name` to a fresh variable holding `v` (not a comptime integer).
    fn bind_value(&mut self, name: &str, v: Val, mutable: bool, out: &mut Vec<Stmt>) -> R<()> {
        let (t, value) = match v {
            Val::Poison => return Err(()),
            Val::Ct(_) => return self.reject("StmtLocal", format!("`{}` needs a type", name)),
            Val::E(e) => (LTy::S(e.ty), e),
            Val::P(e, t) => (t, e),
            Val::M(p) => {
                let k = match p.temp {
                    // A temporary nobody else sees becomes the variable.
                    Some(k) => {
                        match p.addr.kind {
                            ExprKind::Slot(_) => {}
                            ExprKind::Seq { stmts, .. } => out.extend(stmts),
                            _ => out.push(Stmt::Eval(p.addr)),
                        }
                        k
                    }
                    None => {
                        let k = self.new_slot(&p.ty)?;
                        let dst = Place { addr: slot_expr(k), off: 0, ty: p.ty.clone(), mutable: true, temp: None };
                        self.copy(&dst, p.clone(), out)?;
                        k
                    }
                };
                let dst = Place { addr: slot_expr(k), off: 0, ty: p.ty, mutable, temp: None };
                self.bind(name, Binding::Mem(dst));
                return Ok(());
            }
        };
        if self.addr_taken.contains(name) {
            let k = self.new_slot(&t)?;
            out.push(Stmt::Store { addr: slot_expr(k), off: 0, value });
            let dst = Place { addr: slot_expr(k), off: 0, ty: t, mutable, temp: None };
            self.bind(name, Binding::Mem(dst));
        } else {
            let id = self.new_lvar(name, t, mutable);
            out.push(Stmt::Assign { var: id, value });
        }
        Ok(())
    }

    fn assign(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        if n.children.len() != 2 {
            return self.reject("StmtAssign", "unexpected shape".into());
        }
        let target = &n.children[0];
        let op = n.extra_op.as_str();
        if target.kind == NodeKind::ExprFieldAccess {
            let dst = self.lvalue(target)?;
            return self.store(dst, op, &n.children[1], out);
        }
        if target.kind != NodeKind::ExprIdentifier {
            let k = kind_name(target);
            return self.reject(&k, "assignment target".into());
        }
        let name = target.name.clone();
        match self.lookup(&name) {
            Some(Binding::Mem(dst)) => self.store(dst, op, &n.children[1], out),
            Some(Binding::Var { id, mutable }) if !matches!(self.ltys[id as usize], LTy::S(_)) => {
                if !mutable {
                    return self.reject("StmtAssign", format!("assignment to constant `{}`", name));
                }
                if !(op.is_empty() || op == "=") {
                    return self.reject("StmtAssign", format!("`{}` on a pointer", op));
                }
                let t = self.ltys[id as usize].clone();
                let v = self.expr_as(&n.children[1], &t)?;
                let value = self.reg(v)?;
                out.push(Stmt::Assign { var: id, value });
                Ok(())
            }
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
            // A name whose declaration was rejected: its cascade.
            Some(Binding::Const(Val::Poison)) => {
                let _ = self.expr(&n.children[1]);
                Err(())
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
                    Val::Poison => self.bind(&name, Binding::Const(Val::Poison)),
                    Val::Ct(c) if assigned <= 1 => {
                        self.bind(&name, Binding::Const(Val::Ct(c)));
                    }
                    Val::Ct(_) => {
                        return self.reject(
                            "StmtAssign",
                            format!("test binding `{}` reassigned but its type is unknown", name),
                        )
                    }
                    v => self.bind_value(&name, v, assigned > 1, out)?,
                }
                Ok(())
            }
            None => self.reject("StmtAssign(undeclared)", format!("assignment to undeclared `{}`", name)),
        }
    }

    fn call_stmt(&mut self, c: &Node, out: &mut Vec<Stmt>) -> R<()> {
        self.see(c);
        match c.name.as_str() {
            "assert" => {
                if c.children.len() != 1 {
                    return self.reject("ExprCall(assert with message)", format!("assert with {} arguments", c.children.len()));
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
                if a.is_poison() || b.is_poison() {
                    return Err(());
                }
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
                let (call, _, _) = self.call(c, None)?;
                out.push(Stmt::Eval(call));
                Ok(())
            }
        }
    }

    /// A call: the `Call` expression (typed `Bool` for a void fn), the
    /// result type, and the temporary slot a struct result is built in when
    /// no `sret` destination is given.
    fn call(&mut self, c: &Node, sret: Option<Expr>) -> R<(Expr, Option<LTy>, Option<u32>)> {
        self.see(c);
        let (id, params, ret) = match self.sigs.get(&c.name) {
            Some(s) if s.poisoned => {
                // Recovery mode: report what the arguments contain, then drop
                // the call; its signature was already reported.
                for a in &c.children {
                    let _ = self.expr(a)?;
                }
                return Err(());
            }
            Some(s) => (s.id, s.params.clone(), s.ret.clone()),
            None if self.recover && self.poison_names.contains(&c.name) => {
                for a in &c.children {
                    let _ = self.expr(a)?;
                }
                return Err(());
            }
            // Normal mode: the callee's own rejection is the real blocker.
            None if self.poison_names.contains(&c.name) => {
                return self.reject(
                    "ExprCall(rejected fn)",
                    format!("call to `{}`, whose declaration was rejected", c.name),
                );
            }
            None => {
                let what = if c.name.starts_with('@') {
                    format!("ExprCall({})", c.name)
                } else if c.name == "assert" || c.name == "assert_eq" {
                    "ExprCall(assert in expression)".to_string()
                } else if matches!(c.name.as_str(), "Ok" | "Err" | "Some" | "None") {
                    "ExprCall(Result/Option constructor)".to_string()
                } else if c.name.starts_with("std.") {
                    "ExprCall(std.*)".to_string()
                } else if c.name.contains('.') {
                    "ExprCall(method)".to_string()
                } else if matches!(c.name.as_str(), "len" | "expect") {
                    format!("ExprCall({})", c.name)
                } else {
                    "ExprCall(undeclared fn)".to_string()
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
            let v = self.expr_as(a, &params[i])?;
            args.push(match v {
                // By reference; the callee never writes it.
                Val::M(p) => addr_of(&p),
                v => self.reg(v)?,
            });
        }
        let mut temp = None;
        let ty = match &ret {
            None => Ty::Bool,
            Some(LTy::Struct(_)) => {
                let dst = match sret {
                    Some(d) => d,
                    None => {
                        let k = self.new_slot(ret.as_ref().unwrap())?;
                        temp = Some(k);
                        slot_expr(k)
                    }
                };
                args.push(dst);
                Ty::Ptr
            }
            Some(t) => reg_ty(t).unwrap(),
        };
        Ok((Expr { ty, kind: ExprKind::Call { func: id, args } }, ret, temp))
    }

    // ----------------------------------------------------------- expressions

    fn cond(&mut self, n: &Node) -> R<Expr> {
        let v = self.expr(n)?;
        match v {
            // Recovery mode: stand in a constant so the branches are lowered.
            Val::Poison => Ok(Expr { ty: Ty::Bool, kind: ExprKind::Const(0) }),
            Val::E(e) if e.ty == Ty::Bool => Ok(e),
            Val::E(e) => self.reject("condition", format!("expected bool, found {}", e.ty.name())),
            Val::Ct(_) => self.reject("condition", "expected bool, found an integer literal".into()),
            Val::P(..) => self.reject("condition", "expected bool, found a pointer".into()),
            Val::M(_) => self.reject("condition", "expected bool, found a struct".into()),
        }
    }

    fn coerce(&mut self, v: Val, to: Ty) -> R<Expr> {
        match v {
            Val::Poison => Err(()),
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
            Val::P(..) => self.reject("type mismatch", format!("expected {}, found a pointer", to.name())),
            Val::M(_) => self.reject("type mismatch", format!("expected {}, found a struct", to.name())),
        }
    }

    /// Bring two operands to one type (Zig peer type resolution, integers).
    fn peer(&mut self, a: Val, b: Val, what: &str) -> R<(Expr, Expr)> {
        if a.is_poison() || b.is_poison() {
            return Err(());
        }
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
            (Val::Poison, _) | (_, Val::Poison) => Err(()),
            (Val::M(_), _) | (_, Val::M(_)) => self.reject("type mismatch", format!("`{}` on a struct", what)),
            (Val::P(..), _) | (_, Val::P(..)) => self.reject("type mismatch", format!("`{}` on a pointer", what)),
        }
    }

    fn expr(&mut self, n: &Node) -> R<Val> {
        match self.expr_inner(n) {
            Err(()) if self.recover => Ok(Val::Poison),
            r => r,
        }
    }

    fn expr_inner(&mut self, n: &Node) -> R<Val> {
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
                    Some(Binding::Var { id, .. }) => {
                        let e = Expr { ty: self.vars[id as usize].ty, kind: ExprKind::Var(id) };
                        Ok(val_of(e, &self.ltys[id as usize]))
                    }
                    Some(Binding::Const(v)) => Ok(v),
                    Some(Binding::Mem(p)) => self.place_value(p),
                    None => match self.global(name)? {
                        Some(v) => Ok(v),
                        None => self.unknown_name(name),
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
                if op == "&" {
                    let p = self.lvalue(&n.children[0])?;
                    let t = LTy::Ptr(Box::new(p.ty.clone()), p.mutable);
                    return Ok(Val::P(addr_of(&p), t));
                }
                let v = self.expr(&n.children[0])?;
                self.unary(&op, v)
            }
            NodeKind::ExprCall => {
                let (call, ret, temp) = self.call(n, None)?;
                match ret {
                    Some(t @ LTy::Struct(_)) => {
                        Ok(Val::M(Place { addr: call, off: 0, ty: t, mutable: false, temp }))
                    }
                    Some(t) => Ok(val_of(call, &t)),
                    None => self.reject("ExprCall", format!("void fn `{}` used as a value", n.name)),
                }
            }
            NodeKind::ExprCast => {
                if n.children.len() != 1 {
                    return self.reject("ExprCast", "unexpected shape".into());
                }
                // The operand first, so that in recovery mode an unsupported
                // operand and an unsupported target type are both named.
                let v = self.expr(&n.children[0])?;
                let to = self.ty(n.extra_type.trim())?;
                self.cast(v, to)
            }
            NodeKind::ExprFieldAccess => {
                let p = self.lvalue(n)?;
                self.place_value(p)
            }
            NodeKind::ExprStructLit => {
                if n.name.is_empty() {
                    return self.reject("ExprStructLit", "anonymous `.{}` literal with no result type".into());
                }
                let t = self.lty(&n.name)?;
                self.struct_temp(n, t)
            }
            _ => {
                let k = kind_name(n);
                self.reject(&k, String::new())
            }
        }
    }

    /// `v as to`. Lossless conversions (bool to an integer as 0 / 1
    /// included) are a `Widen`. A narrowing between two unsigned types
    /// truncates, like Zig's `@truncate`; every other one is checked, like
    /// `@intCast`, and traps when the value is outside `to`. In Wrap mode every
    /// narrowing truncates, as a C cast does.
    fn cast(&mut self, v: Val, to: Ty) -> R<Val> {
        let e = match v {
            Val::Poison => return Err(()),
            Val::Ct(c) if to.is_int() => return Ok(Val::E(self.coerce(Val::Ct(c), to)?)),
            Val::Ct(_) => return self.reject("ExprCast(to bool)", "integer literal as bool".into()),
            Val::E(e) => e,
            Val::P(..) => return self.reject("ExprCast", format!("pointer as {}", to.name())),
            Val::M(_) => return self.reject("ExprCast", format!("struct as {}", to.name())),
        };
        let from = e.ty;
        if from == to {
            return Ok(Val::E(e));
        }
        if to == Ty::Bool {
            return self.reject("ExprCast(to bool)", format!("{} as bool", from.name()));
        }
        if from == Ty::Bool || to.can_widen_from(from) {
            if let ExprKind::Const(c) = e.kind {
                return Ok(Val::E(Expr { ty: to, kind: ExprKind::Const(c) }));
            }
            return Ok(Val::E(Expr { ty: to, kind: ExprKind::Widen(Box::new(e)) }));
        }
        let truncate = self.mode == OverflowMode::Wrap || (!from.signed() && !to.signed());
        if let ExprKind::Const(c) = e.kind {
            if truncate {
                return Ok(Val::E(Expr { ty: to, kind: ExprKind::Const(to.wrap(c)) }));
            }
            if to.fits(c) {
                return Ok(Val::E(Expr { ty: to, kind: ExprKind::Const(c) }));
            }
        }
        let site = if truncate {
            0
        } else {
            self.site(TrapKind::Cast, format!("{} as {}", from.name(), to.name()), to)
        };
        Ok(Val::E(Expr { ty: to, kind: ExprKind::Cast { arg: Box::new(e), site } }))
    }

    /// A name that is neither in scope nor a module-level constant.
    fn unknown_name<T>(&mut self, name: &str) -> R<T> {
        if self.poison_names.contains(name) {
            if self.recover {
                return Err(());
            }
            return self.reject(
                "ExprIdentifier(rejected decl)",
                format!("`{}`, whose declaration was rejected", name),
            );
        }
        // `E::V` of an enum this file rejected: its cascade.
        if self.recover && name.split_once("::").map_or(false, |(e, _)| self.poison_names.contains(e)) {
            return Err(());
        }
        let what = if name == "null" {
            "ExprLiteral(null)"
        } else if name.contains("::") {
            "ExprIdentifier(E::V)"
        } else if Ty::from_name(name).is_some()
            || name.starts_with('[')
            || matches!(name, "f32" | "f64" | "usize" | "isize" | "type")
        {
            "ExprIdentifier(type as value)"
        } else {
            "ExprIdentifier(undeclared)"
        };
        self.reject(what, format!("unknown name `{}`", name))
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
                } else if s.starts_with('-') && parse_int(&s[1..]).is_some() {
                    "negative literal"
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
        if v.is_poison() {
            return Err(());
        }
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
            (op, _) => self.reject(&format!("ExprUnary({})", op), "operand is a pointer or a struct".into()),
        }
    }

    fn binary(&mut self, op: &str, a: Val, b: Val) -> R<Val> {
        if a.is_poison() || b.is_poison() {
            return Err(());
        }
        if matches!(a, Val::P(..) | Val::M(_)) || matches!(b, Val::P(..) | Val::M(_)) {
            let what = if matches!(a, Val::M(_)) || matches!(b, Val::M(_)) { "a struct" } else { "a pointer" };
            return self.reject("type mismatch", format!("`{}` on {}", op, what));
        }
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
            (Val::Ct(_), _) => self.reject(
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
                    _ => self.reject("ExprBinary(<< >>)", "shift amount is a pointer or a struct".into()),
                }
            }
            _ => self.reject("ExprBinary(<< >>)", "shift of a pointer or a struct".into()),
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

    // ---------------------------------------------------------------- memory

    /// A source type: `*T`, `*const T`, a scalar, or a struct (laid out).
    fn lty(&mut self, name: &str) -> R<LTy> {
        self.lty_in(name, true)
    }

    /// `by_value`: whether a struct's layout is needed now. The pointee of a
    /// pointer is only named, so a struct may point to itself.
    fn lty_in(&mut self, name: &str, by_value: bool) -> R<LTy> {
        let t = name.trim();
        if let Some(rest) = t.strip_prefix('*') {
            let rest = rest.trim_start();
            let (inner, mutable) = match rest.strip_prefix("const ") {
                Some(r) => (r, false),
                None => (rest, true),
            };
            let inner = self.lty_in(inner, false)?;
            return Ok(LTy::Ptr(Box::new(inner), mutable));
        }
        if let Some(ty) = Ty::from_name(t) {
            return Ok(LTy::S(ty));
        }
        if self.struct_nodes.contains_key(t) {
            let id = self.struct_id(t);
            if by_value {
                self.layout(id)?;
            }
            return Ok(LTy::Struct(id));
        }
        let (construct, detail) = self.type_construct(t);
        self.reject(&construct, detail)
    }

    fn struct_id(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.struct_ids.get(name) {
            return id;
        }
        let id = self.structs.len() as u32;
        self.structs.push(StructDef { name: name.to_string(), fields: Vec::new(), size: None, align: 1, fail: None });
        self.struct_ids.insert(name.to_string(), id);
        id
    }

    /// Lay out struct `id` (C rules) if that has not been done.
    fn layout(&mut self, id: u32) -> R<()> {
        let sd = &self.structs[id as usize];
        if sd.size.is_some() {
            return Ok(());
        }
        if let Some(r) = sd.fail.clone() {
            // Reported where it was first used; every later use says so too,
            // except in recovery mode, where a construct is named once.
            if !self.recover {
                self.errors.push(Reject { line: self.line, ..r });
            }
            return Err(());
        }
        let name = sd.name.clone();
        let key = format!("struct {}", name);
        if !self.resolving.insert(key.clone()) {
            return self.reject("StructDecl", format!("`{}` contains itself", name));
        }
        let node = self.struct_nodes[&name];
        let saved = self.line;
        let src = self.src;
        if let Some(l) = src.and_then(|s| header_line(s, "const", &name).or_else(|| header_line(s, "pub const", &name))) {
            self.line = l;
        }
        let nerr = self.errors.len();
        let r = self.layout_fields(node);
        self.line = saved;
        self.resolving.remove(&key);
        match r {
            Ok((fields, size, align)) => {
                let sd = &mut self.structs[id as usize];
                sd.fields = fields;
                sd.size = Some(size);
                sd.align = align;
                Ok(())
            }
            Err(()) => {
                if let Some(e) = self.errors.get(nerr).cloned() {
                    self.structs[id as usize].fail = Some(e);
                }
                Err(())
            }
        }
    }

    fn layout_fields(&mut self, node: &'a Node) -> R<(Vec<Field<'a>>, u32, u32)> {
        if !node.params.is_empty() {
            return self.reject("StructDecl", format!("generic struct `{}`", node.name));
        }
        let mut fields: Vec<Field<'a>> = Vec::new();
        let (mut size, mut align) = (0u32, 1u32);
        for f in &node.children {
            if f.kind != NodeKind::ExprIdentifier || f.extra_type.trim().is_empty() || f.children.len() > 1 {
                let k = kind_name(f);
                return self.reject("StructDecl", format!("`{}` member of kind {}", node.name, k));
            }
            if fields.iter().any(|g| g.name == f.name) {
                return self.reject("StructDecl", format!("`{}` has two fields `{}`", node.name, f.name));
            }
            let ty = self.lty(&f.extra_type)?;
            let (fs, fa) = self.size_align(&ty)?;
            let off = size.div_ceil(fa) * fa;
            size = off + fs;
            align = align.max(fa);
            fields.push(Field { name: f.name.clone(), ty, off, default: f.children.first() });
        }
        Ok((fields, size.div_ceil(align) * align, align))
    }

    fn size_align(&mut self, t: &LTy) -> R<(u32, u32)> {
        match t {
            LTy::S(ty) => Ok((ty.bytes(), ty.bytes())),
            LTy::Ptr(..) => Ok((8, 8)),
            LTy::Struct(id) => {
                self.layout(*id)?;
                let sd = &self.structs[*id as usize];
                Ok((sd.size.unwrap(), sd.align))
            }
        }
    }

    fn type_name(&self, t: &LTy) -> String {
        match t {
            LTy::S(ty) => ty.name().to_string(),
            LTy::Ptr(inner, m) => format!("*{}{}", if *m { "" } else { "const " }, self.type_name(inner)),
            LTy::Struct(id) => self.structs[*id as usize].name.clone(),
        }
    }

    /// A new frame slot for one value of type `t`.
    fn new_slot(&mut self, t: &LTy) -> R<u32> {
        let (size, align) = self.size_align(t)?;
        self.slots.push(SlotInfo { size: size.max(1), align: align.max(1) });
        Ok((self.slots.len() - 1) as u32)
    }

    fn fields(&mut self, id: u32) -> R<Vec<Field<'a>>> {
        self.layout(id)?;
        Ok(self.structs[id as usize].fields.clone())
    }

    /// Evaluate `n` as a value of type `want`: a struct literal takes its
    /// type from here, so it may be anonymous (`.{ ... }`).
    fn expr_as(&mut self, n: &Node, want: &LTy) -> R<Val> {
        if n.kind == NodeKind::ExprStructLit && matches!(want, LTy::Struct(_)) {
            self.see(n);
            self.lit_type(n, want)?;
            return self.struct_temp(n, want.clone());
        }
        let v = self.expr(n)?;
        self.coerce_to(v, want)
    }

    /// Check a struct literal's own name, if it has one, against `want`.
    fn lit_type(&mut self, n: &Node, want: &LTy) -> R<()> {
        if n.name.is_empty() {
            return Ok(());
        }
        let t = self.lty(&n.name)?;
        if &t != want {
            let (a, b) = (self.type_name(want), self.type_name(&t));
            return self.reject("type mismatch", format!("expected {}, found {}", a, b));
        }
        Ok(())
    }

    fn coerce_to(&mut self, v: Val, want: &LTy) -> R<Val> {
        if v.is_poison() {
            // Recovery mode: the caller decides what an unknown value costs.
            return Ok(v);
        }
        match want {
            LTy::S(ty) => Ok(Val::E(self.coerce(v, *ty)?)),
            LTy::Ptr(inner, m) => match v {
                // `*T` coerces to `*const T`.
                Val::P(e, LTy::Ptr(got, gm)) if got == *inner && (gm || !*m) => Ok(Val::P(e, want.clone())),
                Val::P(_, t) => {
                    let (a, b) = (self.type_name(want), self.type_name(&t));
                    self.reject("type mismatch", format!("expected {}, found {}", a, b))
                }
                _ => {
                    let a = self.type_name(want);
                    self.reject("type mismatch", format!("expected {}, found a value", a))
                }
            },
            LTy::Struct(_) => match v {
                Val::M(p) if &p.ty == want => Ok(Val::M(p)),
                Val::M(p) => {
                    let (a, b) = (self.type_name(want), self.type_name(&p.ty));
                    self.reject("type mismatch", format!("expected {}, found {}", a, b))
                }
                _ => {
                    let a = self.type_name(want);
                    self.reject("type mismatch", format!("expected {}, found a scalar", a))
                }
            },
        }
    }

    /// The register expression of a scalar or pointer value.
    fn reg(&mut self, v: Val) -> R<Expr> {
        match v {
            Val::E(e) | Val::P(e, _) => Ok(e),
            Val::Poison => Err(()),
            Val::Ct(_) => self.reject("type mismatch", "untyped integer literal".into()),
            Val::M(_) => self.reject("type mismatch", "a struct where a scalar is expected".into()),
        }
    }

    /// A struct literal built in a fresh temporary slot, at the point the
    /// value is evaluated.
    fn struct_temp(&mut self, n: &Node, t: LTy) -> R<Val> {
        let k = self.new_slot(&t)?;
        let dst = Place { addr: slot_expr(k), off: 0, ty: t, mutable: true, temp: None };
        let mut stmts = Vec::new();
        self.init(n, dst.clone(), true, &mut stmts)?;
        let addr = if stmts.is_empty() {
            slot_expr(k)
        } else {
            Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts, value: Box::new(slot_expr(k)) } }
        };
        Ok(Val::M(Place { addr, off: 0, ty: dst.ty, mutable: false, temp: Some(k) }))
    }

    /// Write the value of `n` into `dst`. `fresh`: nothing can read `dst`
    /// while it is being written (a new variable, a temporary, a result), so
    /// a struct literal or a struct-returning call may build in place.
    fn init(&mut self, n: &Node, dst: Place, fresh: bool, out: &mut Vec<Stmt>) -> R<()> {
        self.see(n);
        if is_undefined(n) {
            return Ok(());
        }
        let t = dst.ty.clone();
        let LTy::Struct(id) = t else {
            let v = self.expr_as(n, &t)?;
            let value = self.reg(v)?;
            out.push(Stmt::Store { addr: dst.addr, off: dst.off, value });
            return Ok(());
        };
        if n.kind == NodeKind::ExprStructLit {
            self.lit_type(n, &t)?;
            if fresh && pure_addr(&dst.addr) {
                return self.init_struct(n, id, &dst, out);
            }
            let k = self.new_slot(&t)?;
            let tmp = Place { addr: slot_expr(k), off: 0, ty: t, mutable: true, temp: None };
            self.init_struct(n, id, &tmp, out)?;
            return self.copy(&dst, tmp, out);
        }
        if n.kind == NodeKind::ExprCall && fresh && self.sigs.get(&n.name).is_some_and(|s| s.ret == Some(t.clone())) {
            let (call, _, _) = self.call(n, Some(addr_of(&dst)))?;
            out.push(Stmt::Eval(call));
            return Ok(());
        }
        let v = self.expr_as(n, &t)?;
        match v {
            Val::M(src) => self.copy(&dst, src, out),
            Val::Poison => Err(()),
            _ => self.reject("type mismatch", "internal: struct value not in memory".into()),
        }
    }

    /// The fields of a struct literal, into `dst` (whose address is pure).
    fn init_struct(&mut self, n: &Node, id: u32, dst: &Place, out: &mut Vec<Stmt>) -> R<()> {
        let fields = self.fields(id)?;
        let sname = self.structs[id as usize].name.clone();
        let mut seen = vec![false; fields.len()];
        for c in &n.children {
            self.see(c);
            if c.kind != NodeKind::ExprFieldAccess || c.children.len() != 1 {
                return self.reject("ExprStructLit", format!("positional initializer in a `{}` literal", sname));
            }
            let Some(i) = fields.iter().position(|f| f.name == c.name) else {
                return self.reject("ExprStructLit", format!("`{}` has no field `{}`", sname, c.name));
            };
            if seen[i] {
                return self.reject("ExprStructLit", format!("field `{}` initialised twice", c.name));
            }
            seen[i] = true;
            let sub = field_place(dst, &fields[i]);
            self.init(&c.children[0], sub, true, out)?;
        }
        for (i, f) in fields.iter().enumerate() {
            if seen[i] {
                continue;
            }
            let Some(d) = f.default else {
                return self.reject("ExprStructLit", format!("missing field `{}` of `{}`", f.name, sname));
            };
            // A default sees module scope only.
            let saved = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
            let r = self.init(d, field_place(dst, f), true, out);
            self.scopes = saved;
            r?;
        }
        Ok(())
    }

    /// `dst = src` for a struct: one copy. Both addresses are evaluated, dst
    /// first, even when there is nothing to copy.
    fn copy(&mut self, dst: &Place, src: Place, out: &mut Vec<Stmt>) -> R<()> {
        let (size, _) = self.size_align(&dst.ty)?;
        if size == 0 {
            for p in [dst, &src] {
                if !pure_addr(&p.addr) {
                    out.push(Stmt::Eval(p.addr.clone()));
                }
            }
            return Ok(());
        }
        out.push(Stmt::Copy { dst: addr_of(dst), src: addr_of(&src), size });
        Ok(())
    }

    /// The value stored at a place: a load for a scalar or pointer (folded
    /// when the place is read-only data), the place itself for a struct.
    fn place_value(&mut self, p: Place) -> R<Val> {
        let Some(ty) = reg_ty(&p.ty) else { return Ok(Val::M(p)) };
        if let (ExprKind::Data(k), LTy::S(_)) = (&p.addr.kind, &p.ty) {
            let blob = &self.data[*k as usize];
            let mut raw = 0u64;
            for i in (0..ty.bytes() as usize).rev() {
                raw = (raw << 8) | blob[p.off as usize + i] as u64;
            }
            return Ok(Val::E(Expr { ty, kind: ExprKind::Const(ty.from_raw(raw)) }));
        }
        let e = Expr { ty, kind: ExprKind::Load { addr: Box::new(p.addr), off: p.off } };
        Ok(val_of(e, &p.ty))
    }

    /// The memory an expression names: a variable in memory, a field, or
    /// `p.*`.
    fn lvalue(&mut self, n: &Node) -> R<Place> {
        self.see(n);
        match n.kind {
            NodeKind::ExprIdentifier => match self.lookup(&n.name) {
                Some(Binding::Mem(p)) => Ok(p),
                Some(Binding::Const(Val::Poison)) => Err(()),
                Some(_) => self.reject("ExprUnary(&)", format!("`{}` is not in memory", n.name)),
                None => match self.global(&n.name)? {
                    Some(Val::M(p)) => Ok(p),
                    Some(Val::Poison) => Err(()),
                    Some(_) => self.reject("ExprUnary(&)", format!("address of constant `{}`", n.name)),
                    None => self.unknown_name(&n.name),
                },
            },
            NodeKind::ExprFieldAccess if n.children.len() == 1 => {
                let base = &n.children[0];
                if n.name == "*" {
                    return match self.expr(base)? {
                        Val::P(e, LTy::Ptr(inner, m)) => {
                            if let LTy::Struct(id) = *inner {
                                self.layout(id)?;
                            }
                            Ok(Place { addr: e, off: 0, ty: *inner, mutable: m, temp: None })
                        }
                        Val::Poison => Err(()),
                        _ => self.reject("ExprFieldAccess(.*)", "dereference of a non-pointer".into()),
                    };
                }
                // `Color.red`, `std.math`: the base is not a value.
                if base.kind == NodeKind::ExprIdentifier
                    && self.lookup(&base.name).is_none()
                    && !self.const_nodes.contains_key(&base.name)
                {
                    if self.recover && self.poison_names.contains(&base.name) {
                        return Err(());
                    }
                    return self.reject("ExprFieldAccess", format!("`{}.{}`", base.name, n.name));
                }
                let p = match self.expr(base)? {
                    Val::M(p) => p,
                    Val::Poison => return Err(()),
                    // Field access through a pointer dereferences it.
                    Val::P(e, LTy::Ptr(inner, m)) if matches!(*inner, LTy::Struct(_)) => {
                        Place { addr: e, off: 0, ty: *inner, mutable: m, temp: None }
                    }
                    _ => return self.reject("ExprFieldAccess", format!("`.{}` on a value that is not a struct", n.name)),
                };
                let LTy::Struct(id) = p.ty else { unreachable!() };
                let fields = self.fields(id)?;
                match fields.iter().find(|f| f.name == n.name) {
                    Some(f) => {
                        // A field of a temporary is read from it only once.
                        let mut q = field_place(&p, f);
                        q.mutable = p.mutable && p.temp.is_none();
                        Ok(q)
                    }
                    None => {
                        let s = self.structs[id as usize].name.clone();
                        self.reject("ExprFieldAccess", format!("`{}` has no field `{}`", s, n.name))
                    }
                }
            }
            _ => {
                let k = kind_name(n);
                self.reject(&k, "not addressable".into())
            }
        }
    }

    /// `dst op= rhs` for any place.
    fn store(&mut self, mut dst: Place, op: &str, rhs: &Node, out: &mut Vec<Stmt>) -> R<()> {
        if !dst.mutable {
            return self.reject("StmtAssign", "assignment through a constant".into());
        }
        let plain = op.is_empty() || op == "=";
        if plain {
            return self.init(rhs, dst, false, out);
        }
        let LTy::S(ty) = dst.ty else {
            return self.reject("StmtAssign", format!("`{}` on a pointer or a struct", op));
        };
        let bin = match op.strip_suffix('=') {
            Some(b) if !b.is_empty() => b.to_string(),
            _ => return self.reject("StmtAssign", format!("operator `{}`", op)),
        };
        // The address is computed once, for both the load and the store.
        if !pure_addr(&dst.addr) {
            let h = self.hidden_var("%addr", LTy::Ptr(Box::new(dst.ty.clone()), true));
            out.push(Stmt::Assign { var: h, value: dst.addr });
            dst.addr = Expr { ty: Ty::Ptr, kind: ExprKind::Var(h) };
        }
        let cur = self.place_value(dst.clone())?;
        let r = self.expr(rhs)?;
        let v = self.binary(&bin, cur, r)?;
        let value = self.coerce(v, ty)?;
        out.push(Stmt::Store { addr: dst.addr, off: dst.off, value });
        Ok(())
    }

    /// A module-level struct constant: its bytes in read-only data.
    fn rodata(&mut self, n: &Node, t: LTy) -> R<Val> {
        if n.kind != NodeKind::ExprStructLit {
            let v = self.expr_as(n, &t)?;
            return match v {
                Val::M(p) if matches!(p.addr.kind, ExprKind::Data(_)) => Ok(Val::M(p)),
                _ => self.reject("ConstDecl", "struct constant is not a compile-time value".into()),
            };
        }
        let (size, _) = self.size_align(&t)?;
        let mut buf = vec![0u8; size as usize];
        self.const_fill(n, &t, &mut buf, 0)?;
        self.data.push(buf);
        let k = (self.data.len() - 1) as u32;
        Ok(Val::M(Place { addr: Expr { ty: Ty::Ptr, kind: ExprKind::Data(k) }, off: 0, ty: t, mutable: false, temp: None }))
    }

    fn const_fill(&mut self, n: &Node, t: &LTy, buf: &mut [u8], off: usize) -> R<()> {
        self.see(n);
        if is_undefined(n) {
            return Ok(());
        }
        match t {
            LTy::S(ty) => {
                let v = self.expr(n)?;
                let e = self.coerce(v, *ty)?;
                let ExprKind::Const(c) = e.kind else {
                    return self.reject("ConstDecl", "struct field is not a compile-time value".into());
                };
                let raw = c as u64;
                for i in 0..ty.bytes() as usize {
                    buf[off + i] = (raw >> (8 * i)) as u8;
                }
                Ok(())
            }
            LTy::Ptr(..) => self.reject("ConstDecl", "pointer in a constant struct".into()),
            LTy::Struct(id) if n.kind == NodeKind::ExprStructLit => {
                self.lit_type(n, t)?;
                let fields = self.fields(*id)?;
                let sname = self.structs[*id as usize].name.clone();
                let mut seen = vec![false; fields.len()];
                for c in &n.children {
                    if c.kind != NodeKind::ExprFieldAccess || c.children.len() != 1 {
                        return self.reject("ExprStructLit", format!("positional initializer in a `{}` literal", sname));
                    }
                    let Some(i) = fields.iter().position(|f| f.name == c.name) else {
                        return self.reject("ExprStructLit", format!("`{}` has no field `{}`", sname, c.name));
                    };
                    if seen[i] {
                        return self.reject("ExprStructLit", format!("field `{}` initialised twice", c.name));
                    }
                    seen[i] = true;
                    self.const_fill(&c.children[0], &fields[i].ty, buf, off + fields[i].off as usize)?;
                }
                for (i, f) in fields.iter().enumerate() {
                    if seen[i] {
                        continue;
                    }
                    let Some(d) = f.default else {
                        return self.reject("ExprStructLit", format!("missing field `{}` of `{}`", f.name, sname));
                    };
                    self.const_fill(d, &f.ty, buf, off + f.off as usize)?;
                }
                Ok(())
            }
            LTy::Struct(_) => {
                let v = self.expr_as(n, t)?;
                let Val::M(p) = v else { unreachable!() };
                let ExprKind::Data(k) = p.addr.kind else {
                    return self.reject("ConstDecl", "struct field is not a compile-time value".into());
                };
                let (size, _) = self.size_align(t)?;
                let src = &self.data[k as usize][p.off as usize..(p.off + size) as usize];
                buf[off..off + size as usize].copy_from_slice(src);
                Ok(())
            }
        }
    }
}

/// The register type of a scalar or pointer; None for a struct.
fn reg_ty(t: &LTy) -> Option<Ty> {
    match t {
        LTy::S(ty) => Some(*ty),
        LTy::Ptr(..) => Some(Ty::Ptr),
        LTy::Struct(_) => None,
    }
}

fn val_of(e: Expr, t: &LTy) -> Val {
    match t {
        LTy::S(_) => Val::E(e),
        _ => Val::P(e, t.clone()),
    }
}

fn slot_expr(k: u32) -> Expr {
    Expr { ty: Ty::Ptr, kind: ExprKind::Slot(k) }
}

fn addr_of(p: &Place) -> Expr {
    if p.off == 0 {
        return p.addr.clone();
    }
    let idx = Expr { ty: Ty::U64, kind: ExprKind::Const(p.off as i128) };
    Expr { ty: Ty::Ptr, kind: ExprKind::Offset { base: Box::new(p.addr.clone()), idx: Box::new(idx), scale: 1 } }
}

fn field_place(p: &Place, f: &Field) -> Place {
    Place { addr: p.addr.clone(), off: p.off + f.off, ty: f.ty.clone(), mutable: p.mutable, temp: None }
}

/// An address that may be evaluated more than once with the same result and
/// no effect.
fn pure_addr(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Slot(_) | ExprKind::Data(_) | ExprKind::Var(_) => true,
        ExprKind::Offset { base, idx, .. } => pure_addr(base) && matches!(idx.kind, ExprKind::Const(_)),
        _ => false,
    }
}

fn is_undefined(n: &Node) -> bool {
    n.kind == NodeKind::ExprIdentifier && n.name == "undefined"
}

/// Names whose address is taken (`&name`) anywhere in a body.
fn scan_addr_taken(ns: &[Node], out: &mut HashSet<String>) {
    for n in ns {
        if n.kind == NodeKind::ExprUnary && n.extra_op == "&" {
            if let Some(c) = n.children.first() {
                if c.kind == NodeKind::ExprIdentifier {
                    out.insert(c.name.clone());
                }
            }
        }
        scan_addr_taken(&n.children, out);
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

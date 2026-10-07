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
//!
//! Strings: `str` (also spelled `string`, `&str`, `[]const u8`) is Zig's
//! `[]const u8`, a 16-byte aggregate in memory -- the address of the bytes at
//! offset 0, the length (u64) at offset 8 -- passed, returned and copied like
//! a struct. A string literal is a compile-time value (`Val::S`) whose bytes
//! sit in read-only data; it is written into memory only where a `str` place
//! needs it. `==` and `!=` on strings compare contents, as t27c's Zig backend
//! does with `std.mem.eql`: two literals fold, anything else calls one
//! synthesized IR function, `__t27b_str_eql`. `std.mem.eql(u8, a, b)` is the
//! same comparison, and `std.mem.indexOf(u8, h, n)` compared with `null`
//! calls a second one, `__t27b_str_contains` (`lower/stdmem.rs`).
//!
//! Arrays: `[N]T` (N a literal or an integer constant) is N elements of T back
//! to back in memory, an aggregate like a struct: copied on assignment, passed
//! by pointer, returned through the result slot. An array literal takes the
//! type it is assigned to (t27c's Zig backend writes `.{...}`), so one with no
//! result type is refused, and its element count must be N. An index is a
//! u64; a constant one out of range is refused (Zig refuses it at compile
//! time), a runtime one is checked and traps as `index out of bounds`. `.len`
//! is the constant N. `for (a) |x|` walks the elements with a hidden index. A
//! module constant array of strings cannot live in read-only data (it holds
//! addresses), so it stays a compile-time value (`Val::A`) and is written into
//! a frame temporary only where memory is needed. t27's own `[T; N]` is
//! `[N]T`, as in t27c's `t27_array_type_to_zig`. A repeat -- `[v; n]`, whose
//! element the parser keeps as the text `v;n`, or `[_]T{ a, b } ** n` -- is
//! `.{ ... } ** n` in the reference: its elements are evaluated once, then
//! the filled prefix is copied forward. Element text the parser kept (`[v;
//! n]`, and a list like `[s1]`) is parsed back only when it is a literal, a
//! name, a call or a field access, the shapes that mean the same pasted into
//! Zig.
//!
//! Slices: `[]T` and `[]const T` are Zig's slices, laid out like `str` (which
//! is exactly `[]const u8`): a 16-byte aggregate, the address of the first
//! element at offset 0 and the length (u64) at offset 8. Whether the elements
//! may be written is part of the type, not of the place holding the slice.
//! `a[i..j]` (and the open `a[i..]`) slices an array, a pointer to an array, a
//! slice or a string: the end is checked against the length and the start
//! against the end, both as `index out of bounds`. Indexing a slice or a
//! string is checked against its runtime length. `*[N]T` coerces to `[]T` and
//! `[]const T`, `[]T` to `[]const T`, and `&[_]T{ ... }` to `[]const T`.
//! t27's `[T]` is the mutable slice `[]T`. An array literal passed where a
//! callee declares a slice is `@constCast(&[_]T{ ... })` in the reference (a
//! temporary that outlives the call), unless the element is itself an array
//! or a slice; an untyped local bound to an array literal and passed so is a
//! `[_]T` array of the callee's element type, passed by address (t27c's
//! `slice_locals`). An untyped `const` local bound to an array literal and
//! only ever passed where one array type `[N]T` is declared is that array
//! (the reference's `.{ ... }` coerces at each call); one of plain literals
//! that nothing reads is dropped. An empty literal for a slice field of a
//! named struct literal is an empty slice. `[*]T` and the map `[K:V]` are
//! refused by name.
//! `for (s) |x|` reads the slice's address and length once, before the loop.

mod arraylit;
mod floatas;
mod formulti;
mod lencall;
mod stdmem;
mod unanalyzed;

use crate::codegen;
use crate::compiler::{Node, NodeKind};
use crate::ir::*;
use std::collections::{HashMap, HashSet};

#[path = "lower_float.rs"]
mod float;
mod refvars;
mod tuple;

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
    /// A compile-time float (Zig's comptime_float): its binary128 value
    /// (see `float::Q`).
    Cf(float::Q),
    E(Expr),
    P(Expr, LTy),
    M(Place),
    /// A compile-time string: read-only blob `k` (its bytes, then one NUL
    /// byte that is not part of the string) and the length in bytes.
    S(u32, u64),
    /// A compile-time array whose elements hold strings (`LTy::Arr` type,
    /// one `S` or nested `A` per element): read-only data cannot hold the
    /// address of a string, so the array is written into a frame temporary
    /// wherever memory is needed, and a constant index folds to the element.
    A(LTy, Vec<Val>),
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
    /// `str`: `[]const u8`, a (pointer, length) pair in memory.
    Str,
    /// `[N]T`: N elements of T, back to back, in memory.
    Arr(Box<LTy>, u32),
    /// `[]T` (true: elements writable) or `[]const T` (false): a (pointer,
    /// length) pair in memory. `[]const u8` is `Str`, never this.
    Slice(Box<LTy>, bool),
    /// A plain enum (index into `Lower::enums`): its tag, an integer of
    /// the register type given, in one register.
    Enum(u32, Ty),
    /// `?T`: the payload (a `T`) at offset 0, then one byte, 1 when there
    /// is a value and 0 for `null`, the whole rounded up to `T`'s
    /// alignment. Lives in memory, like a struct.
    Opt(Box<LTy>),
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

/// A plain enum, lowered to its integer tag.
struct EnumDef {
    name: String,
    /// The register type holding the tag.
    tag: Ty,
    /// The width of Zig's tag type: `tag`'s own width, or for an enum
    /// declared with no tag type (Zig picks the smallest unsigned integer
    /// that holds every tag) possibly fewer bits, which t27b has no type for.
    bits: u32,
    /// (name, tag value), in declaration order.
    variants: Vec<(String, i128)>,
}

impl EnumDef {
    /// `@intFromEnum` gives a value of a type t27b has.
    fn exact(&self) -> bool {
        self.bits == self.tag.bits()
    }

    fn value(&self, name: &str) -> Option<i128> {
        self.variants.iter().find(|v| v.0 == name).map(|v| v.1)
    }
}

/// Where a runtime `@intFromEnum` of an enum whose tag type t27b has no type
/// for (`u2`, say) is used: only where its storage type gives the same answer.
#[derive(Clone, Copy, PartialEq)]
enum TagUse {
    /// Nowhere special: refused, since arithmetic on it would overflow at a
    /// different point.
    Value,
    /// An operand of `==`, `<`, ... or `assert_eq`: exact at any width.
    Compare,
    /// Coerced to this integer type, which Zig allows when it holds every
    /// value of the tag type.
    Want(Ty),
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
    /// Module-level `var` declarations, in source order.
    var_nodes: Vec<&'a Node>,
    /// Test, bench and invariant blocks (by address) that name a module-level
    /// var the reference's Zig backend renames to `<name>_arg` there: its
    /// parameter-rename map is only cleared at the next fn, so a block after
    /// `fn f(x: T)` reads an undeclared `x_arg` when `x` is also a module var.
    leaky: HashMap<usize, String>,
    /// Each module-level `var` that lowered: its writable place.
    mod_vars: HashMap<String, Place>,
    /// Initial bytes of each module-level `var` (`Program::globals`).
    globals_init: Vec<Vec<u8>>,
    /// Lowering what t27c's Zig backend evaluates at compile time (an
    /// `invariant`, which it emits as a `comptime` block, or a module-level
    /// initializer): a module-level `var` is not visible there.
    comptime: bool,
    struct_nodes: HashMap<String, &'a Node>,
    structs: Vec<StructDef<'a>>,
    struct_ids: HashMap<String, u32>,
    /// Enum declarations, built on first use (all of them after pass 1).
    enum_nodes: HashMap<String, &'a Node>,
    enums: Vec<EnumDef>,
    enum_ids: HashMap<String, u32>,
    /// Why an enum declaration was refused, to report again at a later use.
    enum_fail: HashMap<String, Reject>,
    data: Vec<Vec<u8>>,
    internal_abi: Vec<FuncId>,
    /// Blob of each string literal's bytes, so equal literals share one.
    strings: HashMap<Vec<u8>, u32>,
    /// The number of source fns: the id `__t27b_str_eql` gets if used.
    nfuncs: FuncId,
    /// `__t27b_str_eql` is called somewhere.
    eql_used: bool,
    /// `__t27b_str_contains` is called somewhere (`stdmem`).
    contains_used: bool,
    // Per-function state.
    vars: Vec<Var>,
    /// The source type of each variable (parallel to `vars`).
    ltys: Vec<LTy>,
    slots: Vec<SlotInfo>,
    /// Names whose address is taken somewhere in the body (`&x`): such a
    /// scalar lives in a frame slot instead of a register.
    addr_taken: HashSet<String>,
    /// Names the reference makes a `var` in this body (`refvars`).
    ref_vars: HashSet<String>,
    /// Untyped locals bound to an array literal and passed where a callee
    /// declares a slice, with that slice's element type: t27c's Zig backend
    /// makes each a `var x = [_]T{ ... }` array and passes `&x`
    /// (`collect_slice_locals`).
    slice_locals: HashMap<String, LTy>,
    /// Untyped `const` locals bound to an array literal whose every mention
    /// is a call argument where the callee declares one array type `[N]T`,
    /// with that type. t27c's Zig backend writes the local as an anonymous
    /// `.{ ... }` (any `[N]T` prefix dropped), which coerces at each call.
    tuple_locals: HashMap<String, LTy>,
    /// Untyped `const` locals bound to an array literal of plain literals
    /// and never mentioned again: the reference's `.{ ... }` plus
    /// `_ = x; // dead after const-inlining`, which does nothing at all.
    dead_lits: HashSet<String>,
    /// The current fn's parameters, each with whether `_ = p;` in a nested
    /// block may discard it: the body names `p` only in such discards.
    /// Empty outside a fn.
    discards: HashMap<String, bool>,
    /// The hidden result pointer of a function returning a struct.
    sret: Option<VarId>,
    scopes: Vec<HashMap<String, Binding>>,
    loop_depth: u32,
    line: u32,
    ret: Option<LTy>,
    in_test: bool,
    /// Lowering a `bench` body (compiled, never run).
    in_bench: bool,
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
    /// The declared integer type of the local whose initializer is being
    /// lowered (`var h : i32 = 1 << d;`). An untyped literal shifted by a
    /// non-literal amount takes this width, as in the Zig backend.
    decl_int: Option<Ty>,
    /// Fns the reference's Zig may analyze: every fn a test, invariant,
    /// bench or module-level declaration names, and every fn those name, to
    /// a fixed point (`analyzed_fns`).
    analyzed: HashSet<String>,
    /// Lowering a fn outside `analyzed`: Zig compiles a fn body only when
    /// something analyzed references it, so a body stub there is never seen.
    unanalyzed_fn: bool,
    /// Fns outside `analyzed` whose signature names a struct t27b cannot lay
    /// out (`unresolved_sig`): no body, and a call to one is refused.
    unresolved: HashSet<String>,
    /// A `layout` failed since this was last cleared (`unresolved_sig`).
    layout_err: bool,
    /// Every type the last rejected `signature` refused failed in `layout`.
    sig_layout_only: bool,
    /// gen-zig's `float_names` (`floatas`): struct fields, and the current
    /// fn's float parameters and locals.
    float_fields: HashSet<String>,
    float_locals: HashSet<String>,
    /// Module fns declared `-> bool`: a bare call to one is a predicate in a
    /// brace invariant (#6315, `invariant_predicate`).
    bool_fns: HashSet<String>,
    /// The top-level statements (by address) of the invariant being lowered
    /// that the reference checks as `assert(<expr>)` (#6315).
    invariant_preds: HashSet<usize>,
    /// Module fns declared to return a value (any return type but none,
    /// `void`, `noreturn` or `()`): a bare call to one is discarded by
    /// name, `_ = f(x);` (#6315, t27c `call_returns_value`).
    value_fns: HashSet<String>,
    /// The tail statements (by address) of the analyzed non-void fn being
    /// lowered that the reference returns (#6315, t27c `zig_tail_returns`).
    tail_returns: HashSet<usize>,
    /// `if` expressions (by address) that the reference's Zig backend prints
    /// without the parentheses the source has: the left operand of a binary
    /// operator, or the base of a field access or index. `(if (c) a else b)
    /// + 1` comes out as `if (c) a else b + 1`, which Zig reads with the `+ 1`
    /// inside the else arm, so no lowering of the source agrees with it.
    misprinted_if: HashSet<usize>,
    /// `*` expressions (by address) that t27c's strength reduction may rewrite
    /// as `<<` (`strength_reduced`); a float `x * 2^k` among them is refused.
    shifted_muls: HashSet<usize>,
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
        var_nodes: Vec::new(),
        leaky: HashMap::new(),
        mod_vars: HashMap::new(),
        globals_init: Vec::new(),
        comptime: false,
        struct_nodes: HashMap::new(),
        structs: Vec::new(),
        struct_ids: HashMap::new(),
        enum_nodes: HashMap::new(),
        enums: Vec::new(),
        enum_ids: HashMap::new(),
        enum_fail: HashMap::new(),
        data: Vec::new(),
        internal_abi: Vec::new(),
        strings: HashMap::new(),
        nfuncs: 0,
        eql_used: false,
        contains_used: false,
        vars: Vec::new(),
        ltys: Vec::new(),
        slots: Vec::new(),
        addr_taken: HashSet::new(),
        ref_vars: HashSet::new(),
        slice_locals: HashMap::new(),
        tuple_locals: HashMap::new(),
        dead_lits: HashSet::new(),
        discards: HashMap::new(),
        sret: None,
        scopes: Vec::new(),
        loop_depth: 0,
        line: ast.line,
        ret: None,
        in_test: false,
        in_bench: false,
        test_assigns: HashMap::new(),
        src,
        recover,
        poison_names: HashSet::new(),
        type_decls: HashMap::new(),
        ret_poison: false,
        decl_int: None,
        analyzed: HashSet::new(),
        unanalyzed_fn: false,
        unresolved: HashSet::new(),
        layout_err: false,
        sig_layout_only: false,
        float_fields: HashSet::new(),
        float_locals: HashSet::new(),
        bool_fns: HashSet::new(),
        invariant_preds: HashSet::new(),
        value_fns: HashSet::new(),
        tail_returns: HashSet::new(),
        misprinted_if: HashSet::new(),
        shifted_muls: HashSet::new(),
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
        if item.kind == NodeKind::EnumDecl && !item.name.is_empty() {
            l.enum_nodes.insert(item.name.clone(), item);
        }
        // Constants too, before any signature: a top-level Zig declaration
        // is visible above its own line, and a length like `[N]T` in a
        // signature may name a constant `use` spliced in after it.
        if item.kind == NodeKind::ConstDecl && !item.extra_mutable && !is_tagged_union(item) {
            l.const_nodes.insert(item.name.clone(), item);
        }
    }

    l.analyzed = analyzed_fns(&items);
    l.float_field_names(&items);
    for item in &items {
        if item.kind == NodeKind::FnDecl && !item.name.is_empty() && item.extra_return_type.trim() == "bool" {
            l.bool_fns.insert(item.name.clone());
        }
        if item.kind == NodeKind::FnDecl && !item.name.is_empty() && returns_value(&item.extra_return_type) {
            l.value_fns.insert(item.name.clone());
        }
    }
    misprinted_ifs(ast, &mut l.misprinted_if);
    strength_reduced(&items, &mut l.shifted_muls);
    l.reference_defects(ast);

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
            NodeKind::ConstDecl if is_tagged_union(item) => {
                l.type_decls.insert(item.name.clone(), "tagged union");
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
                        "FnDecl(duplicate)",
                        format!("duplicate function `{}`", item.name),
                    );
                    continue;
                }
                let nerr = l.errors.len();
                match l.signature(item) {
                    Ok((params, ret)) => {
                        if params.iter().any(is_agg) || ret.as_ref().is_some_and(is_agg) {
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
                    Err(()) if l.unresolved_sig(item, nerr) => {}
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
            NodeKind::TestBlock | NodeKind::InvariantBlock | NodeKind::BenchBlock | NodeKind::StructDecl => {}
            // Built after pass 1, once every constant a tag may name is known.
            NodeKind::EnumDecl if !item.name.is_empty() => {}
            // `union(enum) { ... }`: t27c keeps only its text.
            NodeKind::ConstDecl if is_tagged_union(item) => {
                if let Some(l2) = l.src.and_then(|s| decl_line(s, &item.name)) {
                    l.line = l2;
                }
                let _: R<()> = l.reject(
                    "EnumDecl(union)",
                    format!("tagged union `{}`; only plain enums are supported", item.name),
                );
                l.poison_names.insert(item.name.clone());
            }
            NodeKind::ConstDecl if item.extra_mutable => l.var_nodes.push(item),
            NodeKind::ConstDecl => {
                l.const_nodes.insert(item.name.clone(), item);
            }
            // `use` declarations were already resolved by splicing the
            // imported declarations into the source (front::parse).
            NodeKind::UseDecl => {}
            // A statement at top level: t27c's Zig backend (`gen_decl`) emits
            // nothing for it, so the reference compiles the file without it.
            // The common source is a dotted `module a.b;` or `use a.b;`, which
            // the t27c parser reads as `a` followed by a stray `.b` (#6102); a
            // test body the parser closed early leaves its tail here too.
            NodeKind::StmtExpr => {}
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
    l.nfuncs = next_id;
    let mut const_names: Vec<String> = l.const_nodes.keys().cloned().collect();
    const_names.sort();
    for name in const_names {
        if l.unreferenced_tuple_const(&items, &name) || l.alias_target(&name).is_some() {
            continue;
        }
        let _ = l.global(&name);
    }
    for node in std::mem::take(&mut l.var_nodes) {
        if l.module_var(node).is_err() {
            l.poison_names.insert(node.name.clone());
        }
    }
    let mut enum_names: Vec<String> = l.enum_nodes.keys().cloned().collect();
    enum_names.sort();
    for name in enum_names {
        if !l.enum_ids.contains_key(&name) && !l.enum_fail.contains_key(&name) {
            let _ = l.enum_id(&name);
        }
    }

    let mut renamed: Vec<String> = Vec::new();
    for item in &items {
        match item.kind {
            NodeKind::FnDecl => {
                renamed = item
                    .params
                    .iter()
                    .map(|(p, _)| p.trim().to_string())
                    .filter(|p| l.mod_vars.contains_key(p))
                    .collect();
            }
            NodeKind::TestBlock | NodeKind::BenchBlock | NodeKind::InvariantBlock => {
                if let Some(name) = renamed.iter().find(|p| mentions(&item.children, p)) {
                    l.leaky.insert(*item as *const Node as usize, name.clone());
                }
            }
            _ => {}
        }
    }

    // Pass 2: bodies.
    let mut funcs: Vec<Func> = Vec::new();
    for item in &fn_nodes {
        if let Ok(f) = l.function(item) {
            funcs.push(f);
        }
    }
    let mut unchecked = Vec::new();
    let mut benches: Vec<Func> = Vec::new();
    for item in &items {
        match item.kind {
            NodeKind::TestBlock => {
                if let Ok(f) = l.test(item, false) {
                    funcs.push(f);
                }
            }
            // Lowered and compiled below, never run and never a test.
            NodeKind::BenchBlock => {
                if let Ok(f) = l.bench(item) {
                    benches.push(f);
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
    l.synthesized(&mut funcs);
    // More than 8 parameters of a class: the rest are passed on the stack
    // in t27b's own convention (`Program::stack_args`), never exported.
    for (id, f) in funcs.iter().enumerate() {
        let id = id as FuncId;
        if Program::stack_args(&Program::arg_regs(f)).1 > 0 && !l.internal_abi.contains(&id) {
            l.internal_abi.push(id);
        }
    }
    let prog = Program {
        module,
        funcs,
        sites: l.sites,
        mode,
        unchecked,
        data: l.data,
        globals: l.globals_init,
        internal_abi: l.internal_abi,
    };
    // Bench bodies are compiled to machine code, so a body that lowers but
    // that the code generator refuses still rejects the file, and then
    // dropped: the program that runs is exactly the one without them.
    if !benches.is_empty() {
        let mut with = prog.clone();
        let first = with.funcs.len();
        with.funcs.extend(benches);
        let mut errors = Vec::new();
        for id in first..with.funcs.len() {
            if let Err(e) = codegen::compile_func(&with, id as FuncId, codegen::TrapStyle::Jit) {
                errors.push(Reject {
                    construct: e.construct.to_string(),
                    line: e.line,
                    detail: format!("bench {}: {}", with.funcs[id].name, e.detail),
                });
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
    }
    Ok(prog)
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

/// Line (1-based) of the declaration of `name`: `const`, `pub const`,
/// `enum` or `pub enum`.
fn decl_line(src: &str, name: &str) -> Option<u32> {
    ["const", "pub const", "enum", "pub enum"]
        .iter()
        .filter_map(|k| header_line(src, k, name))
        .min()
}

/// t27c reads `const U = union(enum) { ... };` as a `ConstDecl` with no value
/// and the declaration's tokens as text.
fn is_tagged_union(n: &Node) -> bool {
    n.kind == NodeKind::ConstDecl && n.children.is_empty() && n.value.replace(' ', "").starts_with("union(enum")
}

/// The right-hand side of `const Name = T;` when it may be a type: no
/// annotation, and a bare name (the parser keeps `[N]T` as one name too).
fn alias_text(n: &Node) -> Option<&str> {
    if n.kind != NodeKind::ConstDecl || n.extra_mutable || !n.extra_type.trim().is_empty() || n.children.len() != 1 {
        return None;
    }
    let c = &n.children[0];
    (c.kind == NodeKind::ExprIdentifier && c.children.is_empty() && !c.name.trim().is_empty()).then(|| c.name.trim())
}

fn kind_name(n: &Node) -> String {
    format!("{:?}", n.kind)
}

/// Parse an integer literal: decimal, 0x, 0o, 0b, with `_` separators.
/// The value of a char literal as the t27c parser leaves it: the quotes kept
/// and the lexeme raw, either one byte or a backslash and one byte (the lexer
/// reads anything longer as a string). t27c's Zig backend prints it back
/// verbatim, so only what Zig itself accepts is a value here: a printable
/// ASCII byte, or one of the escapes `\n \r \t \\ \' \"`. Zig rejects `'\0'`
/// ("invalid escape character") and a raw control byte, so neither has a
/// reference result to agree with.
fn char_literal(s: &str) -> Result<i128, &'static str> {
    let inner = s
        .strip_prefix('\'')
        .and_then(|r| r.strip_suffix('\''))
        .ok_or("ExprLiteral(char literal)")?;
    let b = inner.as_bytes();
    match b {
        [b'\\', e] => match e {
            b'n' => Ok(10),
            b'r' => Ok(13),
            b't' => Ok(9),
            b'\\' | b'\'' | b'"' => Ok(*e as i128),
            _ => Err("ExprLiteral(char escape)"),
        },
        [c] if (0x20..0x7f).contains(c) => Ok(*c as i128),
        [_] => Err("ExprLiteral(char byte)"),
        _ => Err("ExprLiteral(char literal)"),
    }
}

/// `n` is a char literal (`'a'`, `'\n'`): an untyped comptime_int in Zig that
/// t27c's constant folder does not see as a literal.
fn is_char_literal(n: &Node) -> bool {
    n.kind == NodeKind::ExprLiteral && n.extra_kind != "string" && n.value.trim().starts_with('\'')
}

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

/// The width t27c's Zig backend pins on an untyped `var` set to a bare
/// integer literal, as Zig refuses a `var` of type comptime_int
/// (`zig_int_literal_default_type` in bootstrap/src/compiler.rs): the
/// literal's suffix, else u32 when the value fits and u64 when it does not.
/// `-1`, `0.0`, `N` or an octal literal stay untyped there, and Zig refuses
/// them, so they get None here (#6967).
fn int_lit_width(n: &Node) -> Option<&'static str> {
    if n.kind != NodeKind::ExprLiteral || n.extra_kind == "string" {
        return None;
    }
    const SUFFIXES: [&str; 10] = ["u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize"];
    if let Some(s) = SUFFIXES.iter().find(|s| **s == n.extra_type) {
        return Some(s);
    }
    let v = n.value.trim();
    if v.starts_with("0o") {
        return None;
    }
    match parse_int(v)? {
        x if x <= u32::MAX as i128 => Some("u32"),
        x if x <= u64::MAX as i128 => Some("u64"),
        _ => None,
    }
}

/// `n` is an integer literal that is a power of two above 1: what t27c's
/// strength reduction rewrites `x * n` into `x << k` for, whatever x's type.
fn pow2_literal(n: &Node) -> bool {
    n.kind == NodeKind::ExprLiteral
        && n.extra_kind != "string"
        && parse_int(n.value.trim()).is_some_and(|v| v > 1 && (v & (v - 1)) == 0)
}

impl<'a> Lower<'a> {
    fn see(&mut self, n: &Node) {
        if n.line != 0 {
            self.line = n.line;
        }
    }

    /// Shapes the reference path (`t27c test-report`: t27c's Zig backend,
    /// then `zig test`) cannot compile, refused here so t27b never passes a
    /// file the reference cannot back (#6358). Each is a deterministic t27c
    /// defect, and each is a Zig parse or AstGen error, which fires whether
    /// or not anything calls the code. The checks read the AST after the
    /// same `optimize` call `Compiler::compile` makes, since that is the
    /// tree the reference emits.
    fn reference_defects(&mut self, ast: &Node) {
        let mut opt = ast.clone();
        crate::compiler::optimize(
            &mut opt,
            &crate::compiler::OptConfig {
                keep_call_discards: true,
                ..crate::compiler::OptConfig::default()
            },
        );
        let items: Vec<&Node> = if opt.kind == NodeKind::Module {
            opt.children.iter().collect()
        } else {
            vec![&opt]
        };
        let declared: HashSet<&str> = items.iter().map(|n| n.name.as_str()).filter(|s| !s.is_empty()).collect();
        let mut found: Vec<(u32, &'static str, String)> = Vec::new();
        for item in &items {
            match item.kind {
                NodeKind::StructDecl => {
                    for f in &item.children {
                        if let Some(b) = undeclared_field_type(&f.extra_type, &declared) {
                            let line = self.src.and_then(|s| decl_line(s, &item.name)).unwrap_or(item.line);
                            found.push((
                                line,
                                "StructDecl(reference undeclared field type)",
                                format!(
                                    "field `{}.{}` has type `{}`; t27c's Zig backend emits `{}`, which Zig does not declare",
                                    item.name, f.name, f.extra_type.trim(), b
                                ),
                            ));
                        }
                    }
                }
                NodeKind::FnDecl => {
                    if let Some(name) = cse_hoist_defect(item) {
                        found.push((
                            item.line,
                            "FnDecl(reference CSE hoist)",
                            format!(
                                "fn `{}`: t27c's optimizer hoists a common subexpression over the local `{}` above its declaration",
                                item.name, name
                            ),
                        ));
                    }
                    let mut muts = HashSet::new();
                    ref_mutable_names(&item.children, &mut muts);
                    for (p, _) in &item.params {
                        if p != "self" && muts.contains(p) && !zig_mutates(&item.children, p) {
                            found.push((
                                item.line,
                                "FnDecl(reference var param)",
                                format!(
                                    "fn `{}`: parameter `{}` is only written through `{}.*`; t27c's Zig backend rebinds it as `var {} = {}_arg;`, which Zig rejects as never mutated",
                                    item.name, p, p, p, p
                                ),
                            ));
                        }
                    }
                }
                NodeKind::TestBlock => {
                    let mut locals: HashSet<&str> = HashSet::new();
                    let mut bound: HashSet<&str> = HashSet::new();
                    for s in &item.children {
                        if s.kind == NodeKind::StmtLocal && !s.name.is_empty() {
                            locals.insert(s.name.as_str());
                        }
                        if s.kind == NodeKind::StmtAssign
                            && s.children.len() >= 2
                            && s.children[0].kind == NodeKind::ExprIdentifier
                            && !s.children[0].name.is_empty()
                        {
                            let name = s.children[0].name.as_str();
                            if bound.insert(name) && locals.contains(name) {
                                found.push((
                                    if s.line != 0 { s.line } else { item.line },
                                    "StmtAssign(reference redeclares)",
                                    format!(
                                        "test `{}`: the first top-level assignment to the local `{}` is emitted by t27c's Zig backend as a fresh `const {} = ..`, a redeclaration",
                                        item.name, name, name
                                    ),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        zig_syntax_defects(&opt.children, 0, &mut found);
        found.sort_by_key(|f| f.0);
        for (line, construct, detail) in found {
            if line != 0 {
                self.line = line;
            }
            let _: R<()> = self.reject(construct, detail);
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
                if let Some(target) = self.alias_target(t) {
                    return self.ty(target);
                }
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

    /// `const Name = T;` with no annotation, where `T` is a bare name that
    /// spells a type: t27c's Zig backend prints it unchanged and Zig reads it
    /// as a type alias, so `Name` is exactly `T` wherever a type is read.
    /// Returns `T`; None for a value constant, an alias cycle, or a `T` that
    /// is not a type t27b can name (`std.mem.Allocator`).
    fn alias_target(&self, name: &str) -> Option<&'a str> {
        let node = *self.const_nodes.get(name)?;
        let t = alias_text(node)?;
        self.spells_type(t, 0).then_some(t)
    }

    /// The text is printed into Zig verbatim, so only Zig spellings count:
    /// `str` / `string` are t27 names Zig does not know, and `[N]T` counts
    /// only when `T` itself spells a type. (`?T`, `*T` and `&str` never get
    /// here: the reference's parser leaves such a constant with no value.)
    fn spells_type(&self, t: &str, depth: u32) -> bool {
        if depth > 16 {
            return false;
        }
        if let Some(rest) = t.strip_prefix('[') {
            return rest
                .split_once(']')
                .is_some_and(|(_, elem)| self.spells_type(elem.trim(), depth + 1));
        }
        Ty::from_name(t).is_some()
            || self.struct_nodes.contains_key(t)
            || self.enum_nodes.contains_key(t)
            || self
                .const_nodes
                .get(t)
                .and_then(|n| alias_text(n))
                .is_some_and(|inner| self.spells_type(inner, depth + 1))
    }

    fn signature(&mut self, n: &Node) -> R<(Vec<LTy>, Option<LTy>)> {
        let mut bad = false;
        let mut params = Vec::new();
        self.sig_layout_only = true;
        for (pname, pty) in &n.params {
            self.layout_err = false;
            let r = if pname.starts_with("comptime ") {
                self.reject("FnDecl(comptime param)", format!("parameter `{}` of `{}`", pname, n.name))
            } else if pty.is_empty() {
                self.reject("FnDecl(untyped param)", format!("parameter `{}` of `{}`", pname, n.name))
            } else {
                self.lty(pty)
            };
            self.sig_layout_only &= r.is_ok() || self.layout_err;
            match r {
                Ok(t) => params.push(t),
                // Recovery mode reports every parameter and the return type.
                Err(()) if self.recover => bad = true,
                Err(()) => return Err(()),
            }
        }
        let rt = n.extra_return_type.trim();
        let ret = if rt.is_empty() || rt == "void" || self.unanalyzed_undefined_ret(n) {
            None
        } else if rt.strip_prefix("struct").is_some_and(|r| r.trim_start().starts_with('{')) {
            Some(self.anon_struct_ret(&n.name, rt)?)
        } else {
            self.layout_err = false;
            let r = self.ret_lty(rt);
            self.sig_layout_only &= r.is_ok() || self.layout_err;
            Some(r?)
        };
        // A struct result is returned through a hidden pointer parameter.
        let total = n.params.len() + ret.as_ref().is_some_and(is_agg) as usize;
        if total > MAX_PARAMS {
            self.sig_layout_only = false;
            return self.reject(
                "FnDecl(too many params)",
                format!("`{}` has {} parameters, at most {} are supported", n.name, total, MAX_PARAMS),
            );
        }
        if bad {
            return Err(());
        }
        Ok((params, ret))
    }

    /// `-> struct { a: T, b: U }`, fn `f`'s own result type. The reference
    /// prints the spelling token by token (`struct { a : [ ] const u8 }`), so
    /// a field type counts only when Zig spells it so (not `str`, not
    /// `[T; N]`), and a field name only when Zig takes it bare. Zig makes
    /// each such type distinct, so it is interned per fn; values come from
    /// `return .{ .a = .. }` and from calls.
    fn anon_struct_ret(&mut self, f: &str, rt: &str) -> R<LTy> {
        // Zig's keywords: printed bare here, as a field name they do not parse.
        const ANON_FIELD_KEYWORDS: &[&str] = &[
            "align", "allowzero", "and", "anyframe", "anytype", "asm", "async", "await", "break", "callconv",
            "catch", "comptime", "const", "continue", "defer", "else", "enum", "errdefer", "error", "export",
            "extern", "fn", "for", "if", "inline", "linksection", "noalias", "noinline", "nosuspend", "opaque",
            "or", "orelse", "packed", "pub", "resume", "return", "struct", "suspend", "switch", "test",
            "threadlocal", "try", "union", "unreachable", "usingnamespace", "var", "volatile", "while",
        ];
        let key =format!("{} (result of `{}`)", rt, f);
        if let Some(&id) = self.struct_ids.get(&key) {
            return Ok(LTy::Struct(id));
        }
        let body = rt
            .strip_prefix("struct")
            .and_then(|r| r.trim().strip_prefix('{'))
            .and_then(|r| r.strip_suffix('}'))
            .unwrap_or("{");
        if body.contains(['{', '}', '(', ')', '=']) {
            return self.reject("type (anonymous struct)", format!("`{}`: only `name: type` fields", rt));
        }
        let mut fields: Vec<Field<'a>> = Vec::new();
        let (mut size, mut align) = (0u32, 1u32);
        let parts: Vec<&str> = body.split(',').map(str::trim).collect();
        for (i, p) in parts.iter().enumerate() {
            // A trailing comma leaves one empty part at the end.
            if p.is_empty() && i + 1 == parts.len() && i > 0 {
                break;
            }
            let Some((name, ty)) = p.split_once(':') else {
                return self.reject("type (anonymous struct)", format!("`{}`: field `{}`", rt, p));
            };
            let name = name.trim();
            let ident = name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !ident || ANON_FIELD_KEYWORDS.contains(&name) {
                return self.reject("type (anonymous struct)", format!("`{}`: field name `{}`", rt, name));
            }
            if fields.iter().any(|g| g.name == name) {
                return self.reject("type (anonymous struct)", format!("`{}` names `{}` twice", rt, name));
            }
            // Rejoin the tokens: `[ ] const u8` is `[]const u8`.
            let mut zt = String::new();
            for tok in ty.split_whitespace() {
                let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
                if zt.ends_with(word) && tok.starts_with(word) {
                    zt.push(' ');
                }
                zt.push_str(tok);
            }
            if !self.zig_spelled(&zt) {
                return self.reject("type (anonymous struct)", format!("`{}`: field type `{}` is not a Zig type", rt, zt));
            }
            let ty = self.lty(&zt)?;
            let (fs, fa) = self.size_align(&ty)?;
            let off = size.div_ceil(fa) * fa;
            size = off + fs;
            align = align.max(fa);
            fields.push(Field { name: name.to_string(), ty, off, default: None });
        }
        let id = self.structs.len() as u32;
        self.structs.push(StructDef { name: key.clone(), fields, size: Some(size.div_ceil(align) * align), align, fail: None });
        self.struct_ids.insert(key, id);
        Ok(LTy::Struct(id))
    }

    /// Whether Zig reads `t` as a type: its own scalars, a struct or enum
    /// declared here, and `?`, `*`, `*const`, `[]`, `[]const` and `[N]` over
    /// one of those.
    fn zig_spelled(&self, t: &str) -> bool {
        let t = t.trim();
        for p in ["?", "*const ", "*", "[]const ", "[]"] {
            if let Some(r) = t.strip_prefix(p) {
                return self.zig_spelled(r);
            }
        }
        if let Some((len, elem)) = t.strip_prefix('[').and_then(|r| r.split_once(']')) {
            let len = len.trim();
            let ok = !len.is_empty() && len.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            return ok && self.zig_spelled(elem);
        }
        Ty::from_name(t).is_some() || self.struct_nodes.contains_key(t) || self.enum_nodes.contains_key(t)
    }

    // ---------------------------------------------------------------- scopes

    fn lookup(&self, name: &str) -> Option<Binding> {
        for s in self.scopes.iter().rev() {
            if let Some(b) = s.get(name) {
                return Some(b.clone());
            }
        }
        if self.comptime {
            return None;
        }
        self.mod_vars.get(name).map(|p| Binding::Mem(p.clone()))
    }

    /// A module-level `var`: writable memory holding its initial value. t27c
    /// emits it as a Zig container-level `var`, whose initializer must be a
    /// compile-time value, and `t27c test-report` runs every test in a fresh
    /// process, so each test starts from that value (`ExprKind::Global`).
    fn module_var(&mut self, node: &'a Node) -> R<()> {
        self.see(node);
        if node.line == 0 {
            let at = self.src.and_then(|s| ["var", "pub var"].iter().filter_map(|k| header_line(s, k, &node.name)).min());
            if let Some(l2) = at {
                self.line = l2;
            }
        }
        let name = node.name.clone();
        let ann = node.extra_type.trim();
        if ann.is_empty() {
            return self.reject(
                "VarDecl(module, untyped)",
                format!("module-level var `{}` has no type; Zig refuses a comptime_int var", name),
            );
        }
        let Some(init) = node.children.first() else {
            return self.reject("VarDecl(module)", format!("module-level var `{}` has no value", name));
        };
        if is_undefined(init) {
            return self.reject(
                "VarDecl(module, undefined)",
                format!("module-level var `{}` = undefined", name),
            );
        }
        let t = self.lty(ann)?;
        match &t {
            LTy::S(_) | LTy::Enum(..) | LTy::Struct(_) | LTy::Arr(..) if !self.holds_str(&t)? => {}
            _ => {
                let d = self.type_name(&t);
                return self.reject(
                    "VarDecl(module, pointer/str/slice)",
                    format!("module-level var `{}` of type {}", name, d),
                );
            }
        }
        let (size, _) = self.size_align(&t)?;
        let mut buf = vec![0u8; size as usize];
        let saved_scopes = std::mem::take(&mut self.scopes);
        let saved_ct = std::mem::replace(&mut self.comptime, true);
        let r = match &t {
            LTy::S(ty) => {
                let ty = *ty;
                let mut f = || -> R<()> {
                    let v = self.expr_as(init, &t)?;
                    let e = self.coerce(v, ty)?;
                    let ExprKind::Const(c) = e.kind else {
                        return self.reject(
                            "VarDecl(module)",
                            format!("module-level var `{}` is not a compile-time integer or bool", name),
                        );
                    };
                    for i in 0..ty.bytes() as usize {
                        buf[i] = ((c as u64) >> (8 * i)) as u8;
                    }
                    Ok(())
                };
                f()
            }
            _ => self.const_fill(init, &t, &mut buf, 0),
        };
        self.comptime = saved_ct;
        self.scopes = saved_scopes;
        r?;
        self.globals_init.push(buf);
        let k = (self.globals_init.len() - 1) as u32;
        let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Global(k) };
        self.mod_vars.insert(name, Place { addr, off: 0, ty: t, mutable: true, temp: None });
        Ok(())
    }

    /// A local, assigned parameter or capture with the name of a module-level
    /// var. In a test the reference's Zig refuses it ("local variable shadows
    /// declaration"); in a fn it renames the local to `<name>_lv` for every
    /// mention in the fn, including the ones meant for the module var. Either
    /// way there is no reference verdict to match, so t27b refuses it.
    fn no_var_shadow(&mut self, name: &str) -> R<()> {
        if self.mod_vars.contains_key(name) {
            return self.reject(
                "StmtLocal(shadows module var)",
                format!("`{}` shadows a module-level var (a compile error in the reference)", name),
            );
        }
        Ok(())
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
        if self.alias_target(name).is_some() {
            return self.reject("ExprIdentifier(type as value)", format!("`{}` is a type alias", name));
        }
        if !self.resolving.insert(name.to_string()) {
            return self.reject("ConstDecl", format!("`{}` refers to itself", name));
        }
        let saved_line = self.line;
        let saved_scopes = std::mem::take(&mut self.scopes);
        let saved_ct = std::mem::replace(&mut self.comptime, true);
        self.see(node);
        let r = self.global_value(node);
        self.comptime = saved_ct;
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
                t @ (LTy::Struct(_) | LTy::Arr(..)) => Some(t),
                LTy::Str => {
                    return match self.expr(init)? {
                        v @ (Val::S(..) | Val::Poison) => Ok(v),
                        _ => self.reject(
                            "ConstDecl",
                            format!("`{}` is a module-level str that is not a string literal", node.name),
                        ),
                    };
                }
                LTy::Ptr(..) => {
                    return self.reject("ConstDecl", format!("`{}` is a module-level pointer", node.name))
                }
                LTy::Slice(..) => {
                    return self.reject("ConstDecl(slice)", format!("`{}` is a module-level slice", node.name))
                }
                t @ LTy::Opt(_) => Some(t),
                t @ LTy::Enum(..) => {
                    let v = self.expr_as(init, &t)?;
                    return match v {
                        Val::P(Expr { kind: ExprKind::Const(_), .. }, _) | Val::Poison => Ok(v),
                        _ => self.reject(
                            "ConstDecl",
                            format!("`{}` is not a compile-time enum value", node.name),
                        ),
                    };
                }
                LTy::S(_) => None,
            }
        } else if init.kind == NodeKind::ExprStructLit && !init.name.is_empty() {
            Some(self.struct_lit_ty(init)?)
        } else {
            None
        };
        if let Some(t) = st {
            if self.holds_str(&t)? {
                return self.const_agg(init, &t);
            }
            return self.rodata(init, t);
        }
        let v = if node.extra_type.trim().is_empty() {
            self.expr(init)?
        } else {
            let ty = self.ty(&node.extra_type)?;
            let v = self.expr_as(init, &LTy::S(ty))?;
            Val::E(self.coerce(v, ty)?)
        };
        match &v {
            Val::Ct(_) | Val::Cf(..) | Val::S(..) | Val::A(..) | Val::Poison => Ok(v),
            Val::E(e) if matches!(e.kind, ExprKind::Const(_)) => Ok(v),
            Val::P(e, LTy::Enum(..)) if matches!(e.kind, ExprKind::Const(_)) => Ok(v),
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
        self.float_locals.clear();
        self.vars.clear();
        self.ltys.clear();
        self.slots.clear();
        self.sret = None;
        self.addr_taken.clear();
        scan_addr_taken(body, &mut self.addr_taken);
        self.collect_ref_vars(body);
        self.slice_locals.clear();
        self.tuple_locals.clear();
        self.dead_lits.clear();
        self.discards.clear();
        let mut arrays = HashSet::new();
        array_locals(body, &mut arrays);
        if !arrays.is_empty() {
            let mut calls = Vec::new();
            calls_in(body, &mut calls);
            for c in &calls {
                let Some(sig) = self.sigs.get(&c.name) else { continue };
                for (i, a) in c.children.iter().enumerate() {
                    if a.kind != NodeKind::ExprIdentifier || !arrays.contains(&a.name) {
                        continue;
                    }
                    // `[]const u8` is a slice of `u8` to the reference too.
                    let elem = match sig.params.get(i) {
                        Some(LTy::Slice(elem, _)) => (**elem).clone(),
                        Some(LTy::Str) => LTy::S(Ty::U8),
                        _ => continue,
                    };
                    if elem == LTy::Str || !has_brackets(&elem) {
                        self.slice_locals.insert(a.name.clone(), elem);
                    }
                }
            }
            for name in &arrays {
                if self.slice_locals.contains_key(name) {
                    continue;
                }
                let mut decls = Vec::new();
                decls_of(body, name, &mut decls);
                let [d] = decls[..] else { continue };
                if d.kind != NodeKind::StmtLocal
                    || d.extra_mutable
                    || !d.extra_type.trim().is_empty()
                    || mutated(body, name)
                {
                    continue;
                }
                // Every node that names it, less the declaration itself.
                let uses = name_count(body, name) - 1;
                if uses == 0 {
                    if plain_lit(&d.children[0]) {
                        self.dead_lits.insert(name.clone());
                    }
                    continue;
                }
                let (mut ty, mut args, mut same) = (None::<LTy>, 0usize, true);
                for c in &calls {
                    let Some(sig) = self.sigs.get(&c.name) else { continue };
                    for (i, a) in c.children.iter().enumerate() {
                        if a.kind != NodeKind::ExprIdentifier || a.name != *name {
                            continue;
                        }
                        if let Some(t @ LTy::Arr(..)) = sig.params.get(i) {
                            args += 1;
                            match &ty {
                                None => ty = Some(t.clone()),
                                Some(u) if u != t => same = false,
                                _ => {}
                            }
                        }
                    }
                }
                if let (Some(t), true, true) = (ty, same, args == uses) {
                    self.tuple_locals.insert(name.clone(), t);
                }
            }
        }
        self.addr_lit_locals(body);
        self.scopes.clear();
        self.scopes.push(HashMap::new());
        self.loop_depth = 0;
    }

    fn function(&mut self, n: &Node) -> R<Func> {
        self.see(n);
        self.begin_body(&n.children);
        self.enter_float_names(n);
        self.in_test = false;
        self.unanalyzed_fn = !self.analyzed.contains(&n.name);
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
                t if is_agg(t) => LTy::Ptr(Box::new(t.clone()), false),
                t => t.clone(),
            };
            ids.push(self.hidden_var(pname, t));
        }
        if ret.as_ref().is_some_and(is_agg) {
            self.sret = Some(self.hidden_var("%sret", LTy::Ptr(Box::new(ret.clone().unwrap()), true)));
        }
        let nparams = self.vars.len();
        let mut body = Vec::new();
        for (pname, _) in n.params.iter() {
            // The reference renames a parameter that shadows a module-level
            // declaration (`x_arg`), except one the body assigns: that one
            // is rebound as a local `var x = x_arg;`, which shadows.
            if mutated(&n.children, pname) {
                self.no_var_shadow(pname)?;
            }
        }
        for (i, (pname, _)) in n.params.iter().enumerate() {
            let var = Expr { ty: Ty::Ptr, kind: ExprKind::Var(ids[i]) };
            match &params[i] {
                // A struct or str argument is the caller's memory, read-only.
                t if is_agg(t) => {
                    let p = Place { addr: var, off: 0, ty: params[i].clone(), mutable: false, temp: None };
                    let p = self.param_place(pname, p, &mut body)?;
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
        for (pname, _) in n.params.iter() {
            let ok = name_mentions(&n.children, pname) == discard_count(&n.children, pname);
            self.discards.insert(pname.clone(), ok);
        }
        // #6315: the reference returns a non-void fn's tail expression
        // (`fn f(v: u32) -> u32 { v + 1 }`). In a fn nothing analyzed
        // reaches, Zig never sees the body and the statement keeps its stub.
        if !self.unanalyzed_fn && returns_value(&n.extra_return_type) {
            mark_tail_returns(&n.children, &mut self.tail_returns);
        }
        let lowered = self.stmts(&n.children);
        self.tail_returns.clear();
        body.extend(lowered?);
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
        let (k, what) = if invariant { ("InvariantBlock", "invariant") } else { ("TestBlock", "test") };
        self.block_body(n, k, what, invariant)
    }

    /// A `bench` block. t27c's Zig backend emits one as a plain
    /// `fn bench_<name>() void { ... }` that nothing calls, so `zig test`
    /// neither runs it nor counts it. Its body is lowered here with the test
    /// binding rule (the one t27c's `gen_bench_block` uses) and compiled by the
    /// caller, but never run: a construct t27b cannot lower in it rejects the
    /// file under that construct's own name.
    ///
    /// A prose clause in it (`measure: nanoseconds to f(x)`, `target: < 100ns`)
    /// reaches here as a childless StmtExpr named `<clause>:` holding the text;
    /// t27c writes `// NOT LOWERED: empty statement` for it (gen_stmt, T43),
    /// so it lowers to nothing here too.
    fn bench(&mut self, n: &Node) -> R<Func> {
        self.in_bench = true;
        let f = self.block_body(n, "BenchBlock", "bench", false);
        self.in_bench = false;
        let mut f = f?;
        f.is_test = false;
        Ok(f)
    }

    fn block_body(&mut self, n: &Node, k: &str, what: &str, invariant: bool) -> R<Func> {
        self.see(n);
        if n.line == 0 {
            if let Some(l) = self.src.and_then(|s| header_line(s, what, &n.name)) {
                self.line = l;
            }
        }
        if n.extra_field == "partial" {
            return self.reject(
                k,
                format!("{} `{}` was only partially parsed by the front-end", what, n.name),
            );
        }
        if let Some(name) = self.leaky.get(&(n as *const Node as usize)).cloned() {
            return self.reject(
                "ExprIdentifier(renamed module var)",
                format!(
                    "{} `{}` names module-level var `{}`, which the reference renames to the undeclared `{}_arg` after a fn with a parameter `{}`",
                    what, n.name, name, name, name
                ),
            );
        }
        self.begin_body(&n.children);
        if invariant || self.in_bench {
            self.ref_vars.clear();
        }
        self.in_test = true;
        // A bench is an uncalled `fn bench_<name>() void` in the reference's
        // Zig, so `zig test` never analyzes its body (see `analyzed_fns`).
        self.unanalyzed_fn = self.in_bench;
        self.comptime = invariant;
        self.ret = None;
        self.ret_poison = false;
        self.test_assigns.clear();
        count_assigns(&n.children, &mut self.test_assigns);
        if invariant {
            for s in &n.children {
                if self.invariant_predicate(s) {
                    self.invariant_preds.insert(s as *const Node as usize);
                }
            }
        }
        let body = self.stmts(&n.children);
        self.invariant_preds.clear();
        self.in_test = false;
        self.comptime = false;
        let body = body?;
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

    /// #6315: is this top-level statement of a brace invariant a predicate
    /// the reference emits as `assert(<expr>)` (t27c `invariant_predicate`)?
    /// A binary or unary expression, a name, an index, a field access, the
    /// literal `true` or `false`, or a call to a module fn declared `-> bool`.
    /// Any other statement keeps its own form.
    fn invariant_predicate(&self, s: &Node) -> bool {
        if s.kind != NodeKind::StmtExpr || s.children.len() != 1 {
            return false;
        }
        let e = &s.children[0];
        match e.kind {
            // `try f()` is wrapped too (`assert(try f())`), and Zig refuses
            // it either way; it stays with `try_stmt`, which says why.
            NodeKind::ExprUnary => e.extra_op.trim() != "try",
            NodeKind::ExprBinary
            | NodeKind::ExprIdentifier
            | NodeKind::ExprIndex
            | NodeKind::ExprFieldAccess => true,
            NodeKind::ExprLiteral => e.value == "true" || e.value == "false",
            NodeKind::ExprCall => self.bool_fns.contains(&e.name),
            _ => false,
        }
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
                // t27c's Zig backend drops the capture, so the body names an
                // undeclared identifier: no reference to agree with.
                if !n.params.is_empty() {
                    let names: Vec<&str> = n.params.iter().map(|(a, _)| a.as_str()).collect();
                    return self.reject("StmtIf(capture)", format!("`|{}|`", names.join(", ")));
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
            NodeKind::StmtFor => self.for_array(n, out),
            // `for i in a..b { ... }`: t27c's Zig backend prints
            // `for (a..b) |i| { ... }`, the same loop as `for (a..b) |i|`.
            NodeKind::StmtForRange => {
                if n.children.len() != 3 {
                    return self.reject("StmtForRange", "unexpected shape".into());
                }
                let body_node = &n.children[2];
                if body_node.kind != NodeKind::Module || body_node.name != "body" {
                    return self.reject("StmtForRange", "unexpected body shape".into());
                }
                let capture = n.name.trim().to_string();
                self.for_range("StmtForRange", &capture, &n.children[0], &n.children[1], body_node, out)
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
            // #6315: a tail expression the reference returns.
            NodeKind::StmtExpr if self.tail_returns.contains(&(n as *const Node as usize)) => {
                self.return_stmt(n.children.first(), out)
            }
            NodeKind::ExprReturn => self.return_stmt(n.children.first(), out),
            // #6315: a brace invariant's bare predicate is checked the way an
            // `assert` is: false fails the reference's compile (comptime) and
            // fails the invariant here when it runs.
            NodeKind::StmtExpr if self.invariant_preds.contains(&(n as *const Node as usize)) => {
                let cond = self.cond(&n.children[0])?;
                let site = self.site(TrapKind::Assert, "assert".into(), Ty::Bool);
                out.push(Stmt::Assert { cond, site });
                Ok(())
            }
            // `defer <stmt>;` / `errdefer <stmt>;`: the parser keeps the
            // statement under a `scope_exit` marker, and t27c's Zig backend
            // renders a statement (as opposed to an expression) under it to
            // nothing -- `// NOT LOWERED: ... (T43)`. Zig never sees it, so
            // it neither runs nor is analyzed; nothing to lower here either.
            NodeKind::StmtExpr
                if n.extra_op == "scope_exit" && n.children.first().is_some_and(|c| !zig_renders(&c.kind)) =>
            {
                Ok(())
            }
            NodeKind::StmtExpr => match n.children.first() {
                Some(c) if c.kind == NodeKind::ExprCall => self.call_stmt(c, true, out),
                Some(c) if c.kind == NodeKind::ExprReturn => self.stmt(c, out),
                Some(c) if c.kind == NodeKind::ExprIdentifier && c.name == "undefined" => self.undefined_stmt(c, out),
                Some(c) if c.kind == NodeKind::ExprUnary && c.extra_op.trim() == "try" => self.try_stmt(c, out),
                Some(c) if is_value_stmt(c) => self.value_stmt(c, out),
                Some(c) => {
                    self.see(c);
                    let k = if c.kind == NodeKind::ExprUnary && !c.extra_op.is_empty() {
                        format!("ExprUnary({}) statement", c.extra_op.trim())
                    } else {
                        format!("{} statement", kind_name(c))
                    };
                    self.reject(&k, "expression statement".into())
                }
                // A bench's prose clause: nothing to run, and t27c emits a
                // comment for it. Outside a bench the same node would be a
                // test clause the front-end kept only as text, so it stays
                // rejected rather than silently dropping a check.
                None if self.in_bench && is_prose_clause(n) => Ok(()),
                None if is_prose_clause(n) => self.reject(
                    "StmtExpr",
                    format!("prose clause `{}` outside a bench, kept only as text", n.name),
                ),
                None => self.reject("StmtExpr", "empty statement".into()),
            },
            NodeKind::ExprCall => self.call_stmt(n, false, out),
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

    /// `for (a) |x| { ... }` over one array: a hidden index counts from 0 to
    /// the length, and `x` is the element it reaches, a constant. The array
    /// is evaluated once, before the loop; its elements are read as the loop
    /// reaches them.
    fn for_array(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        // `for x in xs { ... }` parses to this same node with the loop
        // variable copied into `name` (t27c's `parse_for_range`); the Zig
        // backend ignores `name` and prints `for (xs) |x|`. The parser builds
        // no labelled loops, so any other `name` stays refused.
        let in_form = n.params.len() == 1 && n.name == n.params[0].0;
        if !n.name.is_empty() && !in_form {
            return self.reject("StmtFor", format!("labelled loop `{}`", n.name));
        }
        if n.children.len() < 2 || n.params.is_empty() {
            return self.reject("StmtFor", "no iterable or no capture".into());
        }
        let k = n.children.len() - 1;
        let body_node = &n.children[k];
        if body_node.kind != NodeKind::Module || body_node.name != "body" {
            return self.reject("StmtFor", "unexpected body shape".into());
        }
        if k > 1 {
            return self.for_multi(&n.children[..k], &n.params, body_node, out);
        }
        if n.params.len() != 1 {
            return self.reject("StmtFor", "more than one capture for one iterable".into());
        }
        let iter = &n.children[0];
        if iter.kind == NodeKind::ExprBinary && iter.extra_op == ".." {
            if iter.children.len() != 2 {
                return self.reject("StmtFor(range)", "range without two bounds".into());
            }
            let capture = n.params[0].0.trim().to_string();
            return self.for_range("StmtFor(range)", &capture, &iter.children[0], &iter.children[1], body_node, out);
        }
        let (elem, base, len) = self.for_iterable(iter, out)?;
        let capture = n.params[0].0.trim().to_string();
        self.for_objects(vec![(capture, elem, base)], len, body_node, out)
    }

    /// One iterable of a `for` over memory (an array, a slice or a string):
    /// its element type, the base address and the length, read once into
    /// hidden variables before the loop where they are not constants.
    fn for_iterable(&mut self, iter: &Node, out: &mut Vec<Stmt>) -> R<(LTy, Expr, Expr)> {
        let p = match self.expr(iter)? {
            Val::Poison => return Err(()),
            Val::M(p) if matches!(p.ty, LTy::Arr(..)) => p,
            Val::A(t, elems) => self.materialize(t, elems)?,
            Val::P(e, LTy::Ptr(inner, m)) if matches!(*inner, LTy::Arr(..)) => {
                Place { addr: e, off: 0, ty: *inner, mutable: m, temp: None }
            }
            Val::M(p) if matches!(p.ty, LTy::Str | LTy::Slice(..)) => p,
            v @ Val::S(..) => match self.coerce_to(v, &LTy::Str)? {
                Val::M(p) => p,
                _ => return Err(()),
            },
            v => {
                let d = self.val_desc(&v);
                return self.reject("StmtFor", format!("`for` over {}", d));
            }
        };
        // The base address and the length: constants and the array for an
        // array; for a slice, both read from its header before the loop.
        Ok(match p.ty.clone() {
            LTy::Arr(elem, len) => {
                let mut base = addr_of(&p);
                if !pure_addr(&base) {
                    let h = self.hidden_var("%for_base", LTy::Ptr(Box::new(p.ty.clone()), false));
                    out.push(Stmt::Assign { var: h, value: base });
                    base = Expr { ty: Ty::Ptr, kind: ExprKind::Var(h) };
                }
                (*elem, base, Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) })
            }
            t => {
                let elem = match t {
                    LTy::Slice(elem, _) => *elem,
                    _ => LTy::S(Ty::U8),
                };
                let mut pin = Vec::new();
                let (hdr, off) = self.pin_header(&p, &mut pin)?;
                out.extend(pin);
                let hb = self.hidden_var("%for_base", LTy::Ptr(Box::new(elem.clone()), false));
                let hl = self.hidden_var("%for_len", LTy::S(Ty::U64));
                let ptr = Expr { ty: Ty::Ptr, kind: ExprKind::Load { addr: Box::new(hdr.clone()), off } };
                let n = Expr { ty: Ty::U64, kind: ExprKind::Load { addr: Box::new(hdr), off: off + 8 } };
                out.push(Stmt::Assign { var: hb, value: ptr });
                out.push(Stmt::Assign { var: hl, value: n });
                (elem, Expr { ty: Ty::Ptr, kind: ExprKind::Var(hb) }, Expr { ty: Ty::U64, kind: ExprKind::Var(hl) })
            }
        })
    }

    /// `for (lo..hi) |i| { ... }` as Zig runs it: both bounds are `usize`,
    /// evaluated once, `lo` first; the length `hi - lo` is computed before
    /// the first iteration and traps (integer overflow) when `hi < lo`; `i`
    /// is a `usize` constant per iteration. Bounds Zig cannot coerce to
    /// `usize` (a signed integer, a negative literal) are compile errors in
    /// the reference and are refused here.
    fn for_range(
        &mut self,
        construct: &str,
        capture: &str,
        lo_n: &Node,
        hi_n: &Node,
        body_node: &Node,
        out: &mut Vec<Stmt>,
    ) -> R<()> {
        if capture.is_empty()
            || capture.starts_with('*')
            || capture.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        {
            return self.reject(construct, format!("capture `{}`", capture));
        }
        self.no_var_shadow(capture)?;
        let usize_t = LTy::S(Ty::U64);
        let lo = self.expr_as(lo_n, &usize_t)?;
        let lo = self.coerce(lo, Ty::U64)?;
        let hi = self.expr_as(hi_n, &usize_t)?;
        let hi = self.coerce(hi, Ty::U64)?;
        if let (ExprKind::Const(a), ExprKind::Const(b)) = (&lo.kind, &hi.kind) {
            if a > b {
                // Zig: a comptime-known reversed range is a compile error.
                return self.reject(construct, format!("range {}..{} runs backwards", a, b));
            }
        }
        let lo = match lo.kind {
            ExprKind::Const(_) => lo,
            _ => {
                let h = self.hidden_var("%for_lo", usize_t.clone());
                out.push(Stmt::Assign { var: h, value: lo });
                Expr { ty: Ty::U64, kind: ExprKind::Var(h) }
            }
        };
        let site = self.site(TrapKind::Overflow, "- on usize (for range length)".into(), Ty::U64);
        let len = self.hidden_var("%for_len", usize_t.clone());
        out.push(Stmt::Assign {
            var: len,
            value: Expr {
                ty: Ty::U64,
                kind: ExprKind::Arith { op: ArithOp::Sub, lhs: Box::new(hi), rhs: Box::new(lo.clone()), site },
            },
        });
        let k = self.hidden_var("%for_k", usize_t.clone());
        let var_k = Expr { ty: Ty::U64, kind: ExprKind::Var(k) };
        out.push(Stmt::Assign { var: k, value: Expr { ty: Ty::U64, kind: ExprKind::Const(0) } });
        let mut body = Vec::new();
        self.scopes.push(HashMap::new());
        if capture != "_" {
            // lo + k <= hi, so the sum cannot wrap.
            let value = Expr {
                ty: Ty::U64,
                kind: ExprKind::Arith { op: ArithOp::AddW, lhs: Box::new(lo), rhs: Box::new(var_k.clone()), site: 0 },
            };
            let x = self.new_lvar(capture, usize_t, false);
            body.push(Stmt::Assign { var: x, value });
        }
        self.loop_depth += 1;
        let r = self.stmts(&body_node.children);
        self.loop_depth -= 1;
        self.scopes.pop();
        body.extend(r?);
        let cond = Expr {
            ty: Ty::Bool,
            kind: ExprKind::Cmp {
                op: CmpOp::Lt,
                lhs: Box::new(var_k.clone()),
                rhs: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Var(len) }),
            },
        };
        let next = Expr {
            ty: Ty::U64,
            kind: ExprKind::Arith {
                op: ArithOp::AddW,
                lhs: Box::new(var_k),
                rhs: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Const(1) }),
                site: 0,
            },
        };
        out.push(Stmt::While { cond, body, step: vec![Stmt::Assign { var: k, value: next }] });
        Ok(())
    }

    fn local(&mut self, n: &Node, out: &mut Vec<Stmt>) -> R<()> {
        let name = n.name.clone();
        if name.is_empty() && !n.extra_field.trim().is_empty() {
            return self.destructure_local(n, out);
        }
        if name.is_empty() || name.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            return self.reject("StmtLocal", format!("binding `{}`", name));
        }
        self.no_var_shadow(&name)?;
        let ann = n.extra_type.trim().to_string();
        // The Zig backend's `zig_declared_int_type`: only these spellings
        // pin `1 << n` in the initializer (not an alias that resolves to one).
        let decl_int = match ann.as_str() {
            "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize" => Ty::from_name(&ann),
            _ => None,
        };
        self.decl_int = decl_int;
        let r = self.local_with(n, name, ann, out);
        self.decl_int = None;
        r
    }

    /// `given xs = [a, b]` where `xs` is later passed as a slice: an array
    /// of the callee's element type, as t27c's Zig backend declares it
    /// (`var xs = [_]T{ a, b }`).
    fn slice_local(&mut self, init: &Node, name: String, elem: LTy, out: &mut Vec<Stmt>) -> R<()> {
        self.see(init);
        if init.children.is_empty() && init.extra_type.trim().is_empty() && init.extra_size.contains(';') {
            // The reference pastes `v;n` between `[_]T{` and `}`.
            return self.reject(
                "ExprArrayLiteral(repeat to slice)",
                format!("`{}` = `[{}]` is passed where a slice is declared", name, init.extra_size.trim()),
            );
        }
        let text = self.text_lit(init)?;
        let lit = text.as_ref().unwrap_or(init);
        let t = LTy::Arr(Box::new(elem), lit.children.len() as u32);
        let k = self.new_slot(&t)?;
        let dst = Place { addr: slot_expr(k), off: 0, ty: t, mutable: true, temp: None };
        self.init_array(lit, &dst, out)?;
        self.bind(&name, Binding::Mem(dst));
        Ok(())
    }

    /// `given p = [a, b]` where every use of `p` is an argument the callee
    /// declares as one array type `[N]T`: t27c's Zig backend writes
    /// `const p = .{ a, b }`, which coerces to `[N]T` at each call, so this
    /// is a `[N]T` built once, here.
    fn tuple_local(&mut self, init: &Node, name: String, t: LTy, out: &mut Vec<Stmt>) -> R<()> {
        self.see(init);
        if init.children.is_empty() && init.extra_type.trim().is_empty() && init.extra_size.contains(';') {
            // The reference pastes `v;n` between the braces of `.{ ... }`.
            return self.reject(
                "ExprArrayLiteral(repeat)",
                format!("`{}` = `[{}]` is passed where an array is declared", name, init.extra_size.trim()),
            );
        }
        let k = self.new_slot(&t)?;
        let dst = Place { addr: slot_expr(k), off: 0, ty: t, mutable: false, temp: None };
        self.init(init, dst.clone(), true, out)?;
        self.bind(&name, Binding::Mem(dst));
        Ok(())
    }

    fn local_with(&mut self, n: &Node, name: String, ann: String, out: &mut Vec<Stmt>) -> R<()> {
        let mutable = n.extra_mutable;
        let init = n.children.first().filter(|i| !is_undefined(i));
        let ann = match init.filter(|_| mutable && ann.is_empty()).and_then(int_lit_width) {
            Some(w) => w.to_string(),
            None => ann,
        };
        if !ann.is_empty() {
            let t = self.lty(&ann)?;
            if is_agg(&t) || self.addr_taken.contains(&name) {
                // In memory; the name is bound only after its initializer.
                let k = self.new_slot(&t)?;
                let mutable = mutable || self.ref_var_agg(&t, &name);
                let dst = Place { addr: slot_expr(k), off: 0, ty: t, mutable, temp: None };
                match init {
                    Some(i) => self.init(i, dst.clone(), true, out)?,
                    // A scalar with no value reads as zero, like a register.
                    None if !is_agg(&dst.ty) => {
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
            // `const x = undefined;`: the reference prints it as it is, plus
            // `_ = x;` when nothing reads it, and Zig accepts it. Nothing is
            // bound, so a read of `x` is still refused (#6967).
            None if !mutable
                && n.children.first().is_some_and(is_undefined)
                && self.lookup(&name).is_none()
                && !self.const_nodes.contains_key(&name) =>
            {
                return Ok(())
            }
            None => return self.reject("StmtLocal", format!("`{}` has neither type nor value", name)),
        };
        if self.addr_lit_local(init, &name, out)?.is_some() {
            return Ok(());
        }
        if init.kind == NodeKind::ExprArrayLiteral {
            if let Some(elem) = self.slice_locals.get(&name).cloned() {
                return self.slice_local(init, name, elem, out);
            }
            if !mutable && self.dead_lits.contains(&name) {
                self.see(init);
                return Ok(());
            }
            if let Some(t) = self.tuple_locals.get(&name).filter(|_| !mutable).cloned() {
                return self.tuple_local(init, name, t, out);
            }
        }
        if init.kind == NodeKind::ExprTuple && !mutable {
            return self.tuple_value_local(init, &name, out);
        }
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
            Val::Cf(q) => {
                if mutable {
                    return self.reject(
                        "StmtLocal",
                        format!("`var {}` initialised with an untyped float needs a type", name),
                    );
                }
                self.bind(&name, Binding::Const(Val::Cf(q)));
            }
            v => {
                let mutable = mutable || matches!(&v, Val::M(p) if self.ref_var_agg(&p.ty, &name));
                self.bind_value(&name, v, mutable, out)?
            }
        }
        Ok(())
    }

    /// Bind `name` to a fresh variable holding `v` (not a comptime integer).
    fn bind_value(&mut self, name: &str, v: Val, mutable: bool, out: &mut Vec<Stmt>) -> R<()> {
        let (t, value) = match v {
            Val::Poison => return Err(()),
            Val::Ct(_) | Val::Cf(..) => return self.reject("StmtLocal", format!("`{}` needs a type", name)),
            // A string literal stays a compile-time value, like Zig's
            // `const s = "abc";`.
            Val::S(..) if !mutable => {
                self.bind(name, Binding::Const(v));
                return Ok(());
            }
            Val::S(..) => {
                let v = self.coerce_to(v, &LTy::Str)?;
                return self.bind_value(name, v, mutable, out);
            }
            // A compile-time array, likewise.
            Val::A(..) if !mutable => {
                self.bind(name, Binding::Const(v));
                return Ok(());
            }
            Val::A(t, elems) => {
                let p = self.materialize(t, elems)?;
                return self.bind_value(name, Val::M(p), mutable, out);
            }
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
        if matches!(target.kind, NodeKind::ExprFieldAccess | NodeKind::ExprIndex) {
            let dst = self.lvalue(target)?;
            return self.store(dst, op, &n.children[1], out);
        }
        if target.kind == NodeKind::ExprTuple {
            return self.tuple_assign(n, out);
        }
        if target.kind != NodeKind::ExprIdentifier {
            let k = kind_name(target);
            return self.reject(&k, "assignment target".into());
        }
        let name = target.name.clone();
        // `_ = p;` for a parameter `p` does nothing at run time. At the top
        // of a fn body the reference's dead-store pass (`optimize`,
        // dead_store_elim_with) deletes it, and gen-zig then discards `p`
        // itself if nothing else reads it: the statement is as if absent.
        // In a nested block t27c prints it as written, and Zig's AstGen
        // (every fn, called or not) refuses it when the body also uses `p`
        // (a pointless discard). Discards alone, repeated or under a module
        // declaration's name (gen-zig renames that parameter), compile.
        let rhs = &n.children[1];
        if name == "_" && (op.is_empty() || op == "=") && rhs.kind == NodeKind::ExprIdentifier {
            if let Some(&ok) = self.discards.get(&rhs.name) {
                self.see(rhs);
                if !ok && self.scopes.len() > 1 {
                    return self.reject(
                        "StmtAssign(discard)",
                        format!(
                            "`_ = {0};` in a nested block where the body also uses `{0}` (a pointless discard)",
                            rhs.name
                        ),
                    );
                }
                return Ok(());
            }
        }
        // A module-level var written at the top of a test is a write to
        // module state, as in a fn body: since #6295 the reference no longer
        // binds it as a fresh `const` (`block_fresh_binding`), see #6911.
        match self.lookup(&name) {
            Some(Binding::Mem(dst)) => self.store(dst, op, &n.children[1], out),
            Some(Binding::Var { id, mutable }) if !matches!(self.ltys[id as usize], LTy::S(_)) => {
                if !mutable {
                    return self.reject("StmtAssign", format!("assignment to constant `{}`", name));
                }
                if !(op.is_empty() || op == "=") {
                    let d = if matches!(self.ltys[id as usize], LTy::Enum(..)) { "an enum" } else { "a pointer" };
                    return self.reject("StmtAssign", format!("`{}` on {}", op, d));
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
                let conv = |c: &Node| c.kind == NodeKind::ExprCall && matches!(c.name.as_str(), "@floatFromInt" | "@intFromFloat" | "@floatCast");
                let rhs = if (op.is_empty() || op == "=") && conv(&n.children[1]) {
                    self.expr_as(&n.children[1], &LTy::S(ty))?
                } else {
                    self.expr(&n.children[1])?
                };
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
            None if self.mod_vars.contains_key(&name) => {
                let _ = self.expr(&n.children[1]);
                self.var_at_comptime(&name)
            }
            None if self.poison_names.contains(&name) => {
                let _ = self.expr(&n.children[1]);
                self.unknown_name(&name)
            }
            None if self.in_test && (op.is_empty() || op == "=") && self.scopes.len() == 1 => {
                // Test-block binding form: the first plain assignment to a
                // fresh name declares it (t27c's Zig backend emits `const`, or
                // `var` when the test assigns the name again).
                let assigned = self.test_assigns.get(&name).copied().unwrap_or(0);
                let v = self.expr(&n.children[1])?;
                match v {
                    Val::Poison => self.bind(&name, Binding::Const(Val::Poison)),
                    v @ (Val::Ct(_) | Val::Cf(..)) if assigned <= 1 => {
                        self.bind(&name, Binding::Const(v));
                    }
                    Val::Ct(_) | Val::Cf(..) => {
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

    /// `undefined;`, the body stub a port leaves where plumbing was. t27c's
    /// Zig backend emits it as is, and Zig rejects it ("value of type
    /// '@TypeOf(undefined)' ignored") only in a fn it analyzes, i.e. one
    /// something analyzed references. In a fn nothing reaches, the reference
    /// compiles the file and runs every test, so the stub lowers to a trap
    /// that no test can reach. Anywhere a test, invariant or bench could
    /// reach it, the reference does not compile: refused, not matched.
    fn undefined_stmt(&mut self, c: &Node, out: &mut Vec<Stmt>) -> R<()> {
        self.see(c);
        if !self.unanalyzed_fn {
            return self.reject(
                "ExprIdentifier(undefined) statement",
                "`undefined;` where a test, invariant or bench reaches it: the reference's Zig does not compile it".into(),
            );
        }
        let site = self.site(TrapKind::Stub, "undefined;".into(), Ty::Bool);
        out.push(Stmt::Assert { cond: Expr { ty: Ty::Bool, kind: ExprKind::Const(0) }, site });
        Ok(())
    }

    /// `try <call>;` as a statement. t27c's Zig backend prints it as
    /// written. In a `test` (an error-returning fn in Zig) the error a
    /// failed `std.testing` check returns ends that test as failed and the
    /// next test still runs: here the check is an assert in the test, whose
    /// trap fails that test alone. Three checks lower:
    ///
    /// - `std.testing.expect(ok)`: `ok` must be a bool;
    /// - `std.testing.expectEqual(expected, actual)`: peer-typed, compared
    ///   with `!=` (floats as IEEE values), as `assert_eq`;
    /// - `std.testing.expectApproxEqAbs(expected, actual, tolerance)`: the
    ///   peer type must be f64; `std.math.approxEqAbs` first asserts
    ///   `tolerance >= 0` (a panic, which fails the test), then passes when
    ///   `expected == actual` or `|expected - actual| <= tolerance` (false
    ///   for a NaN). The operands are evaluated once, left to right.
    ///
    /// Everywhere else the reference decides by where the statement sits: a
    /// fn nothing analyzed reaches (or a bench, an uncalled fn) is never
    /// analyzed, so the statement lowers to a trap no test reaches; an
    /// `invariant` is a container-level `comptime` block, where Zig's
    /// AstGen refuses `try` ("'try' outside function scope"); a reachable
    /// fn returns a non-error type in the reference's Zig ("expected type
    /// 'void', found ...error_set"). Each refusal below is named and was
    /// confirmed BLOCKED under `t27c test-report`.
    fn try_stmt(&mut self, c: &Node, out: &mut Vec<Stmt>) -> R<()> {
        self.see(c);
        if c.children.len() != 1 {
            return self.reject("ExprUnary(try)", "unexpected shape".into());
        }
        // The parser leaves `try` itself without a line; its operand has one.
        self.see(&c.children[0]);
        if self.comptime {
            return self.reject(
                "ExprUnary(try) in an invariant",
                "t27c emits an invariant as a container-level `comptime` block, where Zig refuses `try` (outside function scope)".into(),
            );
        }
        if self.unanalyzed_fn {
            let site = self.site(TrapKind::Stub, "try in a fn the reference never analyzes".into(), Ty::Bool);
            out.push(Stmt::Assert { cond: Expr { ty: Ty::Bool, kind: ExprKind::Const(0) }, site });
            return Ok(());
        }
        if !self.in_test {
            return self.reject(
                "ExprUnary(try) in a fn",
                "`try` in a fn a test, invariant or bench reaches: t27c gives the fn a non-error return type and the reference's Zig does not compile it".into(),
            );
        }
        let e = &c.children[0];
        if e.kind != NodeKind::ExprCall {
            return self.reject(
                "ExprUnary(try) on a non-call",
                format!("`try` on {}", kind_name(e)),
            );
        }
        self.see(e);
        let args = &e.children;
        let arity = |n: usize| args.len() == n;
        match e.name.as_str() {
            "std.testing.expect" if arity(1) => {
                let cond = self.cond(&args[0])?;
                let site = self.site(TrapKind::Assert, "std.testing.expect".into(), Ty::Bool);
                out.push(Stmt::Assert { cond, site });
                Ok(())
            }
            "std.testing.expectEqual" if arity(2) => {
                self.assert_eq_stmt(&args[0], &args[1], "std.testing.expectEqual", out)
            }
            "std.testing.expectApproxEqAbs" if arity(3) => self.approx_eq_abs(args, out),
            n if n.starts_with("std.testing.") => self.reject(
                &format!("ExprUnary(try) {}", n),
                format!("`try {}` with {} arguments", n, args.len()),
            ),
            n if self.sigs.contains_key(n) => self.reject(
                "ExprUnary(try) on a non-error call",
                format!("`try {}()`: t27c gives `{}` a non-error return type and the reference's Zig does not compile it", n, n),
            ),
            n => self.reject("ExprUnary(try) statement", format!("`try {}(...)`", n)),
        }
    }

    /// `try std.testing.expectApproxEqAbs(expected, actual, tolerance);` in
    /// a test (see `try_stmt`).
    fn approx_eq_abs(&mut self, args: &[Node], out: &mut Vec<Stmt>) -> R<()> {
        const WHAT: &str = "std.testing.expectApproxEqAbs";
        let mut vals = Vec::with_capacity(3);
        for a in args {
            vals.push(self.expr(a)?);
        }
        if vals.iter().any(|v| v.is_poison()) {
            return Err(());
        }
        // Zig's peer type of the three: f64 if any is a typed f64, and a
        // compile error if any other typed operand takes part or none is
        // typed (two comptime_floats).
        let mut typed: Option<Ty> = None;
        for v in &vals {
            match v {
                // f32 with f64 peers to f64 (Zig widens f32 implicitly).
                Val::E(e) if e.ty.is_float() => {
                    typed = Some(if typed == Some(Ty::F64) { Ty::F64 } else { e.ty })
                }
                Val::Ct(_) | Val::Cf(..) => {}
                v => {
                    let d = match v {
                        Val::E(e) => e.ty.name().to_string(),
                        v => self.val_desc(v),
                    };
                    return self.reject(
                        &format!("ExprCall({}) on a non-float", WHAT),
                        format!("operand of type {}: the reference's Zig refuses it (Unable to compare non floating point values)", d),
                    );
                }
            }
        }
        let Some(fty) = typed else {
            return self.reject(
                &format!("ExprCall({}) on comptime floats", WHAT),
                "no operand is a typed float: the reference's Zig refuses it (Cannot approximately compare two comptime_float values)".into(),
            );
        };
        // Each operand once, in order, into a temporary unless it is a
        // variable or a constant already.
        let mut ops = Vec::with_capacity(3);
        for v in vals {
            let e = self.coerce(v, fty)?;
            let e = if matches!(e.kind, ExprKind::Var(_) | ExprKind::Const(_)) {
                e
            } else {
                let k = self.new_slot(&LTy::S(fty))?;
                out.push(Stmt::Store { addr: slot_expr(k), off: 0, value: e });
                Expr { ty: fty, kind: ExprKind::Load { addr: Box::new(slot_expr(k)), off: 0 } }
            };
            ops.push(e);
        }
        let (x, y, tol) = (ops[0].clone(), ops[1].clone(), ops[2].clone());
        let cmp = |op: CmpOp, l: Expr, r: Expr| Expr {
            ty: Ty::Bool,
            kind: ExprKind::Cmp { op, lhs: Box::new(l), rhs: Box::new(r) },
        };
        let sub = |l: Expr, r: Expr| Expr {
            ty: fty,
            kind: ExprKind::FArith { op: FOp::Sub, lhs: Box::new(l), rhs: Box::new(r) },
        };
        let zero = Expr { ty: fty, kind: ExprKind::Const(0) };
        let site = self.site(TrapKind::Assert, format!("{}: tolerance >= 0", WHAT), Ty::Bool);
        out.push(Stmt::Assert { cond: cmp(CmpOp::Ge, tol.clone(), zero), site });
        // |x - y| <= tol as (x - y <= tol) and (y - x <= tol): y - x is
        // exactly -(x - y), and a NaN difference fails both.
        let near = Expr {
            ty: Ty::Bool,
            kind: ExprKind::And(
                Box::new(cmp(CmpOp::Le, sub(x.clone(), y.clone()), tol.clone())),
                Box::new(cmp(CmpOp::Le, sub(y.clone(), x.clone()), tol)),
            ),
        };
        let cond = Expr { ty: Ty::Bool, kind: ExprKind::Or(Box::new(cmp(CmpOp::Eq, x, y)), Box::new(near)) };
        let site = self.site(TrapKind::Assert, WHAT.into(), Ty::Bool);
        out.push(Stmt::Assert { cond, site });
        Ok(())
    }

    /// `return v;`, `return;`, and a tail expression the reference
    /// returns (`tail_returns`).
    fn return_stmt(&mut self, v: Option<&Node>, out: &mut Vec<Stmt>) -> R<()> {
        // `return undefined;` (also a port's `fn stub() u32 { undefined; }`
        // since #6315) compiles in the reference and hands the caller an
        // undefined value. In a fn nothing analyzed reaches, Zig never sees
        // the body and the return is the stub trap no test can hit. Where
        // analysis reaches the fn, an aggregate result is left unwritten in
        // the caller's memory (`init`, as before #6315); a scalar one t27b
        // refuses rather than guess a value.
        if let Some(c) = v.filter(|c| c.kind == NodeKind::ExprIdentifier && c.name == "undefined") {
            if self.ret.is_some() && !self.ret_poison && self.unanalyzed_fn {
                self.see(c);
                let site = self.site(TrapKind::Stub, "`return undefined;` in a fn the reference never analyzes".into(), Ty::Bool);
                out.push(Stmt::Assert { cond: Expr { ty: Ty::Bool, kind: ExprKind::Const(0) }, site });
                return Ok(());
            }
            if self.ret.as_ref().is_some_and(|t| !is_agg(t)) && !self.ret_poison {
                self.see(c);
                return self.reject(
                    "ExprReturn(undefined)",
                    "returns `undefined`: the reference's Zig returns an undefined value, which t27b does not guess".into(),
                );
            }
        }
        if self.ret_poison {
            // Recovery mode: the return type was already rejected.
            if let Some(c) = v {
                let _ = self.expr(c)?;
            }
            return Ok(());
        }
        match (self.ret.clone(), v) {
            (None, None) => out.push(Stmt::Return(None)),
            (Some(t), Some(c)) if is_agg(&t) => {
                // Build the result in the caller's memory.
                let sret = Expr { ty: Ty::Ptr, kind: ExprKind::Var(self.sret.unwrap()) };
                let dst = Place { addr: sret.clone(), off: 0, ty: t, mutable: true, temp: None };
                if !self.slice_literal_return(c, &dst, out)? {
                    self.init(c, dst, true, out)?;
                }
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

    /// An expression statement whose value nothing uses, and which is
    /// neither a tail the reference returns (`tail_returns`) nor a brace
    /// invariant's predicate (`invariant_preds`): a bare comparison, a value
    /// before the end of a body, a tail expression in a test or a void fn.
    /// t27c's Zig backend emits it as is (`expr;`), and Zig
    /// rejects it ("value of type 'bool' ignored") wherever it analyzes the
    /// body. The same rule as `undefined;` (`undefined_stmt`) then decides:
    /// in a fn nothing analyzed reaches, the reference compiles the file and
    /// the statement lowers to a trap no test can hit; in a test, an
    /// invariant (a `comptime` block) or a reachable fn, the reference does
    /// not compile, so the file is refused, not matched.
    fn value_stmt(&mut self, c: &Node, out: &mut Vec<Stmt>) -> R<()> {
        self.see(c);
        if !self.unanalyzed_fn {
            let k = format!("{}(value ignored) statement", kind_name(c));
            let op = c.extra_op.trim();
            let what = if op.is_empty() { kind_name(c) } else { format!("{} `{}`", kind_name(c), op) };
            return self.reject(
                &k,
                format!(
                    "{} as a statement where a test, invariant or bench reaches it: t27c emits it as `expr;` (no implicit return) and the reference's Zig does not compile it (value ignored)",
                    what
                ),
            );
        }
        let site = self.site(TrapKind::Stub, "ignored value".into(), Ty::Bool);
        out.push(Stmt::Assert { cond: Expr { ty: Ty::Bool, kind: ExprKind::Const(0) }, site });
        Ok(())
    }

    /// A call as a statement. `discard`: it is the whole of an expression
    /// statement, the one form the reference prefixes with `_ = ` when the
    /// callee is a module fn that returns a value.
    fn call_stmt(&mut self, c: &Node, discard: bool, out: &mut Vec<Stmt>) -> R<()> {
        self.see(c);
        match c.name.as_str() {
            // `@compileAssert` is not a separate construct in the reference:
            // t27c's Zig backend lowers it through the very arm that lowers
            // `assert` (`node.name == "@compileAssert" || node.name ==
            // "assert"`), so both give the same `if (!(cond))` check. Inside
            // an `invariant` that check sits in a `comptime` block and a false
            // condition fails the reference's compile; here it fails the
            // invariant when run. Either way the file does not pass, and the
            // operands are held to the same comptime rules (`self.comptime`).
            "assert" | "@compileAssert" => {
                // `assert(cond, "msg")`: t27c's Zig backend emits
                // `if (!(cond)) @panic("msg")` and never evaluates the
                // message, so the verdict is the one-argument assert's. Only a
                // string-literal message is accepted: anything else would be a
                // value the reference silently drops, and matching that is a
                // claim this backend has not checked.
                match c.children.len() {
                    1 => {}
                    2 => {
                        let m = &c.children[1];
                        if m.kind != NodeKind::ExprLiteral || m.extra_kind != "string" {
                            return self.reject(
                                "ExprCall(assert with non-literal message)",
                                format!("assert message is a {:?}, not a string literal", m.kind),
                            );
                        }
                    }
                    n => {
                        return self.reject("ExprCall(assert with message)", format!("assert with {} arguments", n));
                    }
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
                self.assert_eq_stmt(&c.children[0], &c.children[1], "assert_eq", out)
            }
            _ => {
                let (call, ret, _) = self.call(c, None)?;
                // `f(x);` with a non-void module fn `f`: t27c prints
                // `_ = f(x);` (#6315) and the value is dropped. Any other
                // call that returns a value it prints as is, and Zig refuses
                // it ("value of type 'bool' ignored") wherever it analyzes
                // the body -- the same rule as `value_stmt`.
                let dropped = discard && self.value_fns.contains(&c.name);
                if let Some(t) = ret.filter(|_| !self.unanalyzed_fn && !dropped) {
                    let t = self.type_name(&t);
                    return self.reject(
                        "ExprCall(value ignored) statement",
                        format!(
                            "call to `{}`, which returns {}, as a statement where a test, invariant or bench reaches it: the reference's Zig does not compile it (value ignored)",
                            c.name, t
                        ),
                    );
                }
                out.push(Stmt::Eval(call));
                Ok(())
            }
        }
    }

    /// `assert_eq(a, b)`, and `try std.testing.expectEqual(a, b)` in a test:
    /// both compare `a` and `b` after peer typing, floats as IEEE values,
    /// and fail the test on a difference. `what` names the call in
    /// refusals and in the trap.
    fn assert_eq_stmt(&mut self, an: &Node, bn: &Node, what: &str, out: &mut Vec<Stmt>) -> R<()> {
        let (a, b) = self.operands(an, bn)?;
        if a.is_poison() || b.is_poison() {
            return Err(());
        }
        let (lhs, rhs) = match (a, b) {
            (Val::P(x, LTy::Enum(i, _)), Val::P(y, LTy::Enum(j, _))) if i == j => (x, y),
            (a @ Val::P(_, LTy::Enum(..)), b) | (a, b @ Val::P(_, LTy::Enum(..))) => {
                let (x, y) = (self.val_desc(&a), self.val_desc(&b));
                return self.reject("type mismatch", format!("{} on {} and {}", what, x, y));
            }
            (Val::Ct(x), Val::Ct(y)) => {
                let ty = if Ty::I64.fits(x) && Ty::I64.fits(y) {
                    Ty::I64
                } else if Ty::U64.fits(x) && Ty::U64.fits(y) {
                    Ty::U64
                } else {
                    return self.reject("ExprCall", format!("{} on out-of-range literals", what));
                };
                (
                    Expr { ty, kind: ExprKind::Const(x) },
                    Expr { ty, kind: ExprKind::Const(y) },
                )
            }
            (a, b) => self.peer(a, b, what)?,
        };
        let site = self.site(TrapKind::AssertEq, what.into(), lhs.ty);
        out.push(Stmt::AssertEq { lhs, rhs, site });
        Ok(())
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
            None if self.unresolved.contains(&c.name) => return self.unresolved_call(c),
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
            let v = self.arg_as(a, &params[i])?;
            args.push(match v {
                // By reference; the callee never writes it.
                Val::M(p) => addr_of(&p),
                v => self.reg(v)?,
            });
        }
        let mut temp = None;
        let ty = match &ret {
            None => Ty::Bool,
            Some(t) if is_agg(t) => {
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
        self.cond_val(v)
    }

    fn cond_val(&mut self, v: Val) -> R<Expr> {
        match v {
            // Recovery mode: stand in a constant so the branches are lowered.
            Val::Poison => Ok(Expr { ty: Ty::Bool, kind: ExprKind::Const(0) }),
            Val::E(e) if e.ty == Ty::Bool => Ok(e),
            Val::E(e) => self.reject("condition", format!("expected bool, found {}", e.ty.name())),
            Val::Ct(_) => self.reject("condition", "expected bool, found an integer literal".into()),
            Val::Cf(..) => self.reject("condition", "expected bool, found a float literal".into()),
            Val::P(_, ref t @ LTy::Enum(..)) => {
                let d = self.type_name(t);
                self.reject("condition", format!("expected bool, found {}", d))
            }
            Val::P(..) => self.reject("condition", "expected bool, found a pointer".into()),
            Val::M(ref p) if matches!(p.ty, LTy::Arr(..)) => self.reject("condition", "expected bool, found an array".into()),
            Val::A(..) => self.reject("condition", "expected bool, found an array".into()),
            Val::M(_) => self.reject("condition", "expected bool, found a struct".into()),
            Val::S(..) => self.reject("condition", "expected bool, found a string".into()),
        }
    }

    fn coerce(&mut self, v: Val, to: Ty) -> R<Expr> {
        match v {
            Val::Poison => Err(()),
            // A comptime_int becomes an f64 only when it is one exactly.
            Val::Ct(c) if to == Ty::F64 => {
                let f = c as f64;
                if f as i128 != c || f.abs() >= 1.0e30 {
                    return self.reject(
                        "literal out of range",
                        format!("{} is not exactly an f64", c),
                    );
                }
                Ok(Expr { ty: to, kind: ExprKind::Const(f64_bits(f)) })
            }
            // One rounding of the binary128 value, as Zig coerces it.
            Val::Cf(q) if to == Ty::F64 => match q.to_f64() {
                Some(f) => Ok(Expr { ty: to, kind: ExprKind::Const(f64_bits(f)) }),
                None => self.reject("literal out of range", format!("{:e} is outside the f64 range", q.approx())),
            },
            Val::Ct(_) | Val::Cf(..) if to == Ty::F32 => self.comptime_to_f32(v),
            Val::Cf(..) => self.reject(
                "type mismatch",
                format!("expected {}, found a float literal", to.name()),
            ),
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
                if to == Ty::F64 && e.ty == Ty::F32 {
                    return Ok(float::float_cast(e, to));
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
            Val::P(_, ref t @ LTy::Enum(..)) => {
                let d = self.type_name(t);
                self.reject("type mismatch", format!("expected {}, found {}", to.name(), d))
            }
            Val::P(..) => self.reject("type mismatch", format!("expected {}, found a pointer", to.name())),
            Val::M(ref p) if matches!(p.ty, LTy::Arr(..)) => {
                self.reject("type mismatch", format!("expected {}, found an array", to.name()))
            }
            Val::A(..) => self.reject("type mismatch", format!("expected {}, found an array", to.name())),
            Val::M(_) => self.reject("type mismatch", format!("expected {}, found a struct", to.name())),
            Val::S(..) => self.reject("type mismatch", format!("expected {}, found a string", to.name())),
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
                // f32 with f64: Zig's peer type is f64 (exact widening).
                if x.ty == Ty::F64 && y.ty == Ty::F32 {
                    return Ok((x, float::float_cast(y, Ty::F64)));
                }
                if x.ty == Ty::F32 && y.ty == Ty::F64 {
                    return Ok((float::float_cast(x, Ty::F64), y));
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
            (Val::E(x), v @ Val::Cf(..)) => {
                let t = x.ty;
                let y = self.coerce(v, t)?;
                Ok((x, y))
            }
            (v @ Val::Cf(..), Val::E(y)) => {
                let t = y.ty;
                let x = self.coerce(v, t)?;
                Ok((x, y))
            }
            (Val::Ct(_) | Val::Cf(..), Val::Ct(_) | Val::Cf(..)) => {
                self.reject("type mismatch", format!("`{}` on two untyped literals", what))
            }
            (Val::Poison, _) | (_, Val::Poison) => Err(()),
            (Val::A(..), _) | (_, Val::A(..)) => self.reject("type mismatch", format!("`{}` on an array", what)),
            (Val::M(p), _) | (_, Val::M(p)) if matches!(p.ty, LTy::Arr(..)) => {
                self.reject("type mismatch", format!("`{}` on an array", what))
            }
            (Val::M(_), _) | (_, Val::M(_)) => self.reject("type mismatch", format!("`{}` on a struct", what)),
            (Val::S(..), _) | (_, Val::S(..)) => self.reject("type mismatch", format!("`{}` on a string", what)),
            (Val::P(_, t @ LTy::Enum(..)), _) | (_, Val::P(_, t @ LTy::Enum(..))) => {
                let d = self.type_name(&t);
                self.reject("type mismatch", format!("`{}` on {}", what, d))
            }
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
                if let Some((e, v)) = name.split_once("::") {
                    if self.lookup(e).is_none() && self.enum_nodes.contains_key(e) {
                        let Some(id) = self.enum_id(e)? else { unreachable!() };
                        return self.enum_value(id, v);
                    }
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
                if (op == "==" || op == "!=") && (self.is_null(&n.children[0]) || self.is_null(&n.children[1])) {
                    return self.null_compare(&op, n);
                }
                let (x, y) = (&n.children[0], &n.children[1]);
                let lit = |n: &Node| n.kind == NodeKind::ExprEnumValue;
                let ordered = (self.names_variant(x) || self.names_variant(y)) && !lit(x) && !lit(y);
                let (a, b) = self.operands(x, y)?;
                if let Some(v) = self.opt_compare(&op, &a, &b)? {
                    return Ok(v);
                }
                if let Some(v) = self.enum_compare(&op, &a, &b, ordered)? {
                    return Ok(v);
                }
                // t27c's Zig backend rewrites `x * 2^k` as `x << k`, which
                // does not compile for a float `x`: no reference to agree with
                // (t27c issue #6284). Only where its optimizer reaches, though:
                // a test's `0.5 * 2` is printed as written.
                if op == "*"
                    && self.shifted_muls.contains(&(n as *const Node as usize))
                    && pow2_literal(y)
                    && matches!(&a, Val::Cf(..) | Val::E(Expr { ty: Ty::F64 | Ty::F32, .. })) {
                    return self.reject(
                        "ExprBinary(f64 * 2^k)",
                        format!("`* {}` on a float: t27c gen emits `<<` for it", y.value.trim()),
                    );
                }
                let a = if (op == "<<" || op == ">>") && is_char_literal(x) {
                    // t27c prints `'a' << @intCast(k)`: a comptime_int shifted
                    // by a runtime amount, which Zig refuses. Only a shift by
                    // an untyped constant stays comptime on both sides.
                    if !matches!(b, Val::Ct(_)) {
                        return self.reject(
                            "ExprBinary(char literal << >>)",
                            "char literal shifted by a typed or runtime amount".into(),
                        );
                    }
                    a
                } else if op == "<<" || op == ">>" {
                    match self.pin_shifted_literal(x, y, a, &b, op == "<<")? {
                        Ok(a) => a,
                        Err(done) => return Ok(done),
                    }
                } else {
                    a
                };
                self.binary(&op, a, b)
            }
            NodeKind::ExprEnumValue => {
                self.reject("ExprEnumValue", format!("enum literal `.{}` with no result type", n.name))
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
            NodeKind::ExprCall if n.name == "@intFromEnum" => self.int_from_enum(n, TagUse::Value),
            NodeKind::ExprCall if n.name == "@floatFromInt" || n.name == "@intFromFloat" || n.name == "@floatCast" => {
                self.reject(&format!("ExprCall({})", n.name), "with no result type".into())
            }
            NodeKind::ExprCall if n.name == "@enumFromInt" => {
                self.reject("ExprCall(@enumFromInt)", "with no enum result type".into())
            }
            // `@sqrt(x)` of a run-time float: gen-zig prints it as written.
            // A compile-time float (`@sqrt(2.0)`) Zig folds at its own
            // precision, and an integer it refuses: both stay refused.
            NodeKind::ExprCall if n.name == "@sqrt" && n.children.len() == 1 => match self.expr(&n.children[0])? {
                Val::E(e) if e.ty.is_float() => {
                    let ty = e.ty;
                    Ok(Val::E(Expr { ty, kind: ExprKind::FSqrt(Box::new(e)) }))
                }
                Val::Poison => Err(()),
                v => {
                    let d = self.val_desc(&v);
                    self.reject("ExprCall(@sqrt)", format!("of {}, not a run-time float", d))
                }
            },
            // `@as(T, x)`: `x` coerced to `T`.
            NodeKind::ExprCall if n.name == "@as" && n.children.len() == 2 && n.children[0].kind == NodeKind::ExprIdentifier => {
                // An identifier is printed as a value (`gf16.GF16`), not
                // through the type mapper: a scoped one names no declaration.
                if n.children[0].name.contains("::") {
                    let (construct, detail) = self.type_construct(n.children[0].name.trim());
                    return self.reject(&construct, detail);
                }
                let t = self.lty(&n.children[0].name)?;
                self.expr_as(&n.children[1], &t)
            }
            NodeKind::ExprCall => {
                if let Some(v) = self.std_mem_call(n)? {
                    return Ok(v);
                }
                if let Some(v) = self.len_call(n)? {
                    return Ok(v);
                }
                if let Some(v) = self.bare_abs(n)? {
                    return Ok(v);
                }
                let (call, ret, temp) = self.call(n, None)?;
                match ret {
                    Some(t) if is_agg(&t) => {
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
                if let Some(v) = self.float_as(&n.children[0], &v, to)? {
                    return Ok(v);
                }
                self.cast(v, to)
            }
            NodeKind::ExprFieldAccess => {
                // `.len` of a compile-time string is a constant.
                if n.name == "len"
                    && n.children.len() == 1
                    && matches!(n.children[0].kind, NodeKind::ExprIdentifier | NodeKind::ExprLiteral)
                {
                    if let Some(len) = self.peek_string(&n.children[0])? {
                        return Ok(Val::E(Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) }));
                    }
                }
                match self.member(n)? {
                    Ok(p) => self.place_value(p),
                    Err(v) => Ok(v),
                }
            }
            NodeKind::ExprIndex => match self.index(n)? {
                Ok(p) => self.place_value(p),
                Err(v) => Ok(v),
            },
            NodeKind::ExprArrayLiteral => {
                self.reject("ExprArrayLiteral", "array literal with no result type".into())
            }
            NodeKind::ExprIf => self.if_expr(n, None),
            NodeKind::ExprSwitch => self.switch_expr(n, None),
            NodeKind::ExprStructLit => {
                if n.name.is_empty() {
                    return self.reject("ExprStructLit", "anonymous `.{}` literal with no result type".into());
                }
                let t = self.struct_lit_ty(n)?;
                self.struct_temp(n, t)
            }
            _ => {
                let k = kind_name(n);
                self.reject(&k, String::new())
            }
        }
    }

    /// A literal or a name, evaluated only if it is a compile-time string:
    /// nothing is reported or emitted otherwise.
    fn peek_string(&mut self, n: &Node) -> R<Option<u64>> {
        if n.kind == NodeKind::ExprLiteral {
            return Ok((n.extra_kind == "string").then(|| n.value.len() as u64));
        }
        match self.lookup(&n.name) {
            Some(Binding::Const(Val::S(_, len))) => Ok(Some(len)),
            Some(_) => Ok(None),
            // Only a constant initialised with a string literal is evaluated
            // here; any rejection it has is its own.
            None if self.const_nodes.get(&n.name).is_some_and(|c| {
                c.children.first().is_some_and(|i| i.kind == NodeKind::ExprLiteral && i.extra_kind == "string")
            }) =>
            {
                match self.global(&n.name)? {
                    Some(Val::S(_, len)) => Ok(Some(len)),
                    _ => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// `v as to`. Lossless conversions (bool to an integer as 0 / 1
    /// included) are a `Widen`. A narrowing between two unsigned types
    /// truncates, like Zig's `@truncate`; every other one is checked, like
    /// `@intCast`, and traps when the value is outside `to`. In Wrap mode every
    /// narrowing truncates, as a C cast does.
    fn cast(&mut self, v: Val, to: Ty) -> R<Val> {
        // `x as f64` with an integer `x`: t27c's Zig backend emits
        // `@as(f64, @floatFromInt(x))` (its `is_float_expr` says the operand
        // is not a float), so this is exactly `@floatFromInt` with result
        // type f64. A float operand (`@floatCast`) and f64 to an integer
        // (`@intFromFloat`) stay refused below.
        // `x as f32` likewise (`@as(f32, @floatFromInt(x))`).
        if to.is_float() && (matches!(v, Val::Ct(_)) || matches!(&v, Val::E(e) if e.ty.is_int())) {
            let what = format!("ExprCast({})", to.name());
            return self.int_to_float(v, to, &what);
        }
        if to.is_float() || matches!(v, Val::Cf(..) | Val::E(Expr { ty: Ty::F64 | Ty::F32, .. })) {
            if v.is_poison() {
                return Err(());
            }
            return self.reject(
                if to == Ty::F32 || matches!(v, Val::E(Expr { ty: Ty::F32, .. })) { "ExprCast(f32)" } else { "ExprCast(f64)" },
                format!("`as` to or from a float (to {}); use @floatFromInt / @intFromFloat / @floatCast", to.name()),
            );
        }
        let e = match v {
            Val::Poison | Val::Cf(..) => return Err(()),
            Val::Ct(c) if to.is_int() => return Ok(Val::E(self.coerce(Val::Ct(c), to)?)),
            Val::Ct(_) => return self.reject("ExprCast(to bool)", "integer literal as bool".into()),
            Val::E(e) => e,
            Val::P(_, t @ LTy::Enum(..)) => {
                let d = self.type_name(&t);
                return self.reject("ExprCast", format!("enum `{}` as {}", d, to.name()));
            }
            Val::P(..) => return self.reject("ExprCast", format!("pointer as {}", to.name())),
            Val::M(_) | Val::A(..) => return self.reject("ExprCast", format!("struct or array as {}", to.name())),
            Val::S(..) => return self.reject("ExprCast", format!("string as {}", to.name())),
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

    /// A module-level `var` named where t27c's Zig backend needs a
    /// compile-time value: Zig cannot read or write a container-level `var`
    /// in a `comptime` block or in a module-level initializer.
    fn var_at_comptime<T>(&mut self, name: &str) -> R<T> {
        self.reject(
            "ExprIdentifier(var at comptime)",
            format!("module-level var `{}` in an invariant or a module-level initializer", name),
        )
    }

    /// A name that is neither in scope nor a module-level constant.
    fn unknown_name<T>(&mut self, name: &str) -> R<T> {
        if self.mod_vars.contains_key(name) {
            return self.var_at_comptime(name);
        }
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
        // The t27c lexer strips the quotes and unescapes, so the node holds
        // the string's own bytes (not trimmed: spaces are part of it).
        if n.extra_kind == "string" {
            return Ok(self.string(n.value.as_bytes()));
        }
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
                    // Zig types a char literal as comptime_int, the same as an
                    // untyped integer literal (t27c's optimizer never folds or
                    // propagates one: its `is_literal` reads integers only).
                    return match char_literal(s) {
                        Ok(v) => Ok(Val::Ct(v)),
                        Err(what) => self.reject(what, format!("`{}`", s)),
                    };
                } else if s.contains('.') || (s.contains(['e', 'E']) && !s.starts_with("0x")) {
                    match float::Q::parse(s) {
                        Ok(q) => {
                            let suffix = n.extra_type.trim();
                            if suffix.is_empty() {
                                return Ok(Val::Cf(q));
                            }
                            let ty = self.ty(suffix)?;
                            return Ok(Val::E(self.coerce(Val::Cf(q), ty)?));
                        }
                        Err(why) => return self.reject("ExprLiteral(float literal)", format!("`{}`: {}", s, why)),
                    }
                } else if let Some(v) = s.strip_prefix('-').and_then(parse_int) {
                    // `const X: i32 = -1;`: the t27c parser folds the minus
                    // into the literal, and its Zig backend prints it back
                    // as written -- Zig's comptime_int `-1`.
                    let suffix = n.extra_type.trim();
                    if suffix.is_empty() {
                        return Ok(Val::Ct(-v));
                    }
                    let ty = self.ty(suffix)?;
                    return Ok(Val::E(self.coerce(Val::Ct(-v), ty)?));
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
            ("-", Val::Cf(q)) => Ok(Val::Cf(q.neg())),
            // Float negation flips the sign bit (of a NaN and of 0 too).
            ("-", Val::E(e)) if e.ty.is_float() => {
                let ty = e.ty;
                if let ExprKind::Const(c) = e.kind {
                    return Ok(Val::E(Expr { ty, kind: ExprKind::Const(c ^ (1i128 << (ty.bits() - 1))) }));
                }
                Ok(Val::E(Expr { ty, kind: ExprKind::FNeg(Box::new(e)) }))
            }
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
            (op, Val::Cf(..)) => self.reject(
                &format!("ExprUnary({})", op),
                "on an untyped float literal".into(),
            ),
            (op, _) => self.reject(&format!("ExprUnary({})", op), "operand is a pointer, a struct or a string".into()),
        }
    }

    fn binary(&mut self, op: &str, a: Val, b: Val) -> R<Val> {
        if a.is_poison() || b.is_poison() {
            return Err(());
        }
        if self.is_str(&a) || self.is_str(&b) {
            return match op {
                "==" | "!=" => self.str_eq(op == "!=", a, b),
                _ => self.reject(&format!("ExprBinary({})", op), "on a string".into()),
            };
        }
        if matches!(a, Val::P(..) | Val::M(_) | Val::A(..)) || matches!(b, Val::P(..) | Val::M(_) | Val::A(..)) {
            let arr = |v: &Val| matches!(v, Val::A(..)) || matches!(v, Val::M(p) if matches!(p.ty, LTy::Arr(..)));
            let what = if arr(&a) || arr(&b) {
                "an array"
            } else if matches!(a, Val::M(_)) || matches!(b, Val::M(_)) {
                "a struct"
            } else {
                "a pointer"
            };
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
        let is_f = |v: &Val| matches!(v, Val::Cf(..) | Val::E(Expr { ty: Ty::F64 | Ty::F32, .. }));
        if is_f(&a) || is_f(&b) {
            return self.farith(op, a, b);
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

    /// The left operand of `lhs << rhs` / `lhs >> rhs` when `lhs` is an
    /// untyped integer literal and `rhs` is not a literal. t27c's Zig backend
    /// emits `@as(T, lhs) << @intCast(rhs)` there (compiler.rs,
    /// `shift_runtime_rhs`), so the literal gets a width even when `rhs` is a
    /// named constant: `T` is the declared type of the local being
    /// initialized when that names an integer type, otherwise `u32` when the
    /// literal fits and `u64` when it does not. Anything else (`-1 << n`,
    /// `(1 + 1) << n`, a literal past u64) has no width there either, stays a
    /// comptime_int, and is refused by `shift` when the amount is runtime.
    ///
    /// t27c folds literal-only trees and propagates literal-initialized local
    /// consts before it emits Zig, but only in some positions of a function
    /// body. So when `rhs` is built only from literals and local names and
    /// its value is constant here, the reference may see a literal (no pin,
    /// comptime_int) or not (pinned to `T`). That is accepted only where both
    /// readings give the same value, and refused by name otherwise.
    ///
    /// `Ok(lhs)` is the left operand to shift; `Err(v)` is the whole shift,
    /// already evaluated.
    fn pin_shifted_literal(&mut self, lhs: &Node, rhs: &Node, a: Val, b: &Val, left: bool) -> R<Result<Val, Val>> {
        let Val::Ct(c) = a else { return Ok(Ok(a)) };
        if lhs.kind != NodeKind::ExprLiteral || rhs.kind == NodeKind::ExprLiteral || !lhs.extra_type.trim().is_empty() {
            return Ok(Ok(Val::Ct(c)));
        }
        let ty = match self.decl_int {
            Some(t) => t,
            None if (0..=u32::MAX as i128).contains(&c) => Ty::U32,
            None if (0..=u64::MAX as i128).contains(&c) => Ty::U64,
            None => return Ok(Ok(Val::Ct(c))),
        };
        let amt = match b {
            Val::Ct(k) => Some(*k),
            Val::E(Expr { kind: ExprKind::Const(k), ty: kt }) if kt.is_int() => Some(*k),
            _ => None,
        };
        if let Some(k) = amt {
            if self.may_fold_to_literal(rhs) {
                let bits = ty.bits() as i128;
                let r = if (0..bits).contains(&k) && ty.fits(c) {
                    if left {
                        let r = c << k;
                        // Same value in the pinned type, in comptime_int,
                        // and in t27c's i64 fold.
                        Some(r).filter(|r| (r >> k) == c && *r <= i64::MAX as i128 && ty.wrap(*r) == *r)
                    } else {
                        Some(c >> k)
                    }
                } else {
                    None
                };
                return match r {
                    Some(r) => Ok(Err(Val::Ct(r))),
                    None => self.reject(
                        "ExprBinary(<< >>)",
                        "untyped literal shift whose width depends on t27c constant folding".into(),
                    ),
                };
            }
        }
        Ok(Ok(Val::E(self.coerce(Val::Ct(c), ty)?)))
    }

    /// Whether t27c's optimizer could turn `n` into a literal before codegen:
    /// a tree of literals and local names (which `const_propagate` may
    /// substitute) joined by unary and binary operators. A module-level
    /// const is never substituted, so any such leaf rules folding out.
    fn may_fold_to_literal(&self, n: &Node) -> bool {
        match n.kind {
            NodeKind::ExprLiteral => n.extra_type.trim().is_empty(),
            NodeKind::ExprIdentifier => self.lookup(&n.name).is_some(),
            NodeKind::ExprBinary | NodeKind::ExprUnary => {
                !n.children.is_empty() && n.children.iter().all(|c| self.may_fold_to_literal(c))
            }
            _ => false,
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
                // A typed constant amount is comptime-known in Zig too
                // (`@intCast(K)`): out of range is a compile error there, so
                // it is refused here, not trapped at run time.
                let amt = match amt {
                    Val::E(Expr { kind: ExprKind::Const(c), ty: aty }) if aty.is_int() => Val::Ct(c),
                    a => a,
                };
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
                        // Comptime-known on both sides: Zig folds it in
                        // the operand's type, and `<<` drops the bits that
                        // leave it (no overflow check, unlike `@shlExact`).
                        if let ExprKind::Const(v) = x.kind {
                            let r = if left { ty.wrap(((v as u128) << c) as i128) } else { v >> c };
                            return Ok(Val::E(Expr { ty, kind: ExprKind::Const(r) }));
                        }
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

    /// A compile-time value as a comptime_float: a Cf, or a Ct that is a
    /// binary128 exactly. None for anything else.
    fn as_cf(&mut self, v: &Val) -> R<Option<float::Q>> {
        match *v {
            Val::Cf(q) => Ok(Some(q)),
            Val::Ct(c) => match float::Q::from_int(c) {
                Some(q) => Ok(Some(q)),
                None => self.reject("literal out of range", format!("{} is not exactly a comptime_float", c)),
            },
            _ => Ok(None),
        }
    }

    /// `a op b` where either side is an f64 or a float literal: IEEE `+ - *
    /// /` only. Two compile-time operands fold in binary128, as Zig does.
    fn farith(&mut self, op: &str, a: Val, b: Val) -> R<Val> {
        let fop = match op {
            "+" => FOp::Add,
            "-" => FOp::Sub,
            "*" => FOp::Mul,
            "/" => FOp::Div,
            _ => return self.reject(&format!("ExprBinary({})", op), "on f64".into()),
        };
        let ct = |v: &Val| matches!(v, Val::Ct(_) | Val::Cf(..));
        if ct(&a) && ct(&b) {
            let (Some(x), Some(y)) = (self.as_cf(&a)?, self.as_cf(&b)?) else { unreachable!() };
            if fop == FOp::Div && y.is_zero() {
                return self.reject("ExprBinary", "constant division by zero".into());
            }
            return match float::Q::op(fop, x, y) {
                Some(r) => Ok(Val::Cf(r)),
                None => self.reject("ExprBinary", "constant float expression overflows f128".into()),
            };
        }
        let (x, y) = self.peer(a, b, op)?;
        if !x.ty.is_float() {
            return self.reject("type mismatch", format!("`{}` on {} and {}", op, x.ty.name(), y.ty.name()));
        }
        let ty = x.ty;
        if let (ExprKind::Const(p), ExprKind::Const(q)) = (&x.kind, &y.kind) {
            return Ok(Val::E(Expr { ty, kind: ExprKind::Const(fop.apply_bits(ty, *p, *q)) }));
        }
        Ok(Val::E(Expr {
            ty,
            kind: ExprKind::FArith { op: fop, lhs: Box::new(x), rhs: Box::new(y) },
        }))
    }

    fn compare(&mut self, op: CmpOp, a: Val, b: Val) -> R<Val> {
        if let (Val::Ct(x), Val::Ct(y)) = (&a, &b) {
            return Ok(Val::E(Expr {
                ty: Ty::Bool,
                kind: ExprKind::Const(op.holds(*x, *y) as i128),
            }));
        }
        if matches!(a, Val::Ct(_) | Val::Cf(..)) && matches!(b, Val::Ct(_) | Val::Cf(..)) {
            // Zig compares the binary128 values: `0.1 + 0.2 != 0.3` holds.
            let (Some(x), Some(y)) = (self.as_cf(&a)?, self.as_cf(&b)?) else { unreachable!() };
            let ord = float::Q::cmp(x, y) as i8 as i128;
            return Ok(Val::E(Expr {
                ty: Ty::Bool,
                kind: ExprKind::Const(op.holds(ord, 0) as i128),
            }));
        }
        // A comptime_int outside the run-time operand's range: Zig settles the
        // comparison at compile time (`x < 1 << 32` holds for every u32) and
        // still evaluates the run-time side.
        let ranged = match (&a, &b) {
            (Val::E(x), Val::Ct(c)) if x.ty.is_int() && !x.ty.fits(*c) => Some((op, *c)),
            (Val::Ct(c), Val::E(y)) if y.ty.is_int() && !y.ty.fits(*c) => Some((op.swap(), *c)),
            _ => None,
        };
        if let Some((op, c)) = ranged {
            let x = match (a, b) {
                (Val::E(x), _) | (_, Val::E(x)) => x,
                _ => unreachable!(),
            };
            // `x op c` with x always below c (or always above it).
            let held = if c > x.ty.max() { op.holds(0, 1) } else { op.holds(1, 0) };
            return Ok(Val::E(Expr {
                ty: Ty::Bool,
                kind: ExprKind::Seq {
                    stmts: vec![Stmt::Eval(x)],
                    value: Box::new(Expr { ty: Ty::Bool, kind: ExprKind::Const(held as i128) }),
                },
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
        // #7415: t27 also spells an optional after the type, `str?`, `Foo?`.
        // t27c's type mapper writes `T?` as Zig's `?T` whenever `T` is not
        // empty and does not itself start with `?`; read it the same way.
        if let Some(inner) = t.strip_suffix('?') {
            let inner = inner.trim();
            if !inner.is_empty() && !inner.starts_with('?') {
                return self.lty_in(&format!("?{}", inner), by_value);
            }
        }
        if let Some(rest) = t.strip_prefix('?') {
            let inner = self.lty_in(rest, by_value)?;
            if matches!(inner, LTy::Opt(_)) {
                return self.reject("type ?T(??T)", format!("`{}`: an optional of an optional", t));
            }
            // Its only non-null value is `undefined`, which Zig coerces to
            // an undefined optional, null flag included.
            if self.is_void(&inner) {
                return self.reject("type ?void", format!("`{}`", t));
            }
            return Ok(LTy::Opt(Box::new(inner)));
        }
        // A scoped path under `*` or `const`: t27c's type mapper replaces the
        // whole spelling with the last segment's mapping, dropping the pointer
        // or the const with it (`*gf16::GF16` is `u16`).
        if t.contains("::") && (t.contains('*') || t.contains("const ")) {
            let (construct, detail) = self.type_construct(t);
            return self.reject(&construct, detail);
        }
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
        // `void` as a parameter, a field or a pointee: Zig's zero-bit type,
        // here a struct with no fields. Its one value is `undefined`.
        if t == "void" {
            return Ok(LTy::Struct(self.void_struct()));
        }
        // #6533: a type spliced in by `use` keeps its module path, and t27c's
        // type mapper writes a path whose last segment it maps on its own as
        // that mapping: `gf16::GF16` is `u16`, as a bare `GF16` is, in a field,
        // a parameter, a result and a local's annotation.
        if is_scoped_gf16(t) {
            return Ok(LTy::S(Ty::U16));
        }
        // t27c's Zig backend spells all four `[]const u8`.
        if matches!(t, "str" | "&str" | "string" | "[]const u8") {
            return Ok(LTy::Str);
        }
        // `[]T`, `[]const T`. The elements are only pointed to, so a struct
        // may hold a slice of itself.
        if let Some(rest) = t.strip_prefix("[]") {
            let rest = rest.trim_start();
            let (inner, mutable) = match rest.strip_prefix("const ") {
                Some(r) => (r.trim(), false),
                None => (rest, true),
            };
            if !inner.is_empty() {
                let inner = self.lty_in(inner, false)?;
                if inner == LTy::S(Ty::U8) && !mutable {
                    return Ok(LTy::Str);
                }
                return Ok(LTy::Slice(Box::new(inner), mutable));
            }
        }
        if self.enum_nodes.contains_key(t) {
            let Some(id) = self.enum_id(t)? else { unreachable!() };
            return Ok(LTy::Enum(id, self.enums[id as usize].tag));
        }
        if self.struct_nodes.contains_key(t) {
            let id = self.struct_id(t);
            if by_value {
                self.layout(id)?;
            }
            return Ok(LTy::Struct(id));
        }
        if let Some(target) = self.alias_target(t) {
            return self.lty_in(target, by_value);
        }
        // t27's own spellings, mapped the way t27c's Zig backend maps them
        // (`t27_array_type_to_zig`): `[T; N]` is `[N]T`, and `[T]` -- one
        // bracket pair around the whole type, no `;` -- is the mutable slice
        // `[]T`.
        if let Some(inner) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            if close_of_open(t) == Some(t.len() - 1) {
                let inner = inner.trim();
                if inner == "*" || inner.starts_with('*') {
                    return self.reject("type [*]T", format!("`{}`: a many-item pointer", t));
                }
                if let Some(semi) = inner.rfind(';') {
                    let (elem, len) = (inner[..semi].trim(), inner[semi + 1..].trim());
                    if !elem.is_empty() && !len.is_empty() {
                        let n = self.array_len(t, len)?;
                        let elem = self.lty_in(elem, by_value)?;
                        return Ok(LTy::Arr(Box::new(elem), n));
                    }
                } else if inner.contains(':') {
                    return self.reject("type [K:V]", format!("`{}`: a map", t));
                } else if !inner.is_empty() {
                    let elem = self.lty_in(inner, false)?;
                    return Ok(LTy::Slice(Box::new(elem), true));
                }
            }
        }
        if t.starts_with("[*]") {
            return self.reject("type [*]T", format!("`{}`: a many-item pointer", t));
        }
        // `[N]T`, N a literal or the name of a compile-time integer.
        if let Some((len, elem)) = t.strip_prefix('[').and_then(|r| r.split_once(']')) {
            let (len, elem) = (len.trim(), elem.trim());
            if !len.is_empty() && !elem.is_empty() {
                let n = self.array_len(t, len)?;
                let inner = self.lty_in(elem, by_value)?;
                return Ok(LTy::Arr(Box::new(inner), n));
            }
        }
        let (construct, detail) = self.type_construct(t);
        self.reject(&construct, detail)
    }

    /// The length of array type `t`, spelled `len`.
    fn array_len(&mut self, t: &str, len: &str) -> R<u32> {
        let v = if let Some(c) = parse_int(len) {
            Some(c)
        } else if len.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !len.starts_with(|c: char| c.is_ascii_digit()) && len != "_" {
            let v = match self.lookup(len) {
                Some(Binding::Const(v)) => Some(v),
                Some(_) => None,
                None => self.global(len)?,
            };
            match v {
                Some(Val::Poison) => return Err(()),
                Some(Val::Ct(c)) => Some(c),
                Some(Val::E(Expr { kind: ExprKind::Const(c), ty })) if ty.is_int() => Some(c),
                _ => None,
            }
        } else {
            None
        };
        match v {
            Some(c) if (0..=u32::MAX as i128).contains(&c) => Ok(c as u32),
            Some(c) => self.reject("type [N]T", format!("`{}`: length {} out of range", t, c)),
            None => self.reject("type [N]T", format!("`{}`: length `{}` is not a compile-time integer", t, len)),
        }
    }

    /// The fieldless, zero-size struct that stands for `void`. Its key is
    /// a keyword, so no declared struct can take it.
    fn void_struct(&mut self) -> u32 {
        if let Some(&id) = self.struct_ids.get("void") {
            return id;
        }
        let id = self.structs.len() as u32;
        self.structs.push(StructDef { name: "void".to_string(), fields: Vec::new(), size: Some(0), align: 1, fail: None });
        self.struct_ids.insert("void".to_string(), id);
        id
    }

    fn is_void(&self, t: &LTy) -> bool {
        matches!(t, LTy::Struct(id) if self.struct_ids.get("void") == Some(id))
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

    /// The id of enum `name`, building it on first use; None when no enum
    /// of that name is declared. A refused declaration is reported where it
    /// is first built and, outside recovery mode, again at every later use.
    fn enum_id(&mut self, name: &str) -> R<Option<u32>> {
        if let Some(&id) = self.enum_ids.get(name) {
            return Ok(Some(id));
        }
        if let Some(r) = self.enum_fail.get(name).cloned() {
            if !self.recover {
                self.errors.push(Reject { line: self.line, ..r });
            }
            return Err(());
        }
        let Some(&node) = self.enum_nodes.get(name) else { return Ok(None) };
        let key = format!("enum {}", name);
        if !self.resolving.insert(key.clone()) {
            return self.reject("EnumDecl", format!("a tag of `{}` refers to `{}` itself", name, name));
        }
        let saved = self.line;
        let saved_scopes = std::mem::take(&mut self.scopes);
        self.line = 0;
        self.see(node);
        if node.line == 0 {
            if let Some(l) = self.src.and_then(|s| decl_line(s, name)) {
                self.line = l;
            }
        }
        let nerr = self.errors.len();
        let r = self.enum_build(node);
        self.scopes = saved_scopes;
        self.line = saved;
        self.resolving.remove(&key);
        match r {
            Ok(def) => {
                let id = self.enums.len() as u32;
                self.enums.push(def);
                self.enum_ids.insert(name.to_string(), id);
                Ok(Some(id))
            }
            Err(()) => {
                if let Some(e) = self.errors.get(nerr).cloned() {
                    self.enum_fail.insert(name.to_string(), e);
                }
                Err(())
            }
        }
    }

    /// Tags as t27c's Zig backend declares them: `enum(T)` when a tag type
    /// is written, `enum(i32)` when it is not but some variant has a value,
    /// plain `enum` (Zig's smallest unsigned tag, values 0, 1, ...) otherwise.
    fn enum_build(&mut self, node: &Node) -> R<EnumDef> {
        let name = node.name.clone();
        for v in &node.children {
            if v.kind != NodeKind::EnumVariant {
                return self.reject("EnumDecl", format!("`{}` member of kind {}", name, kind_name(v)));
            }
        }
        // t27c reads every word in the braces as a variant, so a method or a
        // declaration inside the enum shows up as variants named after its
        // keywords.
        if let Some(v) = node.children.iter().find(|v| matches!(v.name.as_str(), "fn" | "pub" | "const" | "var")) {
            return self.reject(
                "EnumDecl(method)",
                format!("`{}` declares something inside it (`{}`); only plain enums are supported", name, v.name),
            );
        }
        if node.children.iter().any(|v| v.name == "_") {
            return self.reject(
                "EnumDecl(non-exhaustive)",
                format!("`{}` has a `_` variant; only exhaustive enums are supported", name),
            );
        }
        if node.children.is_empty() {
            return self.reject("EnumDecl", format!("`{}` has no variants", name));
        }
        if let Some(v) = node.children.iter().find(|v| v.name.starts_with(|c: char| c.is_ascii_digit())) {
            return self.reject("EnumDecl", format!("`{}`: `{}` is not a variant name", name, v.name));
        }
        let n = node.children.len();
        let ann = node.extra_type.trim();
        let valued = node.children.iter().any(|v| !v.value.is_empty());
        let (tag, bits) = if !ann.is_empty() {
            match Ty::from_name(ann) {
                Some(t) if t.is_int() => (t, t.bits()),
                _ => {
                    return self.reject(
                        "EnumDecl(tag type)",
                        format!("`{}` has tag type `{}`", name, ann),
                    )
                }
            }
        } else if valued {
            (Ty::I32, 32)
        } else {
            let mut bits = 0u32;
            while (1u128 << bits) < n as u128 {
                bits += 1;
            }
            let tag = match bits {
                0..=8 => Ty::U8,
                9..=16 => Ty::U16,
                17..=32 => Ty::U32,
                _ => Ty::U64,
            };
            (tag, bits)
        };
        let mut variants: Vec<(String, i128)> = Vec::new();
        let mut next: i128 = 0;
        for v in &node.children {
            let val = if v.value.is_empty() {
                next
            } else {
                let (neg, digits) = match v.value.strip_prefix('-') {
                    Some(d) => (true, d),
                    None => (false, v.value.as_str()),
                };
                let c = if let Some(c) = parse_int(digits) {
                    c
                } else {
                    match self.global(digits)? {
                        Some(Val::Ct(c)) => c,
                        Some(Val::E(Expr { kind: ExprKind::Const(c), ty })) if ty.is_int() => c,
                        Some(Val::Poison) => return Err(()),
                        _ => {
                            return self.reject(
                                "EnumDecl",
                                format!("`{}.{}` = `{}` is not a compile-time integer", name, v.name, v.value),
                            )
                        }
                    }
                };
                if neg { -c } else { c }
            };
            if !tag.fits(val) {
                return self.reject(
                    "EnumDecl",
                    format!("`{}.{}` = {} does not fit the tag type {}", name, v.name, val, tag.name()),
                );
            }
            if variants.iter().any(|w| w.0 == v.name) {
                return self.reject("EnumDecl", format!("`{}` has two variants `{}`", name, v.name));
            }
            if let Some(w) = variants.iter().find(|w| w.1 == val) {
                return self.reject(
                    "EnumDecl",
                    format!("`{}.{}` and `{}.{}` have the same tag {}", name, w.0, name, v.name, val),
                );
            }
            variants.push((v.name.clone(), val));
            next = val + 1;
        }
        Ok(EnumDef { name, tag, bits, variants })
    }

    /// `E.v` (also `E::v`, and `.v` where an `E` is expected).
    fn enum_value(&mut self, id: u32, variant: &str) -> R<Val> {
        let def = &self.enums[id as usize];
        let tag = def.tag;
        match def.value(variant) {
            Some(c) => Ok(Val::P(Expr { ty: tag, kind: ExprKind::Const(c) }, LTy::Enum(id, tag))),
            None => {
                let e = def.name.clone();
                self.reject("ExprFieldAccess(enum)", format!("`{}` has no variant `{}`", e, variant))
            }
        }
    }

    /// `n` as a value of enum type `want`, when it is one of the forms that
    /// take their enum type from where they are used: `.v` and
    /// `@enumFromInt(x)`. None for any other expression.
    fn enum_literal(&mut self, n: &Node, want: &LTy) -> R<Option<Val>> {
        let LTy::Enum(id, tag) = *want else { return Ok(None) };
        if n.kind == NodeKind::ExprEnumValue {
            self.see(n);
            return self.enum_value(id, &n.name).map(Some);
        }
        if n.kind == NodeKind::ExprCall && n.name == "@enumFromInt" {
            self.see(n);
            if n.children.len() != 1 {
                return self.reject("ExprCall(@enumFromInt)", format!("{} arguments", n.children.len()));
            }
            let v = self.expr(&n.children[0])?;
            return self.enum_from_int(v, id, tag).map(Some);
        }
        Ok(None)
    }

    /// `@enumFromInt(v)` into enum `id`: a constant must be one of its tags
    /// (Zig refuses any other at compile time); a runtime value is checked
    /// and traps as `invalid enum value` when it is none of them.
    fn enum_from_int(&mut self, v: Val, id: u32, tag: Ty) -> R<Val> {
        let ename = self.enums[id as usize].name.clone();
        let lty = LTy::Enum(id, tag);
        let e = match v {
            Val::Poison => return Err(()),
            Val::Ct(c) => Expr { ty: tag, kind: ExprKind::Const(c) },
            Val::E(e) if e.ty.is_int() => e,
            v => {
                let d = self.val_desc(&v);
                return self.reject("ExprCall(@enumFromInt)", format!("{} into `{}`", d, ename));
            }
        };
        if let ExprKind::Const(c) = e.kind {
            if !self.enums[id as usize].variants.iter().any(|w| w.1 == c) {
                return self.reject("ExprCall(@enumFromInt)", format!("{} is no tag of `{}`", c, ename));
            }
            return Ok(Val::P(Expr { ty: tag, kind: ExprKind::Const(c) }, lty));
        }
        let mut tags: Vec<i128> = self.enums[id as usize].variants.iter().map(|w| w.1).collect();
        tags.sort();
        let (lo, hi) = (tags[0], *tags.last().unwrap());
        let n = tags.len() as i128;
        if hi - lo + 1 != n {
            return self.reject(
                "ExprCall(@enumFromInt)",
                format!("runtime value into `{}`, whose tags are not contiguous", ename),
            );
        }
        // idx = (x - lo) mod 2^64, which is below n exactly when x is a tag.
        let idx = if e.ty == Ty::U64 {
            if lo < 0 {
                return self.reject(
                    "ExprCall(@enumFromInt)",
                    format!("u64 value into `{}`, which has negative tags", ename),
                );
            }
            Expr {
                ty: Ty::U64,
                kind: ExprKind::Arith {
                    op: ArithOp::SubW,
                    lhs: Box::new(e),
                    rhs: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Const(lo) }),
                    site: 0,
                },
            }
        } else {
            if lo - i64::MIN as i128 > (1i128 << 64) - n {
                return self.reject(
                    "ExprCall(@enumFromInt)",
                    format!("runtime value into `{}`, whose tags are too large", ename),
                );
            }
            let x = if e.ty == Ty::I64 { e } else { Expr { ty: Ty::I64, kind: ExprKind::Widen(Box::new(e)) } };
            let d = Expr {
                ty: Ty::I64,
                kind: ExprKind::Arith {
                    op: ArithOp::SubW,
                    lhs: Box::new(x),
                    rhs: Box::new(Expr { ty: Ty::I64, kind: ExprKind::Const(lo) }),
                    site: 0,
                },
            };
            Expr { ty: Ty::U64, kind: ExprKind::Cast { arg: Box::new(d), site: 0 } }
        };
        let site = self.site(TrapKind::EnumTag, format!("@enumFromInt into {}", ename), Ty::U64);
        let checked = Expr {
            ty: Ty::U64,
            kind: ExprKind::Bounds {
                idx: Box::new(idx),
                len: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Const(n) }),
                site,
            },
        };
        let val = if lo == 0 {
            checked
        } else {
            Expr {
                ty: Ty::U64,
                kind: ExprKind::Arith {
                    op: ArithOp::AddW,
                    lhs: Box::new(checked),
                    rhs: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Const(Ty::U64.wrap(lo)) }),
                    site: 0,
                },
            }
        };
        let val = if tag == Ty::U64 { val } else { Expr { ty: tag, kind: ExprKind::Cast { arg: Box::new(val), site: 0 } } };
        Ok(Val::P(val, lty))
    }

    /// `@intFromEnum(x)`: the tag. A constant one of a tag type t27b has no
    /// type for is a compile-time integer; a runtime one is allowed only
    /// where `use` says its storage type gives Zig's answer.
    fn int_from_enum(&mut self, n: &Node, use_: TagUse) -> R<Val> {
        self.see(n);
        if n.children.len() != 1 {
            return self.reject("ExprCall(@intFromEnum)", format!("{} arguments", n.children.len()));
        }
        if n.children[0].kind == NodeKind::ExprEnumValue {
            return self.reject(
                "ExprCall(@intFromEnum)",
                format!("of `.{}`, which has no enum type here", n.children[0].name),
            );
        }
        let (e, id) = match self.expr(&n.children[0])? {
            Val::P(e, LTy::Enum(id, _)) => (e, id),
            Val::Poison => return Err(()),
            v => {
                let d = self.val_desc(&v);
                return self.reject("ExprCall(@intFromEnum)", format!("of {}, not an enum", d));
            }
        };
        let def = &self.enums[id as usize];
        let (exact, bits, ename) = (def.exact(), def.bits, def.name.clone());
        if exact {
            return Ok(Val::E(e));
        }
        if let ExprKind::Const(c) = e.kind {
            return Ok(Val::Ct(c));
        }
        match use_ {
            TagUse::Compare => Ok(Val::E(e)),
            TagUse::Want(want) => {
                let holds = want.is_int() && if want.signed() { want.bits() > bits } else { want.bits() >= bits };
                if !holds {
                    return self.reject(
                        "type mismatch",
                        format!("expected {}, found u{} (the tag of `{}`)", want.name(), bits, ename),
                    );
                }
                if want == e.ty {
                    Ok(Val::E(e))
                } else if want.can_widen_from(e.ty) {
                    Ok(Val::E(Expr { ty: want, kind: ExprKind::Widen(Box::new(e)) }))
                } else {
                    // The value is below 2^bits, so it fits `want`.
                    Ok(Val::E(Expr { ty: want, kind: ExprKind::Cast { arg: Box::new(e), site: 0 } }))
                }
            }
            TagUse::Value => self.reject(
                "ExprCall(@intFromEnum auto-tag)",
                format!(
                    "the tag type of `{}` is u{}, which t27b has no type for; compare it or give it an integer result type",
                    ename, bits
                ),
            ),
        }
    }

    /// Two operands of a comparison: `.v` takes the enum type of the other
    /// side, and `@intFromEnum` of any width may be compared.
    fn operands(&mut self, x: &Node, y: &Node) -> R<(Val, Val)> {
        let lit = |n: &Node| n.kind == NodeKind::ExprEnumValue;
        let tag = |n: &Node| n.kind == NodeKind::ExprCall && n.name == "@intFromEnum";
        let one = |l: &mut Self, n: &Node| -> R<Val> {
            if tag(n) {
                match l.int_from_enum(n, TagUse::Compare) {
                    Err(()) if l.recover => Ok(Val::Poison),
                    r => r,
                }
            } else {
                l.expr(n)
            }
        };
        if lit(x) && !lit(y) {
            let b = one(self, y)?;
            let a = self.enum_operand(x, &b)?;
            return Ok((a, b));
        }
        if lit(y) && !lit(x) {
            let a = one(self, x)?;
            let b = self.enum_operand(y, &a)?;
            return Ok((a, b));
        }
        let a = one(self, x)?;
        let b = one(self, y)?;
        Ok((a, b))
    }

    /// `.v` compared with `other`, which must be an enum.
    fn enum_operand(&mut self, n: &Node, other: &Val) -> R<Val> {
        match other {
            Val::P(_, t @ LTy::Enum(..)) => {
                let t = t.clone();
                match self.enum_literal(n, &t) {
                    Err(()) if self.recover => Ok(Val::Poison),
                    r => r.map(|v| v.unwrap()),
                }
            }
            Val::Poison => Ok(Val::Poison),
            v => {
                let d = self.val_desc(v);
                self.see(n);
                self.reject("type mismatch", format!("`.{}` compared with {}, not an enum", n.name, d))
            }
        }
    }

    /// A comparison with an enum operand, or None when neither is one.
    /// `ordered`: one side is `E.v` or `E::v`, for which t27c's Zig backend
    /// writes `@intFromEnum(a) < @intFromEnum(b)`; on two enum values Zig has
    /// no `<`.
    fn enum_compare(&mut self, op: &str, a: &Val, b: &Val, ordered: bool) -> R<Option<Val>> {
        let (ea, ia) = match a {
            Val::P(e, LTy::Enum(id, _)) => (Some(e), Some(*id)),
            _ => (None, None),
        };
        let (eb, ib) = match b {
            Val::P(e, LTy::Enum(id, _)) => (Some(e), Some(*id)),
            _ => (None, None),
        };
        if ia.is_none() && ib.is_none() {
            return Ok(None);
        }
        if a.is_poison() || b.is_poison() {
            return Err(());
        }
        let cmp = match op {
            "==" => CmpOp::Eq,
            "!=" => CmpOp::Ne,
            "<" => CmpOp::Lt,
            "<=" => CmpOp::Le,
            ">" => CmpOp::Gt,
            ">=" => CmpOp::Ge,
            _ => return self.reject(&format!("ExprBinary({}) on enum", op), "arithmetic on an enum".into()),
        };
        if ia != ib || ia.is_none() || ib.is_none() {
            let (x, y) = (self.val_desc(a), self.val_desc(b));
            return self.reject("type mismatch", format!("`{}` on {} and {}", op, x, y));
        }
        if !matches!(cmp, CmpOp::Eq | CmpOp::Ne) && !ordered {
            let t = self.enums[ia.unwrap() as usize].name.clone();
            return self.reject(
                &format!("ExprBinary({}) on enum", op),
                format!("`{}` on two `{}` values, which Zig does not order", op, t),
            );
        }
        let (x, y) = (ea.unwrap().clone(), eb.unwrap().clone());
        if let (ExprKind::Const(p), ExprKind::Const(q)) = (&x.kind, &y.kind) {
            return Ok(Some(Val::E(Expr { ty: Ty::Bool, kind: ExprKind::Const(cmp.holds(*p, *q) as i128) })));
        }
        Ok(Some(Val::E(Expr { ty: Ty::Bool, kind: ExprKind::Cmp { op: cmp, lhs: Box::new(x), rhs: Box::new(y) } })))
    }

    /// `E.v` or `E::v` of a declared enum (what t27c's Zig backend orders by
    /// tag).
    fn names_variant(&self, n: &Node) -> bool {
        match n.kind {
            NodeKind::ExprFieldAccess => n.children.first().is_some_and(|b| {
                b.kind == NodeKind::ExprIdentifier && self.enum_nodes.contains_key(&b.name)
            }),
            NodeKind::ExprIdentifier => {
                n.name.split_once("::").is_some_and(|(e, _)| self.enum_nodes.contains_key(e))
            }
            _ => false,
        }
    }

    /// Lay out struct `id` (C rules) if that has not been done.
    fn layout(&mut self, id: u32) -> R<()> {
        let r = self.layout_once(id);
        self.layout_err |= r.is_err();
        r
    }

    fn layout_once(&mut self, id: u32) -> R<()> {
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
            LTy::Enum(_, ty) => Ok((ty.bytes(), ty.bytes())),
            LTy::Str | LTy::Slice(..) => Ok((16, 8)),
            LTy::Opt(inner) => {
                let (s, a) = self.size_align(inner)?;
                let a = a.max(1);
                Ok(((s + 1).div_ceil(a) * a, a))
            }
            LTy::Struct(id) => {
                self.layout(*id)?;
                let sd = &self.structs[*id as usize];
                Ok((sd.size.unwrap(), sd.align))
            }
            LTy::Arr(inner, n) => {
                let (es, ea) = self.size_align(inner)?;
                match es.checked_mul(*n) {
                    Some(size) if size < 1 << 30 => Ok((size, ea)),
                    _ => {
                        let what = self.type_name(t);
                        self.reject("type [N]T", format!("`{}` is too large", what))
                    }
                }
            }
        }
    }

    fn type_name(&self, t: &LTy) -> String {
        match t {
            LTy::S(ty) => ty.name().to_string(),
            LTy::Ptr(inner, m) => format!("*{}{}", if *m { "" } else { "const " }, self.type_name(inner)),
            LTy::Struct(id) => self.structs[*id as usize].name.clone(),
            LTy::Enum(id, _) => self.enums[*id as usize].name.clone(),
            LTy::Str => "str".to_string(),
            LTy::Arr(inner, n) => format!("[{}]{}", n, self.type_name(inner)),
            LTy::Slice(inner, m) => format!("[]{}{}", if *m { "" } else { "const " }, self.type_name(inner)),
            LTy::Opt(inner) => format!("?{}", self.type_name(inner)),
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
        if n.kind == NodeKind::ExprIf || n.kind == NodeKind::ExprSwitch {
            self.see(n);
            let r = if n.kind == NodeKind::ExprIf { self.if_expr(n, Some(want)) } else { self.switch_expr(n, Some(want)) };
            return match r {
                Err(()) if self.recover => Ok(Val::Poison),
                r => r,
            };
        }
        if let LTy::Opt(inner) = want {
            if self.is_null(n) {
                self.see(n);
                return self.opt_temp(want, None);
            }
            // A literal or builtin takes its type from the payload's.
            let typed = matches!(
                n.kind,
                NodeKind::ExprStructLit | NodeKind::ExprArrayLiteral | NodeKind::ExprEnumValue | NodeKind::ExprUnary
            ) || is_repeat_op(n)
                || (n.kind == NodeKind::ExprCall && n.name.starts_with('@'));
            let inner = (**inner).clone();
            let v = if typed { self.expr_as(n, &inner)? } else { self.expr(n)? };
            return self.coerce_to(v, want);
        }
        if let Some(v) = self.enum_literal(n, want)? {
            return Ok(v);
        }
        if let (NodeKind::ExprCall, "@floatFromInt" | "@intFromFloat", LTy::S(ty)) = (&n.kind, n.name.as_str(), want) {
            return self.convert(n, *ty);
        }
        if let (NodeKind::ExprCall, "@floatCast", LTy::S(ty)) = (&n.kind, n.name.as_str(), want) {
            return self.float_cast_call(n, *ty);
        }
        if let (NodeKind::ExprCall, "@intFromEnum", LTy::S(ty)) = (&n.kind, n.name.as_str(), want) {
            let v = self.int_from_enum(n, TagUse::Want(*ty))?;
            return self.coerce_to(v, want);
        }
        if n.kind == NodeKind::ExprStructLit && matches!(want, LTy::Struct(_)) {
            self.see(n);
            self.lit_type(n, want)?;
            return self.struct_temp(n, want.clone());
        }
        if (n.kind == NodeKind::ExprArrayLiteral || is_repeat_op(n) || n.kind == NodeKind::ExprTuple)
            && matches!(want, LTy::Arr(..))
            || n.kind == NodeKind::ExprTuple && matches!(want, LTy::Struct(_))
        {
            self.see(n);
            return self.struct_temp(n, want.clone());
        }
        // `&[_]T{ ... }` where a `[]const T` is wanted: the literal in a
        // temporary, and a slice of all of it.
        if n.kind == NodeKind::ExprUnary
            && n.extra_op == "&"
            && n.children.len() == 1
            && n.children[0].kind == NodeKind::ExprArrayLiteral
        {
            let elem = match want {
                LTy::Str => Some(LTy::S(Ty::U8)),
                LTy::Slice(t, false) => Some((**t).clone()),
                _ => None,
            };
            if let Some(elem) = elem {
                self.see(n);
                self.see(&n.children[0]);
                let len = n.children[0].children.len() as u32;
                let Val::M(arr) = self.struct_temp(&n.children[0], LTy::Arr(Box::new(elem), len))? else {
                    return Err(());
                };
                return self.slice_of(addr_of(&arr), len, want.clone());
            }
        }
        // `&[_]T{ ... } ** n` where a slice is wanted: t27c prints
        // `&.{ ... } ** n`, which Zig reads as `(&.{ ... }) ** n`, a pointer
        // to a tuple, and refuses once it analyzes it ("expected indexable").
        // In a fn nothing analyzed reaches it is only parsed, so it lowers to
        // the stub trap no test reaches; anywhere else it stays refused.
        if self.unanalyzed_fn
            && n.kind == NodeKind::ExprUnary
            && n.extra_op == "&"
            && n.children.len() == 1
            && is_repeat_op(&n.children[0])
            && matches!(want, LTy::Str | LTy::Slice(..))
        {
            self.see(n);
            let k = self.new_slot(want)?;
            let site = self.site(TrapKind::Stub, "`&` of a repeated array literal in a fn the reference never analyzes".into(), Ty::Bool);
            let trap = Stmt::Assert { cond: Expr { ty: Ty::Bool, kind: ExprKind::Const(0) }, site };
            let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts: vec![trap], value: Box::new(slot_expr(k)) } };
            return Ok(Val::M(Place { addr, off: 0, ty: want.clone(), mutable: false, temp: Some(k) }));
        }
        let v = self.expr(n)?;
        self.coerce_to(v, want)
    }

    /// A call argument. Beyond `expr_as`: an array literal where the callee
    /// declares a slice, which t27c's Zig backend writes as
    /// `@constCast(&[_]T{ ... })` -- the literal in a temporary that outlives
    /// the call, and a slice of all of it. The reference does that only when
    /// the element type is not itself an array, a slice or a tuple
    /// (`slice_element_type`); otherwise the literal stays `.{ ... }`, which
    /// Zig refuses.
    fn arg_as(&mut self, n: &Node, want: &LTy) -> R<Val> {
        // `undefined` for a `void` parameter is its one value: nothing to
        // write, nothing to read.
        if is_undefined(n) && self.is_void(want) {
            return self.struct_temp(n, want.clone());
        }
        // `&x` of a slice local is the same slice (`arraylit`).
        let local = arraylit::addr_of_name(n).unwrap_or(n);
        if local.kind == NodeKind::ExprIdentifier && self.slice_locals.contains_key(&local.name) {
            let elem = match want {
                LTy::Slice(elem, _) => Some((**elem).clone()),
                LTy::Str => Some(LTy::S(Ty::U8)),
                _ => None,
            };
            if let (Some(elem), Some(Binding::Mem(p))) = (elem, self.lookup(&local.name)) {
                if let LTy::Arr(e, len) = &p.ty {
                    if **e == elem {
                        self.see(n);
                        return self.slice_of(addr_of(&p), *len, want.clone());
                    }
                }
            }
        }
        if n.kind != NodeKind::ExprArrayLiteral {
            return self.expr_as(n, want);
        }
        // `[]const u8` is a slice of `u8` to the reference too.
        let elem = match want {
            LTy::Slice(elem, _) => elem.clone(),
            LTy::Str => Box::new(LTy::S(Ty::U8)),
            _ => return self.expr_as(n, want),
        };
        self.see(n);
        if !(*elem == LTy::Str || !has_brackets(&elem)) {
            return self.reject(
                "ExprArrayLiteral(to slice)",
                "an array literal passed where a slice of arrays or slices is declared".into(),
            );
        }
        if n.children.is_empty() && n.extra_type.trim().is_empty() && n.extra_size.contains(';') {
            // t27c pastes `v;n` between the braces of `[_]T{ ... }`.
            return self.reject(
                "ExprArrayLiteral(repeat to slice)",
                format!("`[{}]` passed where a slice is declared", n.extra_size.trim()),
            );
        }
        if let Some(lit) = self.text_lit(n)? {
            return self.arg_as(&lit, want);
        }
        let len = n.children.len() as u32;
        let Val::M(arr) = self.struct_temp(n, LTy::Arr(elem.clone(), len))? else {
            return Err(());
        };
        self.slice_of(addr_of(&arr), len, want.clone())
    }

    /// `if (c) a else b` used as a value; t27c's Zig backend emits it as
    /// written. A condition known at compile time picks its arm and the other
    /// is not lowered at all (Zig does not analyze it). Otherwise both arms
    /// take the result type `want` -- or, with none, Zig's peer type -- and
    /// only the taken one runs (`ExprKind::Select`); an aggregate (a string,
    /// a slice, a struct, an array) selects the address of its arm.
    fn if_expr(&mut self, n: &Node, want: Option<&LTy>) -> R<Val> {
        if n.children.len() != 3 {
            return self.reject("ExprIf(no else)", "`if` used as a value without `else`".into());
        }
        if self.misprinted_if.contains(&(n as *const Node as usize)) {
            return self.reject(
                "ExprIf(left operand)",
                "the reference prints this `if` without its parentheses, so its last arm takes in what follows".into(),
            );
        }
        let c = self.expr(&n.children[0])?;
        if c.is_poison() {
            // Recovery mode: still name what the arms use.
            for a in &n.children[1..] {
                let _ = self.if_arm(a, want)?;
            }
            return Err(());
        }
        let cond = self.cond_val(c)?;
        if let ExprKind::Const(k) = cond.kind {
            return self.if_arm(&n.children[if k != 0 { 1 } else { 2 }], want);
        }
        let a = self.if_arm(&n.children[1], want)?;
        let b = self.if_arm(&n.children[2], want)?;
        if a.is_poison() || b.is_poison() {
            return Err(());
        }
        self.select_vals(cond, a, b, want, "ExprIf", "both arms are untyped literals")
    }

    /// `cond ? a : b` over two arms already lowered (to `want`, when there is
    /// one): only the taken arm runs. With no result type, Zig's peer type of
    /// the two; an aggregate selects the address of its arm. `what` names the
    /// construct in a refusal.
    fn select_vals(&mut self, cond: Expr, a: Val, b: Val, want: Option<&LTy>, what: &str, untyped: &str) -> R<Val> {
        let select = |cond: Expr, x: Expr, y: Expr| Expr {
            ty: x.ty,
            kind: ExprKind::Select { cond: Box::new(cond), then: Box::new(x), els: Box::new(y) },
        };
        let in_memory = |v: &Val| matches!(v, Val::S(..)) || matches!(v, Val::M(p) if p.ty == LTy::Str);
        let (a, b) = match want {
            Some(_) => (a, b),
            None if matches!(a, Val::Ct(_) | Val::Cf(..)) && matches!(b, Val::Ct(_) | Val::Cf(..)) => {
                return self.reject(
                    &format!("{}(comptime arms)", what),
                    format!("{} and the condition is runtime: Zig needs a result type", untyped),
                )
            }
            // Peer type of strings (and of a string and a literal) is `[]const u8`.
            None if in_memory(&a) || in_memory(&b) => (self.coerce_to(a, &LTy::Str)?, self.coerce_to(b, &LTy::Str)?),
            None => match (a, b) {
                (Val::M(p), Val::M(q)) if p.ty == q.ty => (Val::M(p), Val::M(q)),
                (Val::P(x, t), Val::P(y, u)) if t == u => (Val::P(x, t), Val::P(y, u)),
                (a, b) => {
                    let (x, y) = self.peer(a, b, if what == "ExprIf" { "if" } else { "switch" })?;
                    return Ok(Val::E(select(cond, x, y)));
                }
            },
        };
        match (a, b) {
            (Val::M(p), Val::M(q)) if p.ty == q.ty => {
                let addr = select(cond, addr_of(&p), addr_of(&q));
                Ok(Val::M(Place { addr, off: 0, ty: p.ty, mutable: false, temp: None }))
            }
            (Val::P(x, t), Val::P(y, u)) if t == u && x.ty == y.ty => Ok(Val::P(select(cond, x, y), t)),
            (Val::E(x), Val::E(y)) if x.ty == y.ty => Ok(Val::E(select(cond, x, y))),
            (a, b) => {
                let (a, b) = (self.val_desc(&a), self.val_desc(&b));
                self.reject(what, format!("arms of different shapes: {} and {}", a, b))
            }
        }
    }

    fn if_arm(&mut self, n: &Node, want: Option<&LTy>) -> R<Val> {
        match want {
            Some(t) => self.expr_as(n, t),
            None => self.expr(n),
        }
    }

    /// `switch (x) { .a => v, 1 => w, else => u }` used as a value. The t27c
    /// parser keeps one item per prong (`.name` or a bare name, an integer,
    /// `-N`, a char literal, `else`) and its Zig backend prints each as
    /// `.name`, the integer, or `else`. Zig's rules, refused where they would
    /// not compile: an enum operand takes tag names, every tag or an `else`
    /// but not both; an integer operand takes integers of its type and needs
    /// an `else` unless every value is listed; no item twice. A
    /// compile-time operand lowers only its prong (Zig does not analyze the
    /// others). Otherwise every prong takes the result type `want` -- or,
    /// with none, the peer type -- and only the taken one runs: a chain of
    /// `ExprKind::Select` on `x == item`, the `else` prong (or, with none,
    /// the last one) last. An operand with effects is evaluated once, into a
    /// frame slot.
    fn switch_expr(&mut self, n: &Node, want: Option<&LTy>) -> R<Val> {
        let Some((operand, prongs)) = n.children.split_first() else {
            return self.reject("ExprSwitch", "no operand".into());
        };
        if prongs.is_empty() {
            return self.reject("ExprSwitch(no prongs)", "`switch` with no prongs".into());
        }
        for p in prongs {
            if p.kind != NodeKind::ConstDecl || p.children.len() != 1 {
                return self.reject("ExprSwitch", format!("prong of unexpected shape ({})", kind_name(p)));
            }
        }
        let x = self.expr(operand)?;
        if x.is_poison() {
            // Recovery mode: still name what the prongs use.
            for p in prongs {
                let _ = self.if_arm(&p.children[0], want)?;
            }
            return Err(());
        }
        // The operand as a register expression of type `ty`, and the item
        // values its prongs name (None: `else`).
        let (e, ty, enum_id) = match x {
            Val::P(e, LTy::Enum(id, _)) => {
                let t = e.ty;
                (e, t, Some(id))
            }
            Val::E(e) if e.ty.is_int() => {
                let t = e.ty;
                (e, t, None)
            }
            Val::Ct(_) => {
                return self.reject(
                    "ExprSwitch(on comptime_int)",
                    "switch on an untyped integer literal".into(),
                )
            }
            v => {
                let d = match &v {
                    Val::E(e) => e.ty.name().to_string(),
                    v => self.val_desc(v),
                };
                return self.reject(&format!("ExprSwitch(on {})", d), format!("switch on {}", d));
            }
        };
        let mut items: Vec<Option<i128>> = Vec::new();
        for p in prongs {
            let name = p.name.trim();
            if name.is_empty() || name == "else" {
                if items.contains(&None) {
                    return self.reject("ExprSwitch(two else prongs)", "more than one `else` prong".into());
                }
                items.push(None);
                continue;
            }
            if name.starts_with('\'') {
                return self.reject(
                    "ExprSwitch(char prong)",
                    format!("t27c's Zig backend prints the prong `{}` as `.{}`, which Zig does not parse (#6329)", name, name),
                );
            }
            let numeric = name.starts_with(|c: char| c.is_ascii_digit()) || (name.starts_with('-') && name.len() > 1);
            let v = match (enum_id, numeric) {
                (Some(id), false) => {
                    let def = &self.enums[id as usize];
                    match def.value(name) {
                        Some(v) => v,
                        None => {
                            let en = def.name.clone();
                            return self.reject("ExprSwitch(prong)", format!("`{}` has no variant `{}`", en, name));
                        }
                    }
                }
                (Some(id), true) => {
                    let en = self.enums[id as usize].name.clone();
                    return self.reject("ExprSwitch(prong)", format!("integer prong `{}` on enum `{}`", name, en));
                }
                (None, false) => {
                    return self.reject(
                        "ExprSwitch(prong)",
                        format!("t27c prints the prong `{}` as `.{}`, an enum literal, on a {} operand", name, name, ty.name()),
                    )
                }
                (None, true) => {
                    let v = match name.strip_prefix('-') {
                        Some(r) => parse_int(r).map(|v| -v),
                        None => parse_int(name),
                    };
                    match v {
                        Some(v) if ty.fits(v) => v,
                        _ => {
                            return self.reject(
                                "ExprSwitch(prong)",
                                format!("prong `{}` is not a {} value", name, ty.name()),
                            )
                        }
                    }
                }
            };
            if items.contains(&Some(v)) {
                return self.reject("ExprSwitch(duplicate prong)", format!("the prong `{}` appears twice", name));
            }
            items.push(Some(v));
        }
        let listed = items.iter().filter(|i| i.is_some()).count() as i128;
        let all = match enum_id {
            Some(id) => self.enums[id as usize].variants.len() as i128,
            None => ty.max() - ty.min() + 1,
        };
        let has_else = items.contains(&None);
        if listed < all && !has_else {
            return self.reject(
                "ExprSwitch(not exhaustive)",
                format!("{} of {} values listed and no `else` prong", listed, all),
            );
        }
        if listed == all && has_else {
            return self.reject(
                "ExprSwitch(unreachable else)",
                "an `else` prong after every value is listed, which Zig refuses".into(),
            );
        }
        // The prong that runs when no listed item matched: `else`, or the
        // last prong when every value is listed.
        let fallback = items.iter().position(|i| i.is_none()).unwrap_or(items.len() - 1);
        if let ExprKind::Const(k) = e.kind {
            let i = items.iter().position(|i| *i == Some(k)).unwrap_or(fallback);
            return self.if_arm(&prongs[i].children[0], want);
        }
        let mut vals = Vec::with_capacity(prongs.len());
        for p in prongs {
            vals.push(self.if_arm(&p.children[0], want)?);
        }
        if vals.iter().any(|v| v.is_poison()) {
            return Err(());
        }
        let (stmts, key) = if matches!(e.kind, ExprKind::Var(_)) {
            (Vec::new(), e)
        } else {
            let k = self.new_slot(&LTy::S(ty))?;
            let store = Stmt::Store { addr: slot_expr(k), off: 0, value: e };
            (vec![store], Expr { ty, kind: ExprKind::Load { addr: Box::new(slot_expr(k)), off: 0 } })
        };
        let mut vals: Vec<Option<Val>> = vals.into_iter().map(Some).collect();
        let mut acc = vals[fallback].take().unwrap();
        for i in (0..prongs.len()).rev() {
            if i == fallback {
                continue;
            }
            let Some(item) = items[i] else { unreachable!() };
            let cond = Expr {
                ty: Ty::Bool,
                kind: ExprKind::Cmp {
                    op: CmpOp::Eq,
                    lhs: Box::new(key.clone()),
                    rhs: Box::new(Expr { ty, kind: ExprKind::Const(item) }),
                },
            };
            let v = vals[i].take().unwrap();
            acc = self.select_vals(cond, v, acc, want, "ExprSwitch", "every prong is an untyped literal")?;
        }
        if stmts.is_empty() {
            return Ok(acc);
        }
        let seq = |stmts: Vec<Stmt>, value: Expr| Expr { ty: value.ty, kind: ExprKind::Seq { stmts, value: Box::new(value) } };
        match acc {
            Val::E(x) => Ok(Val::E(seq(stmts, x))),
            Val::P(x, t) => Ok(Val::P(seq(stmts, x), t)),
            Val::M(mut p) => {
                p.addr = seq(stmts, p.addr);
                p.temp = None;
                Ok(Val::M(p))
            }
            v => {
                let d = self.val_desc(&v);
                self.reject("ExprSwitch(one prong)", format!("a single prong giving {} after an operand with effects", d))
            }
        }
    }

    /// `@floatFromInt(x)` / `@intFromFloat(x)` with result type `ty`, Zig's
    /// spelling: the type comes from the context (`@as`, a typed binding, a
    /// parameter, a return), never from the call.
    fn convert(&mut self, n: &Node, ty: Ty) -> R<Val> {
        self.see(n);
        let name = n.name.clone();
        let what = format!("ExprCall({})", name);
        if n.children.len() != 1 {
            return self.reject(&what, format!("{} arguments", n.children.len()));
        }
        let to_float = name == "@floatFromInt";
        if to_float && !ty.is_float() {
            return self.reject(&what, format!("result type {}", ty.name()));
        }
        if !to_float && !ty.is_int() {
            return self.reject(&what, format!("result type {}", ty.name()));
        }
        let v = self.expr(&n.children[0])?;
        if v.is_poison() {
            return Err(());
        }
        if to_float {
            return self.int_to_float(v, ty, &what);
        }
        let e = match v {
            Val::Cf(q) => match q.exact_f64() {
                Some(f) => Expr { ty: Ty::F64, kind: ExprKind::Const(f64_bits(f)) },
                None => return self.reject(&what, "of a comptime_float that is not exactly an f64".into()),
            },
            Val::E(e) if e.ty.is_float() => e,
            v => {
                let d = match &v { Val::E(e) => e.ty.name().to_string(), _ => self.val_desc(&v) };
                return self.reject(&what, format!("operand is {}, not a float", d));
            }
        };
        self.from_float(e, ty, &what)
    }

    /// The typed float `e` converted to the integer type `ty` (Zig's
    /// `@intFromFloat`): toward zero, trapping when out of range. `what`
    /// names the construct in a rejection.
    fn from_float(&mut self, e: Expr, ty: Ty, what: &str) -> R<Val> {
        if let ExprKind::Const(c) = e.kind {
            let x = float_of(c, e.ty);
            return match float_to_int(x, ty) {
                Some(r) => Ok(Val::E(Expr { ty, kind: ExprKind::Const(r) })),
                None => self.reject(what, format!("{} does not fit {} at compile time", x, ty.name())),
            };
        }
        let site = self.site(TrapKind::FloatToInt, format!("@intFromFloat to {}", ty.name()), ty);
        Ok(Val::E(Expr { ty, kind: ExprKind::FloatToInt { arg: Box::new(e), site } }))
    }

    /// `@floatFromInt(v)` with float result type `to`: the integer operand
    /// `v` converted, rounding to nearest. `what` names the construct in a
    /// rejection.
    fn int_to_float(&mut self, v: Val, to: Ty, what: &str) -> R<Val> {
        let e = match v {
            Val::Poison => return Err(()),
            Val::Ct(c) => return Ok(Val::E(self.coerce(Val::Ct(c), to)?)),
            Val::E(e) if e.ty.is_int() => e,
            v => {
                let d = self.val_desc(&v);
                let d = match &v { Val::E(e) => e.ty.name().to_string(), _ => d };
                return self.reject(what, format!("operand is {}, not an integer", d));
            }
        };
        if let ExprKind::Const(c) = e.kind {
            // i128 to f64 / f32 rounds to nearest, ties to even, as SCVTF
            // does (straight to f32: one rounding).
            let bits = if to == Ty::F32 { f32_bits(c as f32) } else { f64_bits(c as f64) };
            return Ok(Val::E(Expr { ty: to, kind: ExprKind::Const(bits) }));
        }
        Ok(Val::E(Expr { ty: to, kind: ExprKind::IntToFloat(Box::new(e)) }))
    }

    /// A slice of type `t` (a `Slice` or `Str`) of all `len` elements at
    /// `ptr`, as a fresh temporary.
    fn slice_of(&mut self, ptr: Expr, len: u32, t: LTy) -> R<Val> {
        let k = self.new_slot(&t)?;
        let n = Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) };
        let stmts = vec![
            Stmt::Store { addr: slot_expr(k), off: 0, value: ptr },
            Stmt::Store { addr: slot_expr(k), off: 8, value: n },
        ];
        let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts, value: Box::new(slot_expr(k)) } };
        Ok(Val::M(Place { addr, off: 0, ty: t, mutable: false, temp: Some(k) }))
    }

    /// The header of slice place `p`, at a pure address: `p` itself, or a
    /// copy of it made by `pin` (which runs first, once).
    fn pin_header(&mut self, p: &Place, pin: &mut Vec<Stmt>) -> R<(Expr, u32)> {
        if pure_addr(&p.addr) {
            return Ok((p.addr.clone(), p.off));
        }
        let k = self.new_slot(&p.ty)?;
        pin.push(Stmt::Copy { dst: slot_expr(k), src: addr_of(p), size: 16 });
        Ok((slot_expr(k), 0))
    }

    /// The type a named struct literal builds. Only a struct takes `T{ .f = .. }`:
    /// an alias of a scalar (`const Duo = u8;`) is refused here, as Zig does,
    /// rather than handed back to `init`, which would re-enter for ever.
    fn struct_lit_ty(&mut self, n: &Node) -> R<LTy> {
        let t = self.lty(&n.name)?;
        if !matches!(t, LTy::Struct(_)) {
            let tn = self.type_name(&t);
            return self.reject("ExprStructLit", format!("`{}` is {}, not a struct", n.name, tn));
        }
        Ok(t)
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

    /// `null` as a value: the identifier, unless a local shadows it.
    fn is_null(&self, n: &Node) -> bool {
        n.kind == NodeKind::ExprIdentifier && n.name == "null" && self.lookup("null").is_none()
    }

    /// A fresh `?T` temporary (`want` is `?T`): `null` when `payload` is
    /// None, else holding `payload`, a value already of type `T`.
    fn opt_temp(&mut self, want: &LTy, payload: Option<Val>) -> R<Val> {
        let LTy::Opt(inner) = want else { unreachable!() };
        let inner = (**inner).clone();
        let (s, _) = self.size_align(&inner)?;
        let k = self.new_slot(want)?;
        let mut stmts = Vec::new();
        let has = payload.is_some();
        if let Some(v) = payload {
            if reg_ty(&inner).is_some() {
                let value = self.reg(v)?;
                stmts.push(Stmt::Store { addr: slot_expr(k), off: 0, value });
            } else {
                match v {
                    Val::M(src) if s > 0 => stmts.push(Stmt::Copy { dst: slot_expr(k), src: addr_of(&src), size: s }),
                    Val::M(src) => {
                        if !pure_addr(&src.addr) {
                            stmts.push(Stmt::Eval(src.addr));
                        }
                    }
                    Val::Poison => return Err(()),
                    v => {
                        let d = self.val_desc(&v);
                        return self.reject("type mismatch", format!("internal: optional payload {} not in memory", d));
                    }
                }
            }
        }
        let flag = Expr { ty: Ty::Bool, kind: ExprKind::Const(has as i128) };
        stmts.push(Stmt::Store { addr: slot_expr(k), off: s, value: flag });
        let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts, value: Box::new(slot_expr(k)) } };
        Ok(Val::M(Place { addr, off: 0, ty: want.clone(), mutable: false, temp: Some(k) }))
    }

    /// `x == null` / `x != null` (`n`, one operand the literal `null`):
    /// the has-value flag of the optional `x`.
    fn null_compare(&mut self, op: &str, n: &Node) -> R<Val> {
        let (x, y) = (&n.children[0], &n.children[1]);
        if self.is_null(x) && self.is_null(y) {
            return self.reject("ExprBinary(null)", format!("`null {} null`", op));
        }
        let (other, lit) = if self.is_null(x) { (y, x) } else { (x, y) };
        self.see(lit);
        if let Some(v) = self.index_of_null(op, other)? {
            return Ok(v);
        }
        let p = match self.expr(other)? {
            Val::M(p) if matches!(p.ty, LTy::Opt(_)) => p,
            Val::Poison => return Err(()),
            v => {
                let d = match &v {
                    Val::M(p) => self.type_name(&p.ty),
                    _ => self.val_desc(&v),
                };
                return self.reject("ExprBinary(null)", format!("`{}` compares {} with null", op, d));
            }
        };
        let LTy::Opt(inner) = &p.ty else { unreachable!() };
        let inner = (**inner).clone();
        let (s, _) = self.size_align(&inner)?;
        let flag = Expr { ty: Ty::Bool, kind: ExprKind::Load { addr: Box::new(p.addr), off: p.off + s } };
        let e = if op == "==" { Expr { ty: Ty::Bool, kind: ExprKind::Not(Box::new(flag)) } } else { flag };
        Ok(Val::E(e))
    }

    /// `x == v` / `x != v` with `x` a `?T` (`T` a scalar) and `v` a `T`,
    /// either way round: Zig's comparison of an optional with a payload,
    /// equal only when `x` holds a value equal to `v`. None when neither
    /// operand is an optional.
    fn opt_compare(&mut self, op: &str, a: &Val, b: &Val) -> R<Option<Val>> {
        let is_opt = |v: &Val| matches!(v, Val::M(p) if matches!(p.ty, LTy::Opt(_)));
        if !is_opt(a) && !is_opt(b) {
            return Ok(None);
        }
        let what = "ExprBinary(?T)";
        if is_opt(a) && is_opt(b) {
            return self.reject(what, format!("`{}` of two optionals", op));
        }
        if op != "==" && op != "!=" {
            return self.reject(what, format!("`{}` on an optional", op));
        }
        let left = is_opt(a);
        let (p, other) = if left { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        let Val::M(p) = p else { unreachable!() };
        // The payload is read only when there is one, so the other operand
        // is evaluated inside the check: it must have no effect.
        match &other {
            Val::Ct(_) | Val::Cf(..) => {}
            Val::E(e) | Val::P(e, _) if matches!(e.kind, ExprKind::Const(_) | ExprKind::Var(_)) => {}
            Val::Poison => return Err(()),
            _ => return self.reject(what, "an optional compared with a value that has effects".into()),
        }
        let LTy::Opt(inner) = p.ty.clone() else { unreachable!() };
        if !matches!(*inner, LTy::S(_)) {
            let t = self.type_name(&p.ty);
            return self.reject(what, format!("`{}` on {}", op, t));
        }
        let (s, _) = self.size_align(&inner)?;
        let mut stmts = Vec::new();
        let (addr, off) = if pure_addr(&p.addr) {
            (p.addr.clone(), p.off)
        } else {
            let k = self.new_slot(&LTy::Ptr(Box::new(LTy::S(Ty::U8)), false))?;
            stmts.push(Stmt::Store { addr: slot_expr(k), off: 0, value: addr_of(&p) });
            (Expr { ty: Ty::Ptr, kind: ExprKind::Load { addr: Box::new(slot_expr(k)), off: 0 } }, 0)
        };
        let flag = Expr { ty: Ty::Bool, kind: ExprKind::Load { addr: Box::new(addr.clone()), off: off + s } };
        let payload = self.place_value(Place { addr, off, ty: *inner, mutable: false, temp: None })?;
        let (l, r) = if left { (payload, other) } else { (other, payload) };
        let cmp = self.binary(op, l, r)?;
        let cmp = self.coerce(cmp, Ty::Bool)?;
        let els = Expr { ty: Ty::Bool, kind: ExprKind::Const((op == "!=") as i128) };
        let e = Expr { ty: Ty::Bool, kind: ExprKind::Select { cond: Box::new(flag), then: Box::new(cmp), els: Box::new(els) } };
        if stmts.is_empty() {
            return Ok(Some(Val::E(e)));
        }
        Ok(Some(Val::E(Expr { ty: Ty::Bool, kind: ExprKind::Seq { stmts, value: Box::new(e) } })))
    }

    /// `x.?` (`base` is `x`): the payload of optional `x`, after a check
    /// that traps like Zig's "attempt to use null value".
    fn unwrap(&mut self, base: &Node) -> R<Place> {
        let p = match self.expr(base)? {
            Val::M(p) if matches!(p.ty, LTy::Opt(_)) => p,
            Val::Poison => return Err(()),
            v => {
                let d = self.val_desc(&v);
                return self.reject("ExprFieldAccess(.?)", format!("`.?` of {}, not an optional", d));
            }
        };
        let LTy::Opt(inner) = p.ty.clone() else { unreachable!() };
        let (s, _) = self.size_align(&inner)?;
        let mut stmts = Vec::new();
        // The address is read twice (flag, payload): pin it when it has
        // effects.
        let (addr, off) = if pure_addr(&p.addr) {
            (p.addr.clone(), p.off)
        } else {
            let k = self.new_slot(&LTy::Ptr(Box::new(LTy::S(Ty::U8)), false))?;
            stmts.push(Stmt::Store { addr: slot_expr(k), off: 0, value: addr_of(&p) });
            (Expr { ty: Ty::Ptr, kind: ExprKind::Load { addr: Box::new(slot_expr(k)), off: 0 } }, 0)
        };
        let flag = Expr { ty: Ty::Bool, kind: ExprKind::Load { addr: Box::new(addr.clone()), off: off + s } };
        let what = format!("`.?` of {}", self.type_name(&p.ty));
        let site = self.site(TrapKind::Null, what, Ty::U64);
        let check = Expr {
            ty: Ty::U64,
            kind: ExprKind::Bounds {
                idx: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Const(0) }),
                len: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Widen(Box::new(flag)) }),
                site,
            },
        };
        stmts.push(Stmt::Eval(check));
        let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts, value: Box::new(addr) } };
        if let LTy::Struct(id) = *inner {
            self.layout(id)?;
        }
        Ok(Place { addr, off, ty: *inner, mutable: p.mutable && p.temp.is_none(), temp: None })
    }

    fn coerce_to(&mut self, v: Val, want: &LTy) -> R<Val> {
        if v.is_poison() {
            // Recovery mode: the caller decides what an unknown value costs.
            return Ok(v);
        }
        match want {
            LTy::S(ty) => Ok(Val::E(self.coerce(v, *ty)?)),
            LTy::Enum(id, _) => match v {
                Val::P(e, LTy::Enum(got, t)) if got == *id => Ok(Val::P(e, LTy::Enum(got, t))),
                v => {
                    let (a, b) = (self.type_name(want), self.val_desc(&v));
                    self.reject("type mismatch", format!("expected {}, found {}", a, b))
                }
            },
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
            // A literal is written into a temporary; the place it is copied
            // to, if any, is the caller's business.
            LTy::Str => match v {
                Val::S(k, len) => {
                    let slot = self.new_slot(want)?;
                    let dst = Place { addr: slot_expr(slot), off: 0, ty: LTy::Str, mutable: true, temp: None };
                    let mut stmts = Vec::new();
                    self.store_str(&dst, k, len, &mut stmts);
                    let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts, value: Box::new(slot_expr(slot)) } };
                    Ok(Val::M(Place { addr, off: 0, ty: LTy::Str, mutable: false, temp: Some(slot) }))
                }
                Val::M(p) if p.ty == LTy::Str => Ok(Val::M(p)),
                // `[]u8` coerces to `[]const u8`.
                Val::M(mut p) if p.ty == LTy::Slice(Box::new(LTy::S(Ty::U8)), true) => {
                    p.ty = LTy::Str;
                    Ok(Val::M(p))
                }
                Val::M(p) => {
                    let b = self.type_name(&p.ty);
                    self.reject("type mismatch", format!("expected str, found {}", b))
                }
                // `*[N]u8` (or `*const [N]u8`) coerces to `[]const u8`.
                Val::P(e, LTy::Ptr(inner, _)) if matches!(&*inner, LTy::Arr(t, _) if **t == LTy::S(Ty::U8)) => {
                    let LTy::Arr(_, n) = *inner else { unreachable!() };
                    self.slice_of(e, n, LTy::Str)
                }
                Val::P(_, t) => {
                    let b = self.type_name(&t);
                    self.reject("type mismatch", format!("expected str, found {}", b))
                }
                _ => self.reject("type mismatch", "expected str, found a scalar".into()),
            },
            LTy::Slice(inner, m) => match v {
                // `[]T` coerces to `[]const T`.
                Val::M(mut p) if matches!(&p.ty, LTy::Slice(t, pm) if t == inner && (*pm || !*m)) => {
                    p.ty = want.clone();
                    Ok(Val::M(p))
                }
                // `*[N]T` coerces to `[]T`, `*const [N]T` only to `[]const T`.
                Val::P(e, LTy::Ptr(pt, pm))
                    if (pm || !*m) && matches!(&*pt, LTy::Arr(t, _) if t == inner) =>
                {
                    let LTy::Arr(_, n) = *pt else { unreachable!() };
                    self.slice_of(e, n, want.clone())
                }
                v => {
                    let (a, b) = (self.type_name(want), self.val_desc(&v));
                    self.reject("type mismatch", format!("expected {}, found {}", a, b))
                }
            },
            LTy::Arr(..) => match v {
                Val::M(p) if &p.ty == want => Ok(Val::M(p)),
                Val::A(t, elems) if &t == want => Ok(Val::M(self.materialize(t, elems)?)),
                v => {
                    let (a, b) = (self.type_name(want), self.val_desc(&v));
                    self.reject("type mismatch", format!("expected {}, found {}", a, b))
                }
            },
            // A `T` coerces to `?T`: the value, in a temporary, with the flag
            // set.
            LTy::Opt(inner) => match v {
                Val::M(p) if &p.ty == want => Ok(Val::M(p)),
                v => {
                    let inner = (**inner).clone();
                    match self.coerce_to(v, &inner)? {
                        Val::Poison => Ok(Val::Poison),
                        v => self.opt_temp(want, Some(v)),
                    }
                }
            },
            LTy::Struct(_) => match v {
                Val::M(p) if &p.ty == want => Ok(Val::M(p)),
                Val::A(t, elems) if &t == want => Ok(Val::M(self.materialize(t, elems)?)),
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
            Val::Cf(..) => self.reject("type mismatch", "untyped float literal".into()),
            Val::M(_) | Val::A(..) => self.reject("type mismatch", "a struct or array where a scalar is expected".into()),
            Val::S(..) => self.reject("type mismatch", "a string where a scalar is expected".into()),
        }
    }

    /// The blob of a string literal's bytes (deduplicated), as a value.
    fn string(&mut self, bytes: &[u8]) -> Val {
        let len = bytes.len() as u64;
        if let Some(&k) = self.strings.get(bytes) {
            return Val::S(k, len);
        }
        // A trailing NUL, as Zig's literals have, so no blob is empty.
        let mut blob = bytes.to_vec();
        blob.push(0);
        self.data.push(blob);
        let k = (self.data.len() - 1) as u32;
        self.strings.insert(bytes.to_vec(), k);
        Val::S(k, len)
    }

    /// `dst = literal`: the address of the bytes, then the length. `dst`'s
    /// address is pure.
    fn store_str(&mut self, dst: &Place, k: u32, len: u64, out: &mut Vec<Stmt>) {
        let ptr = Expr { ty: Ty::Ptr, kind: ExprKind::Data(k) };
        out.push(Stmt::Store { addr: dst.addr.clone(), off: dst.off, value: ptr });
        let n = Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) };
        out.push(Stmt::Store { addr: dst.addr.clone(), off: dst.off + 8, value: n });
    }

    /// A compile-time array, written into a fresh temporary at the point
    /// the value is evaluated.
    fn materialize(&mut self, t: LTy, elems: Vec<Val>) -> R<Place> {
        let k = self.new_slot(&t)?;
        let dst = Place { addr: slot_expr(k), off: 0, ty: t.clone(), mutable: true, temp: None };
        let mut stmts = Vec::new();
        self.store_const(&dst, &elems, &mut stmts)?;
        let addr = if stmts.is_empty() {
            slot_expr(k)
        } else {
            Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts, value: Box::new(slot_expr(k)) } }
        };
        Ok(Place { addr, off: 0, ty: t, mutable: false, temp: Some(k) })
    }

    /// The elements (or fields) of a compile-time aggregate into `dst`
    /// (pure address).
    fn store_const(&mut self, dst: &Place, elems: &[Val], out: &mut Vec<Stmt>) -> R<()> {
        let places: Vec<Place> = match &dst.ty {
            LTy::Arr(elem, _) => {
                let elem = (**elem).clone();
                let (esize, _) = self.size_align(&elem)?;
                (0..elems.len()).map(|i| elem_place(dst, &elem, esize, i as u32)).collect()
            }
            LTy::Struct(id) => self.fields(*id)?.iter().map(|f| field_place(dst, f)).collect(),
            _ => return self.reject("ConstDecl", "internal: constant aggregate type".into()),
        };
        for (p, v) in places.iter().zip(elems) {
            match v {
                Val::S(k, len) => self.store_str(p, *k, *len, out),
                Val::A(_, sub) => self.store_const(p, sub, out)?,
                Val::M(src) => self.copy(p, src.clone(), out)?,
                Val::E(e) | Val::P(e, _) => {
                    out.push(Stmt::Store { addr: p.addr.clone(), off: p.off, value: e.clone() })
                }
                _ => return self.reject("ConstDecl", "internal: aggregate constant element".into()),
            }
        }
        Ok(())
    }

    /// What a value is, for a type error.
    fn val_desc(&self, v: &Val) -> String {
        match v {
            Val::Ct(_) | Val::Cf(..) | Val::E(_) => "a scalar".into(),
            Val::P(_, t @ LTy::Enum(..)) => self.type_name(t),
            Val::P(..) => "a pointer".into(),
            Val::S(..) => "a string".into(),
            Val::M(p) => self.type_name(&p.ty),
            Val::A(t, _) => self.type_name(t),
            Val::Poison => "an unknown value".into(),
        }
    }

    fn is_str(&self, v: &Val) -> bool {
        match v {
            Val::S(..) => true,
            Val::M(p) => p.ty == LTy::Str,
            _ => false,
        }
    }

    /// `a == b` (or `!=`) where one side is a string: content equality.
    fn str_eq(&mut self, negate: bool, a: Val, b: Val) -> R<Val> {
        let eq = if let (Val::S(ka, la), Val::S(kb, lb)) = (&a, &b) {
            let (x, y) = (&self.data[*ka as usize][..*la as usize], &self.data[*kb as usize][..*lb as usize]);
            Expr { ty: Ty::Bool, kind: ExprKind::Const((x == y) as i128) }
        } else {
            let mut args = Vec::new();
            for v in [a, b] {
                match self.coerce_to(v, &LTy::Str)? {
                    Val::M(p) => args.push(addr_of(&p)),
                    _ => return Err(()),
                }
            }
            self.eql_used = true;
            Expr { ty: Ty::Bool, kind: ExprKind::Call { func: self.nfuncs, args } }
        };
        if !negate {
            return Ok(Val::E(eq));
        }
        Ok(Val::E(match eq.kind {
            ExprKind::Const(c) => Expr { ty: Ty::Bool, kind: ExprKind::Const(1 - c) },
            _ => Expr { ty: Ty::Bool, kind: ExprKind::Not(Box::new(eq)) },
        }))
    }

    /// A struct or array literal built in a fresh temporary slot, at the
    /// point the value is evaluated.
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
        // Only an aggregate result has a hidden result pointer: a scalar
        // field initialised from a call (`.x = f(a)`) is a plain store of the
        // call's value.
        let in_place = n.kind == NodeKind::ExprCall
            && fresh
            && is_agg(&t)
            && self.sigs.get(&n.name).is_some_and(|s| s.ret == Some(t.clone()));
        if let LTy::Opt(inner) = &t {
            if self.is_null(n) {
                let (s, _) = self.size_align(inner)?;
                let flag = Expr { ty: Ty::Bool, kind: ExprKind::Const(0) };
                out.push(Stmt::Store { addr: dst.addr, off: dst.off + s, value: flag });
                return Ok(());
            }
            if in_place {
                let (call, _, _) = self.call(n, Some(addr_of(&dst)))?;
                out.push(Stmt::Eval(call));
                return Ok(());
            }
            // Built aside first: the value may read `dst`.
            return match self.expr_as(n, &t)? {
                Val::M(src) => self.copy(&dst, src, out),
                _ => Err(()),
            };
        }
        if n.kind == NodeKind::ExprTuple {
            return self.init_tuple_lit(n, dst, fresh, out);
        }
        if matches!(t, LTy::Str | LTy::Slice(..)) && !in_place {
            let v = if t == LTy::Str && n.kind != NodeKind::ExprUnary { self.expr(n)? } else { self.expr_as(n, &t)? };
            return match v {
                Val::S(k, len) if pure_addr(&dst.addr) => {
                    self.store_str(&dst, k, len, out);
                    Ok(())
                }
                v => match self.coerce_to(v, &t)? {
                    Val::M(src) => self.copy(&dst, src, out),
                    _ => Err(()),
                },
            };
        }
        if let LTy::Arr(..) = t {
            if n.kind == NodeKind::ExprArrayLiteral || is_repeat_op(n) {
                if fresh && pure_addr(&dst.addr) {
                    return self.init_array(n, &dst, out);
                }
                // The literal may read `dst`: build it aside first.
                let k = self.new_slot(&t)?;
                let tmp = Place { addr: slot_expr(k), off: 0, ty: t, mutable: true, temp: None };
                self.init_array(n, &tmp, out)?;
                return self.copy(&dst, tmp, out);
            }
            if in_place {
                let (call, _, _) = self.call(n, Some(addr_of(&dst)))?;
                out.push(Stmt::Eval(call));
                return Ok(());
            }
            let v = self.expr(n)?;
            return match v {
                Val::A(u, elems) if u == t && pure_addr(&dst.addr) => self.store_const(&dst, &elems, out),
                v => match self.coerce_to(v, &t)? {
                    Val::M(src) => self.copy(&dst, src, out),
                    _ => Err(()),
                },
            };
        }
        let LTy::Struct(id) = t else {
            if in_place {
                // A call that returns a str, built in place.
                let (call, _, _) = self.call(n, Some(addr_of(&dst)))?;
                out.push(Stmt::Eval(call));
                return Ok(());
            }
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

    /// The elements of an array literal, into `dst` (whose address is pure).
    /// The literal's own type, if it names one (`[_]u8{ ... }`), is not
    /// checked: t27c's Zig backend writes every array literal as an
    /// anonymous `.{ ... }`, which takes the type of its destination.
    fn init_array(&mut self, n: &Node, dst: &Place, out: &mut Vec<Stmt>) -> R<()> {
        if let Some((elems, count)) = self.repeat_lit(n, &dst.ty)? {
            return self.init_repeat(&elems, count, dst, out);
        }
        if let Some(lit) = self.text_lit(n)? {
            return self.init_array(&lit, dst, out);
        }
        self.array_count(n, &dst.ty)?;
        let LTy::Arr(elem, _) = &dst.ty else { unreachable!() };
        let elem = (**elem).clone();
        let (esize, _) = self.size_align(&elem)?;
        for (i, c) in n.children.iter().enumerate() {
            let p = elem_place(dst, &elem, esize, i as u32);
            self.init(c, p, true, out)?;
        }
        Ok(())
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
            let v = &c.children[0];
            if v.kind == NodeKind::ExprArrayLiteral && matches!(fields[i].ty, LTy::Str | LTy::Slice(..)) {
                self.slice_field(n, &sname, v, sub, out)?;
                continue;
            }
            self.init(v, sub, true, out)?;
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

    /// An array literal for slice field `dst` of struct literal `n`. t27c's
    /// Zig backend writes `@constCast(&[_]T{ ... })` when the literal names
    /// its struct (otherwise `.{ ... }`, which Zig refuses for a slice).
    /// Only the empty literal is taken: its slice has length zero, so where
    /// it points is never read. A non-empty one points at a constant in the
    /// reference, which outlives any frame t27b could build it in.
    fn slice_field(&mut self, n: &Node, sname: &str, v: &Node, dst: Place, out: &mut Vec<Stmt>) -> R<()> {
        self.see(v);
        if n.name != sname {
            return self.reject(
                "ExprArrayLiteral(to slice)",
                "an array literal for a slice field of an anonymous struct literal (the reference writes `.{ ... }`)".into(),
            );
        }
        let elem = match &dst.ty {
            LTy::Slice(elem, _) => (**elem).clone(),
            _ => LTy::S(Ty::U8),
        };
        if !(elem == LTy::Str || !has_brackets(&elem)) {
            return self.reject(
                "ExprArrayLiteral(to slice)",
                "an array literal for a slice-of-arrays field (the reference writes `.{ ... }`)".into(),
            );
        }
        if !v.children.is_empty() || !v.extra_size.trim().is_empty() || !v.extra_type.trim().is_empty() {
            return self.reject(
                "ExprArrayLiteral(to slice field)",
                "a non-empty array literal for a slice field (the reference points into a constant)".into(),
            );
        }
        let Val::M(arr) = self.struct_temp(v, LTy::Arr(Box::new(elem), 0))? else {
            return Err(());
        };
        let Val::M(src) = self.slice_of(addr_of(&arr), 0, dst.ty.clone())? else {
            return Err(());
        };
        self.copy(&dst, src, out)
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
        if let (ExprKind::Data(k), LTy::S(_) | LTy::Enum(..)) = (&p.addr.kind, &p.ty) {
            let blob = &self.data[*k as usize];
            let mut raw = 0u64;
            for i in (0..ty.bytes() as usize).rev() {
                raw = (raw << 8) | blob[p.off as usize + i] as u64;
            }
            return Ok(val_of(Expr { ty, kind: ExprKind::Const(ty.from_raw(raw)) }, &p.ty));
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
                Some(Binding::Const(Val::A(t, elems))) => self.materialize(t, elems),
                Some(_) => self.reject("ExprUnary(&)", format!("`{}` is not in memory", n.name)),
                None => match self.global(&n.name)? {
                    Some(Val::M(p)) => Ok(p),
                    Some(Val::A(t, elems)) => self.materialize(t, elems),
                    Some(Val::Poison) => Err(()),
                    Some(_) => self.reject("ExprUnary(&)", format!("address of constant `{}`", n.name)),
                    None => self.unknown_name(&n.name),
                },
            },
            NodeKind::ExprFieldAccess if n.children.len() == 1 => match self.member(n)? {
                Ok(p) => Ok(p),
                Err(_) => self.reject("ExprFieldAccess(.len)", "`.len` of an array is not a place".into()),
            },
            NodeKind::ExprIndex => match self.index(n)? {
                Ok(p) => Ok(p),
                Err(_) => self.reject("StmtAssign", "assignment through a constant".into()),
            },
            _ => {
                let k = kind_name(n);
                self.reject(&k, "not addressable".into())
            }
        }
    }

    /// `base[i]`: the place of an array element, or, for a constant index
    /// into a compile-time array, the element itself. A constant index out
    /// of range is rejected, as Zig rejects it at compile time; any other
    /// index is checked when it is evaluated and traps out of range, as
    /// Zig's safety check does.
    fn index(&mut self, n: &Node) -> R<Result<Place, Val>> {
        // `x[a..b]` parses as an index whose index is the range `a..b`;
        // `x[a:b]` and `x[a..]` as their own slice node.
        if n.extra_op.is_empty()
            && n.children.len() == 2
            && n.children[1].kind == NodeKind::ExprBinary
            && n.children[1].extra_op == ".."
            && n.children[1].children.len() == 2
        {
            self.see(&n.children[1]);
            let r = &n.children[1];
            return self.slicing(&n.children[0], &r.children[0], Some(&r.children[1])).map(Err);
        }
        match (n.extra_op.as_str(), n.children.len()) {
            ("slice", 3) => return self.slicing(&n.children[0], &n.children[1], Some(&n.children[2])).map(Err),
            ("slice_open", 2) => return self.slicing(&n.children[0], &n.children[1], None).map(Err),
            ("", 2) => {}
            ("", _) => return self.reject("ExprIndex", "unexpected shape".into()),
            (op, _) => return self.reject(&format!("ExprIndex({})", op), "unexpected shape".into()),
        }
        let base = self.expr(&n.children[0])?;
        let idx = self.expr(&n.children[1])?;
        if base.is_poison() || idx.is_poison() {
            return Err(());
        }
        // A string literal at a constant index is a constant.
        if let Val::S(k, len) = base {
            let c = match &idx {
                Val::Ct(c) => Some(*c),
                Val::E(Expr { kind: ExprKind::Const(c), ty }) if ty.is_int() && !ty.signed() => Some(*c),
                _ => None,
            };
            if let Some(c) = c {
                if !(0..len as i128).contains(&c) {
                    return self.reject("ExprIndex", format!("index {} out of bounds for a string of length {}", c, len));
                }
                let b = self.data[k as usize][c as usize];
                return Ok(Err(Val::E(Expr { ty: Ty::U8, kind: ExprKind::Const(b as i128) })));
            }
        }
        let base = match base {
            v @ Val::S(..) => self.coerce_to(v, &LTy::Str)?,
            v => v,
        };
        if let Val::M(p) = &base {
            if matches!(p.ty, LTy::Str | LTy::Slice(..)) {
                let Val::M(p) = base else { unreachable!() };
                return self.slice_index(p, idx).map(Ok);
            }
        }
        let base = match self.tuple_index(base, &idx)? {
            Ok(p) => return Ok(Ok(p)),
            Err(b) => b,
        };
        let p = match base {
            Val::A(t @ LTy::Arr(..), elems) => {
                let c = match &idx {
                    Val::Ct(c) => Some(*c),
                    Val::E(Expr { kind: ExprKind::Const(c), ty }) if ty.is_int() && !ty.signed() => Some(*c),
                    _ => None,
                };
                if let Some(c) = c {
                    if (0..elems.len() as i128).contains(&c) {
                        return Ok(Err(elems[c as usize].clone()));
                    }
                }
                self.materialize(t, elems)?
            }
            Val::M(p) if matches!(p.ty, LTy::Arr(..)) => p,
            Val::P(e, LTy::Ptr(inner, m)) if matches!(*inner, LTy::Arr(..)) => {
                Place { addr: e, off: 0, ty: *inner, mutable: m, temp: None }
            }
            v => {
                let d = self.val_desc(&v);
                return self.reject("ExprIndex", format!("index of {}", d));
            }
        };
        let LTy::Arr(elem, len) = p.ty.clone() else { unreachable!() };
        let (esize, _) = self.size_align(&elem)?;
        let mutable = p.mutable && p.temp.is_none();
        // Zig: the index is a usize.
        let e = self.coerce(idx, Ty::U64)?;
        if let ExprKind::Const(c) = e.kind {
            if c >= len as i128 {
                let t = self.type_name(&p.ty);
                return self.reject("ExprIndex", format!("index {} out of bounds for `{}`", c, t));
            }
            let mut q = elem_place(&p, &elem, esize, c as u32);
            q.mutable = mutable;
            return Ok(Ok(q));
        }
        let t = self.type_name(&p.ty);
        let site = self.site(TrapKind::Bounds, format!("index of {}", t), Ty::U64);
        let len = Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) };
        let checked = Expr { ty: Ty::U64, kind: ExprKind::Bounds { idx: Box::new(e), len: Box::new(len), site } };
        let addr = Expr {
            ty: Ty::Ptr,
            kind: ExprKind::Offset { base: Box::new(addr_of(&p)), idx: Box::new(checked), scale: esize },
        };
        Ok(Ok(Place { addr, off: 0, ty: *elem, mutable, temp: None }))
    }

    /// The element type of a slice type and whether its elements may be
    /// written.
    fn slice_elem(t: &LTy) -> (LTy, bool) {
        match t {
            LTy::Slice(elem, m) => ((**elem).clone(), *m),
            _ => (LTy::S(Ty::U8), false),
        }
    }

    /// `s[i]` for a slice or str `s`: the header is read once, the index is
    /// checked against its length when evaluated, and traps out of range.
    fn slice_index(&mut self, p: Place, idx: Val) -> R<Place> {
        let (elem, m) = Self::slice_elem(&p.ty);
        let (esize, _) = self.size_align(&elem)?;
        let mut pin = Vec::new();
        let (hdr, off) = self.pin_header(&p, &mut pin)?;
        let e = self.coerce(idx, Ty::U64)?;
        let t = self.type_name(&p.ty);
        let site = self.site(TrapKind::Bounds, format!("index of {}", t), Ty::U64);
        let mut ptr = Expr { ty: Ty::Ptr, kind: ExprKind::Load { addr: Box::new(hdr.clone()), off } };
        if !pin.is_empty() {
            ptr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts: pin, value: Box::new(ptr) } };
        }
        let len = Expr { ty: Ty::U64, kind: ExprKind::Load { addr: Box::new(hdr), off: off + 8 } };
        let checked = Expr { ty: Ty::U64, kind: ExprKind::Bounds { idx: Box::new(e), len: Box::new(len), site } };
        let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Offset { base: Box::new(ptr), idx: Box::new(checked), scale: esize } };
        Ok(Place { addr, off: 0, ty: elem, mutable: m, temp: None })
    }

    /// `x[i..j]` (or `x[i..]`, to the end) of an array, a pointer to an
    /// array, a slice or a string: a new slice, built in a temporary. As in
    /// Zig, `i <= j <= len` is checked when the slice is evaluated (at
    /// compile time where all three are constants) and traps otherwise.
    fn slicing(&mut self, base: &Node, start: &Node, end: Option<&Node>) -> R<Val> {
        let construct = if end.is_some() { "ExprIndex(slice)" } else { "ExprIndex(slice_open)" };
        let base = self.expr(base)?;
        if base.is_poison() {
            return Err(());
        }
        let base = match base {
            v @ Val::S(..) => self.coerce_to(v, &LTy::Str)?,
            Val::A(t, elems) => Val::M(self.materialize(t, elems)?),
            v => v,
        };
        // The source: an array place or a slice place.
        let (p, mutable) = match base {
            Val::M(p) if matches!(p.ty, LTy::Arr(..)) => {
                let m = p.mutable && p.temp.is_none();
                (p, m)
            }
            Val::P(e, LTy::Ptr(inner, m)) if matches!(*inner, LTy::Arr(..)) => {
                (Place { addr: e, off: 0, ty: *inner, mutable: m, temp: None }, m)
            }
            Val::M(p) if matches!(p.ty, LTy::Str | LTy::Slice(..)) => {
                let m = Self::slice_elem(&p.ty).1;
                (p, m)
            }
            v => {
                let d = self.val_desc(&v);
                return self.reject(construct, format!("slice of {}", d));
            }
        };
        let elem = match &p.ty {
            LTy::Arr(elem, _) => (**elem).clone(),
            t => Self::slice_elem(t).0,
        };
        let (esize, _) = self.size_align(&elem)?;
        let rt = if elem == LTy::S(Ty::U8) && !mutable { LTy::Str } else { LTy::Slice(Box::new(elem), mutable) };
        let tname = self.type_name(&p.ty);
        let hdr = self.new_slot(&rt)?;
        let scr = self.new_slot(&LTy::S(Ty::U64))?;
        let at = |k: u32, off: u32, ty: Ty| Expr { ty, kind: ExprKind::Load { addr: Box::new(slot_expr(k)), off } };
        let mut stmts = Vec::new();
        // S0: the header of the whole source.
        let alen = if let LTy::Arr(_, len) = p.ty {
            stmts.push(Stmt::Store { addr: slot_expr(hdr), off: 0, value: addr_of(&p) });
            let c = Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) };
            stmts.push(Stmt::Store { addr: slot_expr(hdr), off: 8, value: c });
            Some(len)
        } else {
            stmts.push(Stmt::Copy { dst: slot_expr(hdr), src: addr_of(&p), size: 16 });
            None
        };
        // S1: the start.
        let i = self.expr(start)?;
        let i = self.coerce(i, Ty::U64)?;
        let ci = if let ExprKind::Const(c) = i.kind { Some(c) } else { None };
        stmts.push(Stmt::Store { addr: slot_expr(scr), off: 0, value: i });
        let one = Expr { ty: Ty::U64, kind: ExprKind::Const(1) };
        let len_plus_1 = |l: Expr| Expr {
            ty: Ty::U64,
            kind: ExprKind::Arith { op: ArithOp::AddW, lhs: Box::new(l), rhs: Box::new(one.clone()), site: 0 },
        };
        // S2: the end, checked against the length, becomes the length.
        let mut cj = alen.map(|l| l as i128);
        if let Some(end) = end {
            let j = self.expr(end)?;
            let j = self.coerce(j, Ty::U64)?;
            cj = if let ExprKind::Const(c) = j.kind { Some(c) } else { None };
            if let (Some(c), Some(l)) = (cj, alen) {
                if c > l as i128 {
                    return self.reject(construct, format!("end {} out of bounds for `{}`", c, tname));
                }
            }
            let site = self.site(TrapKind::Bounds, format!("end of a slice of {}", tname), Ty::U64);
            let len = len_plus_1(at(hdr, 8, Ty::U64));
            let checked = Expr { ty: Ty::U64, kind: ExprKind::Bounds { idx: Box::new(j), len: Box::new(len), site } };
            stmts.push(Stmt::Store { addr: slot_expr(hdr), off: 8, value: checked });
        }
        if let (Some(a), Some(b)) = (ci, cj) {
            if a > b {
                return self.reject(construct, format!("start {} is past end {} in a slice of `{}`", a, b, tname));
            }
        }
        // S3: start <= end.
        let site = self.site(TrapKind::Bounds, format!("start of a slice of {}", tname), Ty::U64);
        let len = len_plus_1(at(hdr, 8, Ty::U64));
        let checked = Expr { ty: Ty::U64, kind: ExprKind::Bounds { idx: Box::new(at(scr, 0, Ty::U64)), len: Box::new(len), site } };
        stmts.push(Stmt::Eval(checked));
        // S4, S5: advance the pointer, shorten the length.
        let ptr = Expr {
            ty: Ty::Ptr,
            kind: ExprKind::Offset { base: Box::new(at(hdr, 0, Ty::Ptr)), idx: Box::new(at(scr, 0, Ty::U64)), scale: esize },
        };
        stmts.push(Stmt::Store { addr: slot_expr(hdr), off: 0, value: ptr });
        let rest = Expr {
            ty: Ty::U64,
            kind: ExprKind::Arith {
                op: ArithOp::SubW,
                lhs: Box::new(at(hdr, 8, Ty::U64)),
                rhs: Box::new(at(scr, 0, Ty::U64)),
                site: 0,
            },
        };
        stmts.push(Stmt::Store { addr: slot_expr(hdr), off: 8, value: rest });
        let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts, value: Box::new(slot_expr(hdr)) } };
        Ok(Val::M(Place { addr, off: 0, ty: rt, mutable: false, temp: Some(hdr) }))
    }

    /// `base.name`: the place of a field (or of `p.*`), or, for `.len` of
    /// an array, its value.
    fn member(&mut self, n: &Node) -> R<Result<Place, Val>> {
        if n.children.len() != 1 {
            return self.reject("ExprFieldAccess", "unexpected shape".into());
        }
        let base = &n.children[0];
        if n.name == "?" {
            return self.unwrap(base).map(Ok);
        }
        if n.name == "*" {
            return match self.expr(base)? {
                Val::P(e, LTy::Ptr(inner, m)) => {
                    if let LTy::Struct(id) = *inner {
                        self.layout(id)?;
                    }
                    Ok(Ok(Place { addr: e, off: 0, ty: *inner, mutable: m, temp: None }))
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
            if self.enum_nodes.contains_key(&base.name) {
                let Some(id) = self.enum_id(&base.name)? else { unreachable!() };
                return self.enum_value(id, &n.name).map(Err);
            }
            if self.recover && self.poison_names.contains(&base.name) {
                return Err(());
            }
            return self.reject("ExprFieldAccess", format!("`{}.{}`", base.name, n.name));
        }
        let p = match self.expr(base)? {
            Val::M(p) => p,
            Val::Poison => return Err(()),
            Val::A(LTy::Arr(_, len), _) if n.name == "len" => {
                return Ok(Err(Val::E(Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) })))
            }
            Val::A(t, elems) => self.materialize(t, elems)?,
            // Field access through a pointer dereferences it.
            Val::P(e, LTy::Ptr(inner, m)) if is_agg(&inner) => {
                Place { addr: e, off: 0, ty: *inner, mutable: m, temp: None }
            }
            v @ Val::S(..) => match self.coerce_to(v, &LTy::Str)? {
                Val::M(p) => p,
                _ => return Err(()),
            },
            _ => return self.reject("ExprFieldAccess", format!("`.{}` on a value that is not a struct", n.name)),
        };
        if matches!(p.ty, LTy::Str | LTy::Slice(..)) {
            if n.name != "len" {
                let (c, t) = if p.ty == LTy::Str { ("ExprFieldAccess(str)", "str".to_string()) } else { ("ExprFieldAccess(slice)", self.type_name(&p.ty)) };
                return self.reject(c, format!("`.{}` of a {}", n.name, t));
            }
            // `.len` is read-only here: a str is never resized in place.
            return Ok(Ok(Place { addr: p.addr, off: p.off + 8, ty: LTy::S(Ty::U64), mutable: false, temp: None }));
        }
        if let LTy::Arr(_, len) = p.ty {
            if n.name != "len" {
                return self.reject("ExprFieldAccess", format!("`.{}` of an array", n.name));
            }
            // A compile-time constant; the array is still evaluated.
            let c = Expr { ty: Ty::U64, kind: ExprKind::Const(len as i128) };
            if pure_addr(&p.addr) {
                return Ok(Err(Val::E(c)));
            }
            let kind = ExprKind::Seq { stmts: vec![Stmt::Eval(p.addr)], value: Box::new(c) };
            return Ok(Err(Val::E(Expr { ty: Ty::U64, kind })));
        }
        if let LTy::Opt(_) = p.ty {
            let t = self.type_name(&p.ty);
            return self.reject("ExprFieldAccess(?T)", format!("`.{}` of a {} without `.?`", n.name, t));
        }
        let LTy::Struct(id) = p.ty else { unreachable!() };
        let fields = self.fields(id)?;
        match fields.iter().find(|f| f.name == n.name) {
            Some(f) => {
                // A field of a temporary is read from it only once.
                let mut q = field_place(&p, f);
                q.mutable = p.mutable && p.temp.is_none();
                Ok(Ok(q))
            }
            None => {
                let s = self.structs[id as usize].name.clone();
                self.reject("ExprFieldAccess", format!("`{}` has no field `{}`", s, n.name))
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
            let d = if matches!(dst.ty, LTy::Enum(..)) { "an enum" } else { "a pointer or a struct" };
            return self.reject("StmtAssign", format!("`{}` on {}", op, d));
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
        if let Some(lit) = self.expand_repeat(n, &t)? {
            return self.rodata(&lit, t);
        }
        // An optional is filled by `const_fill`, which also copies another
        // optional constant.
        let opt = matches!(t, LTy::Opt(_));
        if !opt && n.kind != NodeKind::ExprStructLit && n.kind != NodeKind::ExprArrayLiteral {
            let v = self.expr_as(n, &t)?;
            return match v {
                Val::M(p) if matches!(p.addr.kind, ExprKind::Data(_)) => Ok(Val::M(p)),
                Val::Poison => Err(()),
                _ => self.reject("ConstDecl", "struct or array constant is not a compile-time value".into()),
            };
        }
        let (size, _) = self.size_align(&t)?;
        let mut buf = vec![0u8; size as usize];
        self.const_fill(n, &t, &mut buf, 0)?;
        self.data.push(buf);
        let k = (self.data.len() - 1) as u32;
        Ok(Val::M(Place { addr: Expr { ty: Ty::Ptr, kind: ExprKind::Data(k) }, off: 0, ty: t, mutable: false, temp: None }))
    }

    /// A module-level array or struct constant with strings in it:
    /// `Val::A`, one element per array element or per struct field (in
    /// declaration order). Read-only data cannot hold the address of a
    /// string, so the value is written into a frame temporary wherever
    /// memory is needed (`materialize`).
    fn const_agg(&mut self, n: &Node, t: &LTy) -> R<Val> {
        self.see(n);
        if let Some(lit) = self.expand_repeat(n, t)? {
            return self.const_agg(&lit, t);
        }
        let literal = match t {
            LTy::Arr(..) => n.kind == NodeKind::ExprArrayLiteral,
            _ => n.kind == NodeKind::ExprStructLit,
        };
        if !literal {
            let v = self.expr(n)?;
            return match v {
                Val::A(ref u, _) if u == t => Ok(v),
                Val::Poison => Err(()),
                _ => {
                    let a = self.type_name(t);
                    self.reject("ConstDecl", format!("`{}` constant is not a compile-time value", a))
                }
            };
        }
        let mut elems = Vec::new();
        match t {
            LTy::Arr(elem, _) => {
                self.array_count(n, t)?;
                for c in &n.children {
                    elems.push(self.const_elem(c, elem)?);
                }
            }
            LTy::Struct(id) => {
                self.lit_type(n, t)?;
                let fields = self.fields(*id)?;
                let sname = self.structs[*id as usize].name.clone();
                let mut init: Vec<Option<&Node>> = vec![None; fields.len()];
                for c in &n.children {
                    if c.kind != NodeKind::ExprFieldAccess || c.children.len() != 1 {
                        return self.reject("ExprStructLit", format!("positional initializer in a `{}` literal", sname));
                    }
                    let Some(i) = fields.iter().position(|f| f.name == c.name) else {
                        return self.reject("ExprStructLit", format!("`{}` has no field `{}`", sname, c.name));
                    };
                    if init[i].is_some() {
                        return self.reject("ExprStructLit", format!("field `{}` initialised twice", c.name));
                    }
                    init[i] = Some(&c.children[0]);
                }
                for (i, f) in fields.iter().enumerate() {
                    let Some(c) = init[i].or(f.default) else {
                        return self.reject("ExprStructLit", format!("missing field `{}` of `{}`", f.name, sname));
                    };
                    elems.push(self.const_elem(c, &f.ty)?);
                }
            }
            _ => unreachable!(),
        }
        Ok(Val::A(t.clone(), elems))
    }

    /// One element (or field) of a compile-time aggregate: a string
    /// literal, a nested `Val::A`, an aggregate without strings in
    /// read-only data, or a constant scalar or enum.
    fn const_elem(&mut self, c: &Node, t: &LTy) -> R<Val> {
        self.see(c);
        if is_undefined(c) {
            return self.reject("ConstDecl", "`undefined` in a constant with strings".into());
        }
        match t {
            LTy::Str => match self.expr(c)? {
                v @ Val::S(..) => Ok(v),
                Val::Poison => Err(()),
                _ => self.reject("ConstDecl", "str element is not a string literal".into()),
            },
            LTy::Struct(_) | LTy::Arr(..) if self.holds_str(t)? => self.const_agg(c, t),
            LTy::Struct(_) | LTy::Arr(..) => self.rodata(c, t.clone()),
            LTy::Opt(_) => self.rodata(c, t.clone()),
            LTy::S(ty) => {
                let v = self.expr_as(c, t)?;
                let e = self.coerce(v, *ty)?;
                if !matches!(e.kind, ExprKind::Const(_)) {
                    return self.reject("ConstDecl", "struct field is not a compile-time value".into());
                }
                Ok(Val::E(e))
            }
            LTy::Enum(..) => match self.expr_as(c, t)? {
                v @ Val::P(Expr { kind: ExprKind::Const(_), .. }, _) => Ok(v),
                Val::Poison => Err(()),
                _ => self.reject("ConstDecl", "enum field is not a compile-time value".into()),
            },
            LTy::Ptr(..) => self.reject("ConstDecl", "pointer in a constant struct".into()),
            LTy::Slice(..) => self.reject("ConstDecl(slice)", "slice in a module-level constant".into()),
        }
    }

    /// Whether a type has a `str` at a leaf (through arrays and struct
    /// fields): no read-only image of it can exist.
    fn holds_str(&mut self, t: &LTy) -> R<bool> {
        match t {
            LTy::Str => Ok(true),
            LTy::Arr(inner, _) => self.holds_str(inner),
            LTy::Struct(id) => {
                for f in self.fields(*id)? {
                    if self.holds_str(&f.ty)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// An array literal's element count must be the array's length.
    fn array_count(&mut self, n: &Node, t: &LTy) -> R<()> {
        let LTy::Arr(_, len) = t else { unreachable!() };
        if n.children.is_empty() && n.extra_type.trim().is_empty() && !n.extra_size.trim().is_empty() {
            return self.reject(
                "ExprArrayLiteral(text form)",
                format!("`[{}]`: elements kept as text", n.extra_size.trim()),
            );
        }
        if n.children.len() != *len as usize {
            let a = self.type_name(t);
            return self.reject(
                "ExprArrayLiteral",
                format!("{} elements for `{}`", n.children.len(), a),
            );
        }
        Ok(())
    }

    /// A repeated array literal, for array type `t`: `[v; n]` (whose element
    /// the parser keeps as the text `v;n`) or `[_]T{ a, b } ** n`. t27c's
    /// Zig backend writes both as `.{ ... } ** n`, so the elements are
    /// evaluated once and the result holds them `n` times over. Returns the
    /// elements and `n`; `None` when `n` is not a repeat.
    fn repeat_lit(&mut self, n: &Node, t: &LTy) -> R<Option<(Vec<Node>, u32)>> {
        let LTy::Arr(_, len) = t else { return Ok(None) };
        let tn = self.type_name(t);
        let (elems, count) = if is_repeat_op(n) {
            // `[1] ** n` keeps its elements as text; the reference pastes
            // them back as `.{ 1 } ** n`, so they are parsed back the same way.
            let text = self.text_lit(&n.children[0])?;
            let lhs = text.as_ref().unwrap_or(&n.children[0]);
            if lhs.children.is_empty() {
                return self.reject(
                    "ExprArrayLiteral(repeat)",
                    "`** n` applied to an empty array literal".into(),
                );
            }
            let c = &n.children[1];
            let text = match c.kind {
                NodeKind::ExprLiteral => c.value.clone(),
                NodeKind::ExprIdentifier => c.name.clone(),
                _ => return self.reject("ExprArrayLiteral(repeat count)", "`** n` with `n` not a literal or a name".into()),
            };
            let k = self.array_len(&tn, text.trim())?;
            (lhs.children.clone(), k)
        } else if n.kind == NodeKind::ExprArrayLiteral && n.children.is_empty() && n.extra_type.trim().is_empty() {
            let txt = n.extra_size.trim();
            let Some((v, c)) = txt.rsplit_once(';') else { return Ok(None) };
            let k = self.array_len(&tn, c.trim())?;
            let e = self.text_elem(v.trim(), txt)?;
            (vec![e], k)
        } else {
            return Ok(None);
        };
        if count == 0 {
            return self.reject("ExprArrayLiteral(repeat count)", "an array literal repeated zero times".into());
        }
        let total = elems.len() as u64 * count as u64;
        if total != *len as u64 {
            return self.reject("ExprArrayLiteral", format!("{} elements for `{}`", total, tn));
        }
        Ok(Some((elems, count)))
    }

    /// A list literal whose elements the parser kept as text (`[s1]` comes
    /// back with no children and `extra_size` "s1"): the elements parsed
    /// back, split on top-level commas as t27c's Zig backend splits them.
    /// `None` for a literal with children, an empty one, or a repeat.
    fn text_lit(&mut self, n: &Node) -> R<Option<Node>> {
        let txt = n.extra_size.trim();
        // `[_]T{}` keeps its dimension, not elements, in the same field.
        let typed = !n.extra_type.trim().is_empty();
        if n.kind != NodeKind::ExprArrayLiteral || !n.children.is_empty() || typed || txt.is_empty() || txt.contains(';') {
            return Ok(None);
        }
        if txt.contains('{') || txt.contains("][") {
            return self.reject("ExprArrayLiteral(text form)", format!("`[{}]`: elements kept as text", txt));
        }
        let mut parts = Vec::new();
        let (mut depth, mut cur) = (0i32, String::new());
        for ch in txt.chars() {
            match ch {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                _ => {}
            }
            if ch == ',' && depth == 0 {
                parts.push(std::mem::take(&mut cur));
            } else {
                cur.push(ch);
            }
        }
        if !cur.trim().is_empty() {
            parts.push(cur);
        }
        let mut lit = Node::new(NodeKind::ExprArrayLiteral);
        lit.line = n.line;
        for p in parts {
            let e = self.text_elem(p.trim(), txt)?;
            lit.children.push(e);
        }
        Ok(Some(lit))
    }

    /// One element of a literal the parser kept as text (`whole`), parsed
    /// back from `text`. The reference pastes the text into Zig unchanged,
    /// so only shapes that mean the same in both languages are taken:
    /// literals, names, field access, calls of those, and a negated literal.
    /// A name must also mean the same: t27c renames a local that shadows a
    /// module-level name, and the pasted text would then see the
    /// module-level one.
    fn text_elem(&mut self, text: &str, whole: &str) -> R<Node> {
        let src = format!("module r {{ const r = {}; }}", text);
        let parsed = crate::compiler::Compiler::parse_ast_strict(&src).ok();
        let mut e = parsed.as_ref().and_then(|a| find_const(a, "r")).cloned();
        let ok = e.as_ref().is_some_and(simple_repeat_elem);
        let Some(mut e) = e.take().filter(|_| ok) else {
            return self.reject(
                "ExprArrayLiteral(text element)",
                format!("`[{}]`: element `{}` is not a literal, a name or a call (the reference pastes its text)", whole, text),
            );
        };
        let mut names = Vec::new();
        repeat_names(&e, &mut names);
        for nm in names {
            let module_level = self.const_nodes.contains_key(&nm)
                || self.mod_vars.contains_key(&nm)
                || self.sigs.contains_key(&nm);
            if module_level && self.lookup(&nm).is_some() {
                return self.reject(
                    "ExprArrayLiteral(text element)",
                    format!("`[{}]`: `{}` shadows a module-level name (the reference renames it, or Zig refuses the shadow)", whole, nm),
                );
            }
        }
        zero_lines(&mut e);
        Ok(e)
    }

    /// The elements of a repeat, into `dst` (whose address is pure): each
    /// one once, then the filled prefix copied forward, doubling.
    fn init_repeat(&mut self, elems: &[Node], count: u32, dst: &Place, out: &mut Vec<Stmt>) -> R<()> {
        let LTy::Arr(elem, _) = &dst.ty else { unreachable!() };
        let elem = (**elem).clone();
        let (esize, _) = self.size_align(&elem)?;
        for (i, c) in elems.iter().enumerate() {
            let p = elem_place(dst, &elem, esize, i as u32);
            self.init(c, p, true, out)?;
        }
        let total = elems.len() as u32 * count;
        let mut have = elems.len() as u32;
        while have < total && esize > 0 {
            let n = have.min(total - have);
            let d = elem_place(dst, &elem, esize, have);
            let s = elem_place(dst, &elem, esize, 0);
            out.push(Stmt::Copy { dst: addr_of(&d), src: addr_of(&s), size: n * esize });
            have += n;
        }
        Ok(())
    }

    /// The literal `n` for array type `t` with a repeat written out, for the
    /// compile-time paths, which evaluate every element anyway.
    fn expand_repeat(&mut self, n: &Node, t: &LTy) -> R<Option<Node>> {
        if !matches!(t, LTy::Arr(..)) {
            return Ok(None);
        }
        if let Some(lit) = self.text_lit(n)? {
            return Ok(Some(lit));
        }
        let Some((elems, count)) = self.repeat_lit(n, t)? else { return Ok(None) };
        if elems.len() as u64 * count as u64 > MAX_CONST_REPEAT {
            return self.reject("ExprArrayLiteral(repeat)", format!("a constant repeat of {} elements", elems.len() as u64 * count as u64));
        }
        let mut lit = Node::new(NodeKind::ExprArrayLiteral);
        lit.line = n.line;
        for _ in 0..count {
            lit.children.extend(elems.iter().cloned());
        }
        Ok(Some(lit))
    }

    fn const_fill(&mut self, n: &Node, t: &LTy, buf: &mut [u8], off: usize) -> R<()> {
        self.see(n);
        if let Some(lit) = self.expand_repeat(n, t)? {
            return self.const_fill(&lit, t, buf, off);
        }
        if is_undefined(n) {
            return Ok(());
        }
        match t {
            // `?T` as `opt_temp` lays it out: the payload, then the
            // has-value flag at the payload's size. `null` leaves both zero.
            LTy::Opt(inner) => {
                if self.holds_str(inner)? {
                    return self.reject("ConstDecl(?T)", "an optional holding a str in a module-level constant".into());
                }
                if self.is_null(n) {
                    return Ok(());
                }
                if n.kind == NodeKind::ExprIdentifier && self.lookup(&n.name).is_none() {
                    if let Some(Val::M(p)) = self.global(&n.name)? {
                        if let (true, ExprKind::Data(k)) = (&p.ty == t, &p.addr.kind) {
                            let (size, _) = self.size_align(t)?;
                            let src = &self.data[*k as usize][p.off as usize..][..size as usize];
                            buf[off..off + size as usize].copy_from_slice(src);
                            return Ok(());
                        }
                    }
                }
                let (s, _) = self.size_align(inner)?;
                self.const_fill(n, inner, buf, off)?;
                buf[off + s as usize] = 1;
                Ok(())
            }
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
            LTy::Enum(_, ty) => {
                let v = self.expr_as(n, t)?;
                let Val::P(Expr { kind: ExprKind::Const(c), .. }, _) = v else {
                    return self.reject("ConstDecl", "enum field is not a compile-time value".into());
                };
                let raw = c as u64;
                for i in 0..ty.bytes() as usize {
                    buf[off + i] = (raw >> (8 * i)) as u8;
                }
                Ok(())
            }
            LTy::Ptr(..) => self.reject("ConstDecl", "pointer in a constant struct".into()),
            LTy::Str => self.reject("ConstDecl(str field)", "str field in a module-level struct constant".into()),
            LTy::Slice(..) => self.reject("ConstDecl(slice)", "slice in a module-level constant".into()),
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
            LTy::Arr(elem, _) if n.kind == NodeKind::ExprArrayLiteral => {
                self.array_count(n, t)?;
                let (esize, _) = self.size_align(elem)?;
                for (i, c) in n.children.iter().enumerate() {
                    self.const_fill(c, elem, buf, off + i * esize as usize)?;
                }
                Ok(())
            }
            LTy::Struct(_) | LTy::Arr(..) => {
                let v = self.expr_as(n, t)?;
                let Val::M(p) = v else { return Err(()) };
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
/// The byte index of the `]` that closes the `[` at index 0 of `t`.
fn close_of_open(t: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in t.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether `t` spells a `[` in Zig: an array, a slice, a string, or a
/// pointer to one.
fn has_brackets(t: &LTy) -> bool {
    match t {
        LTy::Arr(..) | LTy::Slice(..) | LTy::Str => true,
        LTy::Ptr(inner, _) => has_brackets(inner),
        _ => false,
    }
}

/// Elements a constant repeat may expand to.
const MAX_CONST_REPEAT: u64 = 1 << 16;

/// `[_]T{ ... } ** n`, which the parser builds as a `**` binary node.
fn is_repeat_op(n: &Node) -> bool {
    n.kind == NodeKind::ExprBinary
        && n.extra_op == "**"
        && n.children.len() == 2
        && n.children[0].kind == NodeKind::ExprArrayLiteral
}

fn find_const<'n>(n: &'n Node, name: &str) -> Option<&'n Node> {
    if n.kind == NodeKind::ConstDecl && n.name == name && n.children.len() == 1 {
        return Some(&n.children[0]);
    }
    n.children.iter().find_map(|c| find_const(c, name))
}

/// Whether a repeat element parsed back from its text is one of the shapes
/// `repeat_elem` takes.
fn simple_repeat_elem(n: &Node) -> bool {
    match n.kind {
        NodeKind::ExprLiteral | NodeKind::ExprIdentifier | NodeKind::ExprEnumValue => n.children.is_empty(),
        NodeKind::ExprFieldAccess | NodeKind::ExprCall => n.children.iter().all(simple_repeat_elem),
        NodeKind::ExprUnary => {
            n.extra_op == "-" && n.children.len() == 1 && n.children[0].kind == NodeKind::ExprLiteral
        }
        _ => false,
    }
}

/// The names a repeat element reads (the first segment of a dotted path).
fn repeat_names(n: &Node, out: &mut Vec<String>) {
    if n.kind == NodeKind::ExprIdentifier {
        if let Some(h) = n.name.split(['.', ':']).next() {
            out.push(h.to_string());
        }
    }
    for c in &n.children {
        repeat_names(c, out);
    }
}

/// Parsed from text, a node has no line of its own; `see` then keeps the
/// line of the literal it came from.
fn zero_lines(n: &mut Node) {
    n.line = 0;
    for c in &mut n.children {
        zero_lines(c);
    }
}

/// Names a statement list binds to an array literal (`collect_array_locals`
/// in t27c).
fn array_locals(ns: &[Node], out: &mut HashSet<String>) {
    for n in ns {
        if matches!(n.kind, NodeKind::StmtLocal | NodeKind::StmtAssign)
            && !n.name.is_empty()
            && n.children.first().is_some_and(|c| c.kind == NodeKind::ExprArrayLiteral)
        {
            out.insert(n.name.clone());
        }
        array_locals(&n.children, out);
    }
}

/// The local declarations of `name` under `ns`, at every nesting level.
fn decls_of<'n>(ns: &'n [Node], name: &str, out: &mut Vec<&'n Node>) {
    for n in ns {
        if matches!(n.kind, NodeKind::StmtLocal | NodeKind::StmtAssign) && n.name == name {
            out.push(n);
        }
        decls_of(&n.children, name, out);
    }
}

/// How many nodes under `ns` carry `name` (or a dotted path starting with
/// it) as their name, of any kind: an over-count of its uses.
fn name_count(ns: &[Node], name: &str) -> usize {
    ns.iter()
        .map(|n| {
            let hit = n.name == name || n.name.strip_prefix(name).is_some_and(|r| r.starts_with('.'));
            hit as usize + name_count(&n.children, name)
        })
        .sum()
}

/// `name_count`, plus each node whose type or size text names `name` as a
/// word (array literal elements and array sizes are kept as text): an
/// over-count of its mentions.
fn name_mentions(ns: &[Node], name: &str) -> usize {
    fn word_in(text: &str, name: &str) -> bool {
        let b = text.as_bytes();
        let id = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
        text.match_indices(name).any(|(i, _)| {
            let e = i + name.len();
            (i == 0 || !id(b[i - 1])) && (e >= b.len() || !id(b[e]))
        })
    }
    ns.iter()
        .map(|n| {
            let hit = n.name == name || n.name.strip_prefix(name).is_some_and(|r| r.starts_with('.'));
            let text = [&n.extra_size, &n.extra_type, &n.extra_field, &n.extra_return_type]
                .iter()
                .any(|t| word_in(t, name));
            hit as usize + text as usize + name_mentions(&n.children, name)
        })
        .sum()
}

/// How many `_ = name;` statements lie under `ns`, at any depth.
fn discard_count(ns: &[Node], name: &str) -> usize {
    ns.iter()
        .map(|n| {
            let hit = n.kind == NodeKind::StmtAssign
                && matches!(n.extra_op.as_str(), "" | "=")
                && n.children.len() == 2
                && n.children[0].kind == NodeKind::ExprIdentifier
                && n.children[0].name == "_"
                && n.children[1].kind == NodeKind::ExprIdentifier
                && n.children[1].name == name;
            hit as usize + discard_count(&n.children, name)
        })
        .sum()
}

/// An array literal whose elements are all plain literals (a negated one
/// too): evaluating it does nothing.
fn plain_lit(n: &Node) -> bool {
    if n.kind != NodeKind::ExprArrayLiteral {
        return false;
    }
    if n.children.is_empty() {
        return n.extra_size.trim().is_empty();
    }
    n.children.iter().all(|c| {
        c.kind == NodeKind::ExprLiteral
            || (c.kind == NodeKind::ExprUnary
                && c.extra_op == "-"
                && c.children.len() == 1
                && c.children[0].kind == NodeKind::ExprLiteral)
    })
}

fn calls_in<'n>(ns: &'n [Node], out: &mut Vec<&'n Node>) {
    for n in ns {
        if n.kind == NodeKind::ExprCall {
            out.push(n);
        }
        calls_in(&n.children, out);
    }
}

fn reg_ty(t: &LTy) -> Option<Ty> {
    match t {
        LTy::S(ty) => Some(*ty),
        LTy::Ptr(..) => Some(Ty::Ptr),
        LTy::Enum(_, ty) => Some(*ty),
        LTy::Struct(_) | LTy::Str | LTy::Arr(..) | LTy::Slice(..) | LTy::Opt(_) => None,
    }
}

/// Lives in memory and is passed by reference: a struct, a `str`, an
/// array or a slice.
fn is_agg(t: &LTy) -> bool {
    matches!(t, LTy::Struct(_) | LTy::Str | LTy::Arr(..) | LTy::Slice(..) | LTy::Opt(_))
}

/// The place of element `i` (a constant) of array place `p`.
fn elem_place(p: &Place, elem: &LTy, esize: u32, i: u32) -> Place {
    Place { addr: p.addr.clone(), off: p.off + i * esize, ty: elem.clone(), mutable: p.mutable, temp: None }
}

/// `__t27b_str_eql(a: *const str, b: *const str) bool`: equal lengths and
/// equal bytes, what `std.mem.eql(u8, a, b)` computes.
fn str_eql_func(noreturn_site: SiteId) -> Func {
    let var = |id: VarId, ty: Ty| Expr { ty, kind: ExprKind::Var(id) };
    let cnst = |ty: Ty, v: i128| Expr { ty, kind: ExprKind::Const(v) };
    let load = |addr: Expr, off: u32, ty: Ty| Expr { ty, kind: ExprKind::Load { addr: Box::new(addr), off } };
    let ne = |a: Expr, b: Expr| Expr { ty: Ty::Bool, kind: ExprKind::Cmp { op: CmpOp::Ne, lhs: Box::new(a), rhs: Box::new(b) } };
    let byte = |s: VarId| {
        let base = load(var(s, Ty::Ptr), 0, Ty::Ptr);
        let at = Expr {
            ty: Ty::Ptr,
            kind: ExprKind::Offset { base: Box::new(base), idx: Box::new(var(3, Ty::U64)), scale: 1 },
        };
        load(at, 0, Ty::U8)
    };
    let (a, b, n, i) = (0, 1, 2, 3);
    let body = vec![
        Stmt::Assign { var: n, value: load(var(a, Ty::Ptr), 8, Ty::U64) },
        Stmt::If {
            cond: ne(load(var(b, Ty::Ptr), 8, Ty::U64), var(n, Ty::U64)),
            then: vec![Stmt::Return(Some(cnst(Ty::Bool, 0)))],
            els: vec![],
        },
        Stmt::Assign { var: i, value: cnst(Ty::U64, 0) },
        Stmt::While {
            cond: Expr {
                ty: Ty::Bool,
                kind: ExprKind::Cmp { op: CmpOp::Lt, lhs: Box::new(var(i, Ty::U64)), rhs: Box::new(var(n, Ty::U64)) },
            },
            body: vec![Stmt::If {
                cond: ne(byte(a), byte(b)),
                then: vec![Stmt::Return(Some(cnst(Ty::Bool, 0)))],
                els: vec![],
            }],
            // i < n <= 2^64 - 1, so i + 1 cannot wrap.
            step: vec![Stmt::Assign {
                var: i,
                value: Expr {
                    ty: Ty::U64,
                    kind: ExprKind::Arith {
                        op: ArithOp::AddW,
                        lhs: Box::new(var(i, Ty::U64)),
                        rhs: Box::new(cnst(Ty::U64, 1)),
                        site: 0,
                    },
                },
            }],
        },
        Stmt::Return(Some(cnst(Ty::Bool, 1))),
    ];
    let v = |name: &str, ty: Ty| Var { name: name.to_string(), ty };
    Func {
        name: STR_EQL.to_string(),
        nparams: 2,
        ret: Some(Ty::Bool),
        vars: vec![v("a", Ty::Ptr), v("b", Ty::Ptr), v("n", Ty::U64), v("i", Ty::U64)],
        body,
        line: 0,
        is_test: false,
        is_invariant: false,
        noreturn_site,
        slots: Vec::new(),
    }
}

/// The one function lowering synthesizes; not a name t27 source can declare
/// (`__t27b_` is reserved to the backend).
/// Most parameters (the hidden result pointer included) of one function.
/// Past 8 of a class they go on the stack; the call's outgoing area must
/// stay one `sub sp` immediate (codegen), far below this.
const MAX_PARAMS: usize = 64;

pub const STR_EQL: &str = "__t27b_str_eql";

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
        ExprKind::Slot(_) | ExprKind::Data(_) | ExprKind::Global(_) | ExprKind::Var(_) => true,
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
/// Whether `name` occurs as an identifier anywhere under `ns`.
/// Every name a subtree mentions (identifiers, callees, anything named): an
/// over-approximation of what it references, so a fn left out of
/// `analyzed_fns` is one Zig cannot reach.
fn names_in(ns: &[Node], out: &mut HashSet<String>) {
    for n in ns {
        if !n.name.is_empty() {
            out.insert(n.name.clone());
        }
        names_in(&n.children, out);
    }
}

/// Collect the `if` expressions the reference prints unparenthesized where the
/// source parenthesized them (see `Lower::misprinted_if`): the first child of
/// a binary operator, a field access or an index. An `if` on the right of a
/// binary operator, or as a call argument, prints with the meaning it has.
/// So does a struct literal's field value: `.f = if (c) a else b` is an
/// `ExprFieldAccess` named `f` holding the value, printed as `.f = <value>,`,
/// and the comma ends the `if`.
fn misprinted_ifs(n: &Node, out: &mut HashSet<usize>) {
    if matches!(n.kind, NodeKind::ExprBinary | NodeKind::ExprFieldAccess | NodeKind::ExprIndex) {
        if let Some(c) = n.children.first() {
            if c.kind == NodeKind::ExprIf {
                out.insert(c as *const Node as usize);
            }
        }
    }
    for c in &n.children {
        if n.kind == NodeKind::ExprStructLit && c.kind == NodeKind::ExprFieldAccess {
            for v in &c.children {
                misprinted_ifs(v, out);
            }
        } else {
            misprinted_ifs(c, out);
        }
    }
}

/// `m::GF16` / `a::b::gf16`: a module path whose last segment is GF16.
fn is_scoped_gf16(t: &str) -> bool {
    let segs: Vec<&str> = t.split("::").collect();
    segs.len() > 1
        && matches!(segs[segs.len() - 1], "GF16" | "gf16")
        && segs.iter().all(|s| {
            s.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// The node kinds t27c's `gen_expr` prints anything for; every other kind (a
/// statement) renders as the empty string.
fn zig_renders(k: &NodeKind) -> bool {
    matches!(
        k,
        NodeKind::ExprLiteral
            | NodeKind::ExprIdentifier
            | NodeKind::ExprEnumValue
            | NodeKind::ExprCall
            | NodeKind::ExprBinary
            | NodeKind::ExprUnary
            | NodeKind::ExprFieldAccess
            | NodeKind::ExprIndex
            | NodeKind::ExprSwitch
            | NodeKind::ExprIf
            | NodeKind::ExprArrayLiteral
            | NodeKind::ExprStructLit
            | NodeKind::ExprCast
            | NodeKind::ExprTuple
    )
}

/// Collect the `*` expressions t27c's strength reduction rewrites as `<<`
/// when the right side is a power-of-two literal (`strength_reduce` in
/// bootstrap/src/compiler.rs): those reached from a top-level statement of a
/// module-level fn body -- an assignment's value, a local's initializer, a
/// `return` value -- through binary operators only. Nothing else is
/// rewritten: not a test or an invariant, not a statement nested in an `if`
/// or a loop, not a call argument.
fn strength_reduced(items: &[&Node], out: &mut HashSet<usize>) {
    fn walk(n: &Node, out: &mut HashSet<usize>) {
        if n.kind == NodeKind::ExprBinary && n.children.len() >= 2 {
            walk(&n.children[0], out);
            walk(&n.children[1], out);
            if n.extra_op == "*" {
                out.insert(n as *const Node as usize);
            }
        }
    }
    for f in items.iter().filter(|n| n.kind == NodeKind::FnDecl) {
        let body = f.children.iter().filter(|c| c.kind == NodeKind::Module && c.name == "body");
        for s in f.children.iter().chain(body.flat_map(|c| c.children.iter())) {
            match s.kind {
                NodeKind::StmtAssign if s.children.len() >= 2 => walk(&s.children[1], out),
                NodeKind::StmtLocal | NodeKind::ExprReturn if !s.children.is_empty() => walk(&s.children[0], out),
                _ => {}
            }
        }
    }
}

/// The fns Zig's lazy analysis may reach in `zig test`: the roots are every
/// top-level item that is not a fn (tests; invariants, which t27c emits as
/// `comptime` blocks; constants and vars), and a fn is reached when a reached
/// body names it. `pub` does not make a fn a root, nor does `main`. A bench
/// is not a root: t27c emits it as `fn bench_<name>() void`, which nothing
/// calls, so neither its body nor a fn only it names is analyzed. Nor is a
/// top-level statement, which t27c does not emit at all.
fn analyzed_fns(items: &[&Node]) -> HashSet<String> {
    let mut bodies: HashMap<&str, Vec<&Node>> = HashMap::new();
    let mut work: HashSet<String> = HashSet::new();
    for item in items {
        if item.kind == NodeKind::FnDecl {
            bodies.entry(item.name.as_str()).or_default().push(item);
        } else if !matches!(item.kind, NodeKind::BenchBlock | NodeKind::StmtExpr) {
            names_in(std::slice::from_ref(*item), &mut work);
        }
    }
    let mut reached: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = work.into_iter().collect();
    while let Some(name) = stack.pop() {
        let Some(fns) = bodies.get(name.as_str()) else { continue };
        if !reached.insert(name.clone()) {
            continue;
        }
        let mut more = HashSet::new();
        for f in fns {
            names_in(&f.children, &mut more);
        }
        stack.extend(more.into_iter().filter(|m| !reached.contains(m)));
    }
    reached
}

/// A return type that is a value (t27c `call_returns_value` and the test
/// in `gen_fn_decl` before `zig_tail_returns`): anything but none, `void`,
/// `noreturn` and `()`.
fn returns_value(rt: &str) -> bool {
    !matches!(rt.trim(), "" | "void" | "noreturn" | "()")
}

/// #6315, t27c `zig_tail_returns`: the reference turns the last statement of
/// a non-void fn body into a `return` when it is an expression statement
/// other than `return` and the action calls `assert`, `assert_eq`, `panic`,
/// `unreachable`, `print` and `println`, and through an if/else that is last,
/// the last statement of each branch. Marks those statements by address.
fn mark_tail_returns(ns: &[Node], out: &mut HashSet<usize>) {
    let Some(last) = ns.last() else { return };
    match last.kind {
        NodeKind::StmtExpr if last.children.len() == 1 => {
            let e = &last.children[0];
            let action = match e.kind {
                NodeKind::ExprReturn => true,
                NodeKind::ExprCall => {
                    matches!(e.name.as_str(), "assert" | "assert_eq" | "panic" | "unreachable" | "print" | "println")
                }
                _ => false,
            };
            if !action {
                out.insert(last as *const Node as usize);
            }
        }
        NodeKind::StmtIf if last.children.len() == 3 => {
            mark_tail_returns(&last.children[1].children, out);
            mark_tail_returns(&last.children[2].children, out);
        }
        _ => {}
    }
}

/// An expression whose value Zig refuses to drop when it is a statement
/// (`value of type ... ignored`). Calls, `return`, `try`, `if`, `switch` and
/// `undefined` have their own rules and are not in this set.
fn is_value_stmt(c: &Node) -> bool {
    match c.kind {
        NodeKind::ExprBinary
        | NodeKind::ExprLiteral
        | NodeKind::ExprCast
        | NodeKind::ExprFieldAccess
        | NodeKind::ExprIndex
        | NodeKind::ExprStructLit
        | NodeKind::ExprArrayLiteral => true,
        NodeKind::ExprIdentifier => c.name != "undefined",
        _ => false,
    }
}

fn mentions(ns: &[Node], name: &str) -> bool {
    ns.iter().any(|n| (n.kind == NodeKind::ExprIdentifier && n.name == name) || mentions(&n.children, name))
}

/// The reference's `collect_mutable_names`: is `name` the target (or the base
/// of an indexed or field target) of an assignment in `ns`?
fn mutated(ns: &[Node], name: &str) -> bool {
    ns.iter().any(|n| {
        if n.kind == NodeKind::StmtAssign {
            if let Some(t) = n.children.first() {
                let base = match t.kind {
                    NodeKind::ExprIndex | NodeKind::ExprFieldAccess => t.children.first(),
                    _ => Some(t),
                };
                if base.is_some_and(|b| b.kind == NodeKind::ExprIdentifier && b.name == name) {
                    return true;
                }
            }
        }
        mutated(&n.children, name)
    })
}

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

/// A clause the front-end kept only as verbatim text: a childless StmtExpr
/// named `<word>:` (`measure:`, `target:`), its text in `value`.
fn is_prose_clause(n: &Node) -> bool {
    n.kind == NodeKind::StmtExpr
        && n.children.is_empty()
        && n.name.len() > 1
        && n.name.ends_with(':')
        && n.name[..n.name.len() - 1].chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ------------------------------------------------- reference-path defects

/// A struct field type the reference emits as a name Zig cannot resolve: a
/// generic `List<T>` (a parse error) or a bare identifier that is neither a
/// Zig primitive, nor one t27c's type mapper rewrites, nor a top-level
/// declaration. Returns the offending base name. Anything else (dotted or
/// scoped paths, `@This()`, function types) is left alone.
fn undeclared_field_type(ty: &str, declared: &HashSet<&str>) -> Option<String> {
    let base = type_base(ty)?;
    if base.is_empty() {
        return None;
    }
    if base.contains('<') {
        return Some(base.to_string());
    }
    let ident = base.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && base.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ident || declared.contains(base) || zig_type_name(base) {
        return None;
    }
    if matches!(base, "str" | "string" | "float" | "double" | "int" | "uint" | "GF16" | "gf16") {
        return None;
    }
    Some(base.to_string())
}

/// The element name under `?`, `&`, `*`, `const`, and the array and slice
/// forms of both t27 (`[T]`, `[T; N]`) and Zig (`[]T`, `[N]T`). `None` for a
/// map type `[K:V]`, which is not a name.
fn type_base(ty: &str) -> Option<&str> {
    let t = ty.trim();
    if let Some(r) = t.strip_prefix('?').or_else(|| t.strip_prefix('&')).or_else(|| t.strip_prefix('*')) {
        return type_base(r);
    }
    if let Some(r) = t.strip_prefix("const ") {
        return type_base(r);
    }
    // #7415: the postfix optional `T?`, which t27c writes as `?T`.
    if let Some(r) = t.strip_suffix('?') {
        if !r.trim().is_empty() && !r.trim_start().starts_with('?') {
            return type_base(r);
        }
    }
    if t.starts_with('[') {
        let mut depth = 0i32;
        let mut close = None;
        for (i, c) in t.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close?;
        if close == t.len() - 1 {
            let inner = &t[1..close];
            if inner.contains(':') {
                return None;
            }
            let elem = match inner.rfind(';') {
                Some(s) => &inner[..s],
                None => inner,
            };
            return type_base(elem);
        }
        return type_base(&t[close + 1..]);
    }
    Some(t)
}

/// A type name Zig declares itself.
fn zig_type_name(n: &str) -> bool {
    matches!(
        n,
        "bool" | "void" | "type" | "anyerror" | "anyframe" | "anyopaque" | "noreturn" | "usize" | "isize"
            | "comptime_int" | "comptime_float" | "c_char" | "c_short" | "c_ushort" | "c_int" | "c_uint"
            | "c_long" | "c_ulong" | "c_longlong" | "c_ulonglong" | "c_longdouble"
    ) || (n.len() >= 2
        && (n.starts_with('u') || n.starts_with('i') || n.starts_with('f'))
        && n[1..].chars().all(|c| c.is_ascii_digit()))
}

/// t27c's Zig backend writes every `_cse*` temporary the optimizer made at
/// the very top of the fn body. One that reads a local of the body then
/// names it before its declaration: "use of undeclared identifier".
fn cse_hoist_defect(f: &Node) -> Option<String> {
    fn locals<'n>(ns: &'n [Node], out: &mut HashSet<&'n str>) {
        for n in ns {
            if n.kind == NodeKind::StmtLocal && !n.name.starts_with("_cse") && !n.name.is_empty() {
                out.insert(n.name.as_str());
            }
            locals(&n.children, out);
        }
    }
    fn first_ident<'n>(ns: &'n [Node], names: &HashSet<&str>) -> Option<&'n str> {
        ns.iter().find_map(|n| {
            if n.kind == NodeKind::ExprIdentifier && names.contains(n.name.as_str()) {
                Some(n.name.as_str())
            } else {
                first_ident(&n.children, names)
            }
        })
    }
    let mut ls = HashSet::new();
    locals(&f.children, &mut ls);
    for (p, _) in &f.params {
        ls.remove(p.as_str());
    }
    f.children
        .iter()
        .filter(|s| s.kind == NodeKind::StmtLocal && s.name.starts_with("_cse"))
        .find_map(|s| first_ident(&s.children, &ls))
        .map(str::to_string)
}

/// The reference's `collect_mutable_names`, exactly: assignment targets that
/// are a name, or whose immediate base (index or field) is one, looking into
/// `if`/`while`/`for` bodies and nowhere else.
fn ref_mutable_names(ns: &[Node], out: &mut HashSet<String>) {
    for n in ns {
        match n.kind {
            NodeKind::StmtAssign if !n.children.is_empty() => {
                let t = &n.children[0];
                if t.kind == NodeKind::ExprIdentifier {
                    out.insert(t.name.clone());
                }
                if matches!(t.kind, NodeKind::ExprIndex | NodeKind::ExprFieldAccess) {
                    if let Some(b) = t.children.first() {
                        if b.kind == NodeKind::ExprIdentifier {
                            out.insert(b.name.clone());
                        }
                    }
                }
            }
            NodeKind::StmtIf | NodeKind::StmtWhile | NodeKind::StmtFor | NodeKind::StmtForRange => {
                for c in &n.children {
                    if c.kind == NodeKind::Module {
                        ref_mutable_names(&c.children, out);
                    } else {
                        ref_mutable_names(std::slice::from_ref(c), out);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Does Zig count `var name` as mutated anywhere in `ns`? An assignment whose
/// target reaches `name` through fields and indexes but no `.*`, `&` of such
/// a path, or a method call on it. Errs toward yes: a yes refuses nothing.
fn zig_mutates(ns: &[Node], name: &str) -> bool {
    fn rooted(t: &Node, name: &str) -> bool {
        match t.kind {
            NodeKind::ExprIdentifier => t.name == name,
            NodeKind::ExprFieldAccess if t.name == "*" => false,
            NodeKind::ExprFieldAccess | NodeKind::ExprIndex => t.children.first().is_some_and(|b| rooted(b, name)),
            _ => false,
        }
    }
    ns.iter().any(|n| {
        let hit = match n.kind {
            NodeKind::StmtAssign => n.children.first().is_some_and(|t| rooted(t, name)),
            NodeKind::ExprUnary if n.extra_op == "&" => n.children.first().is_some_and(|t| rooted(t, name)),
            NodeKind::ExprCall => n.name.split('.').next() == Some(name) && n.name.contains('.'),
            _ => false,
        };
        hit || zig_mutates(&n.children, name)
    })
}

/// Syntax the reference prints that Zig cannot parse: a childless typed
/// array literal (`[_]T{}`), whose dimension t27c prints as its element.
/// (A field named for a Zig keyword is not one: since #6451 t27c's
/// `zig_ident` writes it `@"align"` in the literal and the access too.)
fn zig_syntax_defects(ns: &[Node], line: u32, found: &mut Vec<(u32, &'static str, String)>) {
    for n in ns {
        // Expressions carry no line; report the enclosing statement's.
        let at = if n.line != 0 { n.line } else { line };
        match n.kind {
            NodeKind::ExprArrayLiteral
                if n.children.is_empty() && !n.extra_type.trim().is_empty() && !n.extra_size.trim().is_empty() =>
            {
                found.push((
                    at,
                    "ExprArrayLiteral(reference empty typed)",
                    format!(
                        "`[{}]{}{{}}`: t27c's Zig backend prints the dimension `{}` as the literal's only element",
                        n.extra_size.trim(),
                        n.extra_type.trim(),
                        n.extra_size.trim()
                    ),
                ));
            }
            _ => {}
        }
        zig_syntax_defects(&n.children, at, found);
    }
}

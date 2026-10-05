//! `t27c gen-ts --fn`: an opt-in lowering of PURE fn bodies to TypeScript.
//!
//! #4471 drew the line that `gen-js` and `gen-ts` print declarations, never
//! behaviour, and #6101 asked for the separate, opt-in piece of work that line
//! named. This is a first slice of it for `gen-ts` (#6559): the subset a spec
//! of laws needs -- `fn law(a: bool, n: u32) -> bool { return ...; }` -- and
//! nothing more. Without `--fn` none of this runs and the artifact is what it
//! was.
//!
//! What lowers, exactly:
//!
//! - parameters and a return type of `bool`, `str`, or an integer of at most 32
//!   bits (`u8` `u16` `u32` `i8` `i16` `i32`). A 64-bit integer does not fit a
//!   JavaScript number, so a fn that names one is refused;
//! - a body of zero or more `if <cond> { return <expr>; }` followed by one final
//!   `return <expr>;`;
//! - expressions over parameters, the module's emitted consts, literals, calls
//!   to other lowered fns of the same module, `&&` `||` `!`, comparisons, and
//!   integer `+ - * / %`. Strings take part in `==` and `!=` only.
//!
//! Everything else -- a loop, a local, an assignment, a call to anything that
//! is not a lowered fn here, a float, an array -- refuses the whole fn, and the
//! refusal is announced in the artifact and listed in `__NOT_EMITTED__`. A fn
//! that calls a refused fn is refused in turn: half a law is not a law.
//!
//! **Integer width is kept by refusing to leave it, not by wrapping.** #6101
//! leaves "mask or refuse" to the owners, and both of those are compile-time
//! answers this slice does not need to pick. A JavaScript number holds every
//! u32/i32 result of `+ - *` exactly, so each integer operation is checked at
//! run time against its declared width and THROWS a RangeError when it leaves
//! it -- the same thing a safety-checked Zig build does. A wrong number never
//! comes back; a wrapped one would be wrong in silence. `/` truncates, as the
//! const layer already does for t27 integer division, and a division by zero
//! throws through the same check.
//!
//! Types are checked here as well, not left to `tsc`: an argument of the wrong
//! type, a comparison between a `u8` and a `u32`, or a literal outside its
//! declared range refuses the fn. Every artifact this prints type-checks
//! against itself.

use crate::codegen_js::{int_width, js_name, js_string, Target, TS};
use crate::compiler::{Node, NodeKind};
use std::collections::{BTreeMap, BTreeSet};

/// The run-time width check, printed once into any artifact whose lowered fns
/// do integer arithmetic. Not exported: it is how the artifact keeps the spec's
/// widths, not something the spec declared.
pub(crate) const INT_HELPER: &str = "\n// t27c gen-ts: integer arithmetic keeps the width the spec declared. A result\n\
// outside it throws rather than wrapping or growing -- a lowered law never\n\
// hands back a number its own spec says cannot exist.\n\
function __t27_int(v: number, lo: number, hi: number, ty: string): number {\n\
  if (!Number.isInteger(v) || v < lo || v > hi) {\n\
    throw new RangeError(\"t27: \" + v + \" is outside \" + ty);\n\
  }\n\
  return v;\n\
}\n";

/// The type of a lowered expression.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Ty {
    Bool,
    Str,
    /// A declared integer: width in bits, signed.
    Int(u32, bool),
    /// An integer literal with no type of its own yet; it takes the type of
    /// what it meets, and its value is checked against that type's range.
    Lit(i128),
}

impl Ty {
    fn ts(self) -> &'static str {
        match self {
            Ty::Bool => "boolean",
            Ty::Str => "string",
            Ty::Int(..) | Ty::Lit(_) => "number",
        }
    }
}

fn spelled(ty: Ty) -> String {
    match ty {
        Ty::Bool => "bool".into(),
        Ty::Str => "str".into(),
        Ty::Int(bits, true) => format!("i{bits}"),
        Ty::Int(bits, false) => format!("u{bits}"),
        Ty::Lit(v) => format!("the integer literal {v}"),
    }
}

fn range(bits: u32, signed: bool) -> (i128, i128) {
    if signed {
        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
    } else {
        (0, (1i128 << bits) - 1)
    }
}

/// A declared type this slice can carry, or why not.
fn declared(decl: &str) -> Result<Ty, String> {
    let decl = decl.trim();
    match decl {
        "bool" => return Ok(Ty::Bool),
        "str" => return Ok(Ty::Str),
        "" => return Err("it has no declared type".into()),
        _ => {}
    }
    match int_width(decl) {
        Some((bits, signed)) if bits <= 32 => Ok(Ty::Int(bits, signed)),
        Some(_) => Err(format!(
            "{decl} is wider than a JavaScript number holds exactly, and a law that rounds is not the law"
        )),
        None => Err(format!("{decl} is outside the pure subset (bool, str, integers up to 32 bits)")),
    }
}

/// An integer literal's value: decimal, `0x`, `0b`, `0o`, with `_` separators.
fn int_literal(s: &str) -> Option<i128> {
    let (neg, s) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let lower = s.to_ascii_lowercase().replace('_', "");
    let (digits, radix) = if let Some(d) = lower.strip_prefix("0x") {
        (d.to_string(), 16)
    } else if let Some(d) = lower.strip_prefix("0b") {
        (d.to_string(), 2)
    } else if let Some(d) = lower.strip_prefix("0o") {
        (d.to_string(), 8)
    } else {
        (lower, 10)
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    let v = i128::from_str_radix(&digits, radix).ok()?;
    Some(if neg { -v } else { v })
}

fn lit_text(v: i128) -> String {
    if v < 0 {
        format!("({v})")
    } else {
        v.to_string()
    }
}

/// A fn's signature, once its declared types are known to be carried.
struct Sig {
    params: Vec<(String, Ty)>,
    ret: Ty,
}

/// What one body may read.
struct Scope<'a> {
    params: &'a [(String, Ty)],
    consts: &'a BTreeMap<String, Ty>,
    fns: &'a BTreeMap<String, Sig>,
    uses_int: bool,
    /// The nearest line the parser recorded. Expression nodes carry none,
    /// so a refusal inside one names the line of its return or its fn.
    line: u32,
}

/// The line to name for `node`: its own if the parser kept one.
fn line_of(node: &Node, scope: &Scope<'_>) -> u32 {
    if node.line > 0 {
        node.line
    } else {
        scope.line
    }
}

/// Can a value of type `have` stand where `want` is declared?
fn fits(have: Ty, want: Ty) -> bool {
    match (have, want) {
        (Ty::Lit(v), Ty::Int(bits, signed)) => {
            let (lo, hi) = range(bits, signed);
            v >= lo && v <= hi
        }
        (a, b) => a == b,
    }
}

/// The one type two integer operands share, or why they share none.
fn unify_int(a: Ty, b: Ty, op: &str, line: u32) -> Result<Ty, String> {
    match (a, b) {
        (Ty::Int(..), Ty::Int(..)) if a == b => Ok(a),
        (Ty::Int(..), Ty::Lit(_)) if fits(b, a) => Ok(a),
        (Ty::Lit(_), Ty::Int(..)) if fits(a, b) => Ok(b),
        (Ty::Lit(_), Ty::Lit(_)) => Ok(a),
        _ => Err(format!(
            "{:?} at line {} joins {} and {}, which this slice does not convert between",
            op,
            line,
            spelled(a),
            spelled(b)
        )),
    }
}

/// Drop one pair of parentheses that encloses the whole text, so `if` and
/// `return` do not print `((...))`. Strings are skipped while matching.
fn bare(s: &str) -> &str {
    let b = s.as_bytes();
    if b.first() != Some(&b'(') || b.last() != Some(&b')') {
        return s;
    }
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 1;
            } else if c == b'"' {
                in_str = false;
            }
        } else {
            match c {
                b'"' => in_str = true,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 && i != b.len() - 1 {
                        return s;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    &s[1..s.len() - 1]
}

fn checked(scope: &mut Scope<'_>, text: String, ty: Ty) -> String {
    match ty {
        Ty::Int(bits, signed) => {
            scope.uses_int = true;
            let (lo, hi) = range(bits, signed);
            format!("__t27_int({}, {}, {}, {})", bare(&text), lo, hi, js_string(&spelled(ty)))
        }
        _ => text,
    }
}

/// The largest integer a JavaScript number holds exactly (2^53 - 1).
const MAX_SAFE: i128 = 9_007_199_254_740_991;

/// An integer literal, or a value folded from literals, must survive the trip
/// into a JavaScript number unchanged; beyond 2^53 - 1 two distinct values
/// can print as one and compare equal.
fn exact(v: i128, line: u32) -> Result<i128, String> {
    if v.abs() > MAX_SAFE {
        return Err(format!(
            "the integer {} at line {} is beyond 2^53 - 1, which a JavaScript number does not hold exactly",
            v, line
        ));
    }
    Ok(v)
}

fn outside(node: &Node, what: &str) -> String {
    format!("{} at line {} is outside the pure subset this lowering takes", what, node.line)
}

fn lower_expr(node: &Node, scope: &mut Scope<'_>) -> Result<(String, Ty), String> {
    match node.kind {
        NodeKind::ExprLiteral => {
            if node.extra_kind == "string" {
                return Ok((js_string(&node.value), Ty::Str));
            }
            match node.value.as_str() {
                "true" => return Ok(("true".into(), Ty::Bool)),
                "false" => return Ok(("false".into(), Ty::Bool)),
                _ => {}
            }
            match int_literal(&node.value) {
                Some(v) => exact(v, line_of(node, scope)).map(|v| (lit_text(v), Ty::Lit(v))),
                None => Err(outside(node, &format!("the literal {:?}", node.value))),
            }
        }
        NodeKind::ExprIdentifier => {
            let name = node.name.as_str();
            match name {
                "true" => return Ok(("true".into(), Ty::Bool)),
                "false" => return Ok(("false".into(), Ty::Bool)),
                _ => {}
            }
            if let Some((_, ty)) = scope.params.iter().find(|(p, _)| p == name) {
                return Ok((name.to_string(), *ty));
            }
            if let Some(ty) = scope.consts.get(name) {
                return Ok((name.to_string(), *ty));
            }
            Err(format!(
                "{:?} at line {} is neither a parameter nor a const this artifact exports with a type the lowering carries",
                name, node.line
            ))
        }
        NodeKind::ExprUnary => {
            let child = node
                .children
                .first()
                .ok_or_else(|| outside(node, "an operator with no operand"))?;
            let (v, ty) = lower_expr(child, scope)?;
            match (node.extra_op.trim(), ty) {
                ("!", Ty::Bool) => Ok((format!("(!{})", v), Ty::Bool)),
                ("-", Ty::Lit(n)) => exact(-n, line_of(node, scope)).map(|n| (lit_text(n), Ty::Lit(n))),
                ("-", Ty::Int(_, true)) => {
                    let text = format!("(-{})", v);
                    Ok((checked(scope, text, ty), ty))
                }
                (op, ty) => Err(format!(
                    "the unary {:?} at line {} on {} is outside the pure subset this lowering takes",
                    op,
                    node.line,
                    spelled(ty)
                )),
            }
        }
        NodeKind::ExprBinary => {
            let (left, right) = match (node.children.first(), node.children.get(1)) {
                (Some(l), Some(r)) => (l, r),
                _ => return Err(outside(node, "an operator missing an operand")),
            };
            let op = node.extra_op.trim();
            let (a, ta) = lower_expr(left, scope)?;
            let (b, tb) = lower_expr(right, scope)?;
            match op {
                "and" | "&&" | "or" | "||" => {
                    if ta != Ty::Bool || tb != Ty::Bool {
                        return Err(format!(
                            "{:?} at line {} joins {} and {}, and takes two bools",
                            op,
                            node.line,
                            spelled(ta),
                            spelled(tb)
                        ));
                    }
                    let js = if matches!(op, "and" | "&&") { "&&" } else { "||" };
                    Ok((format!("({} {} {})", a, js, b), Ty::Bool))
                }
                "==" | "!=" => {
                    let same = match (ta, tb) {
                        (Ty::Bool, Ty::Bool) | (Ty::Str, Ty::Str) => true,
                        _ => unify_int(ta, tb, op, node.line).is_ok(),
                    };
                    if !same {
                        return Err(format!(
                            "{:?} at line {} compares {} with {}",
                            op,
                            node.line,
                            spelled(ta),
                            spelled(tb)
                        ));
                    }
                    let js = if op == "==" { "===" } else { "!==" };
                    Ok((format!("({} {} {})", a, js, b), Ty::Bool))
                }
                "<" | "<=" | ">" | ">=" => {
                    unify_int(ta, tb, op, node.line)?;
                    Ok((format!("({} {} {})", a, op, b), Ty::Bool))
                }
                "+" | "-" | "*" | "/" | "%" => {
                    let ty = unify_int(ta, tb, op, node.line)?;
                    if let (Ty::Lit(x), Ty::Lit(y)) = (ta, tb) {
                        // Two literals fold here, exactly, so no run-time check
                        // has a type to check against and none is needed.
                        let folded = match op {
                            "+" => x.checked_add(y),
                            "-" => x.checked_sub(y),
                            "*" => x.checked_mul(y),
                            "/" => x.checked_div(y),
                            _ => x.checked_rem(y),
                        }
                        .ok_or_else(|| {
                            format!("{:?} at line {} has no value ({} {} {})", op, node.line, x, op, y)
                        })?;
                        let folded = exact(folded, line_of(node, scope))?;
                        return Ok((lit_text(folded), Ty::Lit(folded)));
                    }
                    let text = if op == "/" {
                        format!("Math.trunc({} / {})", a, b)
                    } else {
                        format!("({} {} {})", a, op, b)
                    };
                    Ok((checked(scope, text, ty), ty))
                }
                other => Err(format!(
                    "the operator {:?} at line {} is outside the pure subset this lowering takes",
                    other, node.line
                )),
            }
        }
        NodeKind::ExprCall => {
            if !node.extra_op.is_empty() {
                return Err(outside(node, "a call through an expression"));
            }
            let sig = scope.fns.get(&node.name).ok_or_else(|| {
                format!(
                    "the call to {:?} at line {} names no lowered fn of this module",
                    node.name, node.line
                )
            })?;
            if sig.params.len() != node.children.len() {
                return Err(format!(
                    "the call to {:?} at line {} passes {} argument(s) to {} parameter(s)",
                    node.name,
                    node.line,
                    node.children.len(),
                    sig.params.len()
                ));
            }
            let wants: Vec<Ty> = sig.params.iter().map(|(_, t)| *t).collect();
            let ret = sig.ret;
            let mut args = Vec::new();
            for (arg, want) in node.children.iter().zip(wants) {
                let (v, have) = lower_expr(arg, scope)?;
                if !fits(have, want) {
                    return Err(format!(
                        "the call to {:?} at line {} passes {} where {} is declared",
                        node.name,
                        node.line,
                        spelled(have),
                        spelled(want)
                    ));
                }
                args.push(bare(&v).to_string());
            }
            Ok((format!("{}({})", node.name, args.join(", ")), ret))
        }
        _ => Err(outside(node, &format!("a {:?}", node.kind))),
    }
}

/// `return <expr>;`, checked against the declared return type.
fn lower_return(node: &Node, ret: Ty, scope: &mut Scope<'_>) -> Result<String, String> {
    if node.kind != NodeKind::ExprReturn {
        return Err(outside(node, &format!("a {:?} where a return belongs", node.kind)));
    }
    let value = node
        .children
        .first()
        .ok_or_else(|| outside(node, "a return with no value"))?;
    if node.line > 0 {
        scope.line = node.line;
    }
    let (v, ty) = lower_expr(value, scope)?;
    if !fits(ty, ret) {
        return Err(format!(
            "the return at line {} gives {} where {} is declared",
            node.line,
            spelled(ty),
            spelled(ret)
        ));
    }
    Ok(format!("return {};", bare(&v)))
}

fn lower_body(f: &Node, sig: &Sig, scope: &mut Scope<'_>) -> Result<String, String> {
    let Some((last, guards)) = f.children.split_last() else {
        return Err(format!("fn {} at line {} has an empty body", f.name, f.line));
    };
    let mut body = String::new();
    scope.line = f.line;
    for stmt in guards {
        // `if <cond> { return <expr>; }`, nothing else: no else, no block of
        // statements, no early return without a condition.
        let shape_ok = stmt.kind == NodeKind::StmtIf
            && stmt.children.len() == 2
            && stmt.children[1].kind == NodeKind::Module
            && stmt.children[1].name == "then"
            && stmt.children[1].children.len() == 1;
        if !shape_ok {
            return Err(outside(
                stmt,
                &format!("a {:?} that is not `if <cond> {{ return <expr>; }}`", stmt.kind),
            ));
        }
        // The guard's own line, else its return's: the `if` node may carry none.
        let then_line = stmt.children[1].children[0].line;
        scope.line = [stmt.line, then_line, f.line].into_iter().find(|l| *l > 0).unwrap_or(0);
        let (cond, ty) = lower_expr(&stmt.children[0], scope)?;
        if ty != Ty::Bool {
            return Err(format!(
                "the condition at line {} is {}, not bool",
                line_of(&stmt.children[0], scope),
                spelled(ty)
            ));
        }
        let ret = lower_return(&stmt.children[1].children[0], sig.ret, scope)?;
        body.push_str(&format!("  if ({}) {{\n    {}\n  }}\n", bare(&cond), ret));
    }
    body.push_str(&format!("  {}\n", lower_return(last, sig.ret, scope)?));
    let params: Vec<String> = sig.params.iter().map(|(n, t)| format!("{}: {}", n, t.ts())).collect();
    Ok(format!(
        "export function {}({}): {} {{\n{}}}\n",
        f.name,
        params.join(", "),
        sig.ret.ts(),
        body
    ))
}

/// The signature of one fn, or why it cannot be carried at all.
/// The globals the lowered code reads: `Math.trunc` for integer division,
/// `Number.isInteger` and `RangeError` inside `__t27_int`. A name in the
/// module or in a fn that hides one of them makes the emitted code call the
/// spec's value instead, so the lowering refuses it.
pub(crate) const RELIED_GLOBALS: &[&str] = &["Math", "Number", "RangeError"];

/// The names every gen-ts artifact exports itself. A fn, a parameter or a
/// module declaration with one of them is a second binding of the same name:
/// `SyntaxError: Identifier has already been declared`.
pub(crate) const ARTIFACT_NAMES: &[&str] = &["__NOT_EMITTED__", "__DECL_ORDER__", "__STRUCT_ORDER__"];

/// Names a strict-mode module refuses as a binding. `eval` and `arguments`
/// cannot be bound at all; the rest are reserved words only in strict code,
/// which an ES module always is.
const STRICT_ONLY: &[&str] = &[
    "eval", "arguments", "implements", "interface", "package", "private", "protected", "public",
];

/// Why `name` cannot be bound in lowered code, if it cannot.
fn reserved_reason(name: &str) -> Option<String> {
    if name.starts_with("__t27") {
        return Some(format!("{:?} is reserved for the artifact's own helpers (prefix __t27)", name));
    }
    if ARTIFACT_NAMES.contains(&name) {
        return Some(format!("{:?} is a name every gen-ts artifact already exports", name));
    }
    if RELIED_GLOBALS.contains(&name) {
        return Some(format!(
            "{:?} would hide the JavaScript global the lowered code relies on",
            name
        ));
    }
    if STRICT_ONLY.contains(&name) {
        return Some(format!("{:?} cannot be bound in a strict-mode module", name));
    }
    None
}

fn signature(f: &Node, module_names: &BTreeSet<String>) -> Result<Sig, String> {
    let t: &Target = &TS;
    js_name(t, &f.name)?;
    if let Some(why) = reserved_reason(&f.name) {
        return Err(format!("the fn name: {}", why));
    }
    let mut params = Vec::new();
    let mut seen = BTreeSet::new();
    for (name, ty) in &f.params {
        js_name(t, name)?;
        if let Some(why) = reserved_reason(name) {
            return Err(format!("the parameter {}", why));
        }
        if !seen.insert(name.clone()) {
            return Err(format!("the parameter {:?} is declared twice", name));
        }
        if module_names.contains(name) {
            return Err(format!(
                "the parameter {:?} has the name of a module declaration, and would hide it inside the body",
                name
            ));
        }
        let ty = declared(ty).map_err(|why| format!("the parameter {:?}: {}", name, why))?;
        params.push((name.clone(), ty));
    }
    let ret = declared(&f.extra_return_type).map_err(|why| format!("the return type: {}", why))?;
    Ok(Sig { params, ret })
}

/// The outcome of lowering a module's fns.
pub(crate) struct Lowered {
    /// Per fn node index in `ast.children`: the TypeScript text, or the reason
    /// it was not lowered.
    pub fns: BTreeMap<usize, Result<String, String>>,
    /// Whether any lowered fn does integer arithmetic, so `INT_HELPER` is due.
    pub uses_int: bool,
}

/// Lower every fn in the module. `emitted_consts` are the consts the artifact
/// actually exports -- a body may read those and no others.
pub(crate) fn lower_module(ast: &Node, emitted_consts: &BTreeSet<String>) -> Lowered {
    let mut consts: BTreeMap<String, Ty> = BTreeMap::new();
    let mut module_names: BTreeSet<String> = BTreeSet::new();
    for n in &ast.children {
        if matches!(
            n.kind,
            NodeKind::ConstDecl | NodeKind::EnumDecl | NodeKind::StructDecl | NodeKind::FnDecl
        ) && !n.name.is_empty()
        {
            module_names.insert(n.name.clone());
        }
        if n.kind == NodeKind::ConstDecl && emitted_consts.contains(&n.name) {
            if let Ok(ty) = declared(&n.extra_type) {
                consts.insert(n.name.clone(), ty);
            }
        }
    }

    let fn_nodes: Vec<(usize, &Node)> = ast
        .children
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == NodeKind::FnDecl)
        .collect();
    let mut fn_count: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, f) in &fn_nodes {
        *fn_count.entry(f.name.as_str()).or_default() += 1;
    }

    let mut fns: BTreeMap<usize, Result<String, String>> = BTreeMap::new();
    // A module-level name that hides a global the lowered code relies on (or
    // a helper of its own) breaks every lowered fn in the module, not only
    // the one that reads it: a module binding shadows the global everywhere.
    let shadowing: Vec<String> = module_names
        .iter()
        .filter(|name| {
            name.starts_with("__t27")
                || RELIED_GLOBALS.contains(&name.as_str())
                || ARTIFACT_NAMES.contains(&name.as_str())
        })
        .cloned()
        .collect();
    if !shadowing.is_empty() {
        let why = format!(
            "the module declares {}, which would hide or redeclare what the lowered code relies on (Math, Number, RangeError, __t27*, __NOT_EMITTED__, __DECL_ORDER__, __STRUCT_ORDER__)",
            shadowing.iter().map(|n| format!("{:?}", n)).collect::<Vec<_>>().join(", ")
        );
        for (i, _) in &fn_nodes {
            fns.insert(*i, Err(why.clone()));
        }
        return Lowered { fns, uses_int: false };
    }
    let mut sigs: BTreeMap<String, Sig> = BTreeMap::new();
    let mut candidates: Vec<(usize, &Node)> = Vec::new();
    for (i, f) in &fn_nodes {
        let clash = fn_count[f.name.as_str()] > 1
            || ast.children.iter().any(|n| {
                n.kind != NodeKind::FnDecl && !n.name.is_empty() && n.name == f.name
            });
        if clash {
            fns.insert(*i, Err(format!("the name {:?} is declared more than once in this module", f.name)));
            continue;
        }
        match signature(f, &module_names) {
            Ok(sig) => {
                sigs.insert(f.name.clone(), sig);
                candidates.push((*i, f));
            }
            Err(why) => {
                fns.insert(*i, Err(why));
            }
        }
    }

    // A fn is lowered only if every fn it calls is. Removing one can strand a
    // caller, so this runs until nothing changes.
    loop {
        let mut dropped = Vec::new();
        for (i, f) in &candidates {
            let sig = &sigs[&f.name];
            let mut scope = Scope { params: &sig.params, consts: &consts, fns: &sigs, uses_int: false, line: 0 };
            if let Err(why) = lower_body(f, sig, &mut scope) {
                dropped.push((*i, f.name.clone(), why));
            }
        }
        if dropped.is_empty() {
            break;
        }
        for (i, name, why) in dropped {
            sigs.remove(&name);
            candidates.retain(|(j, _)| *j != i);
            fns.insert(i, Err(why));
        }
    }

    let mut uses_int = false;
    for (i, f) in &candidates {
        let sig = &sigs[&f.name];
        let mut scope = Scope { params: &sig.params, consts: &consts, fns: &sigs, uses_int: false, line: 0 };
        let text = lower_body(f, sig, &mut scope);
        uses_int |= scope.uses_int;
        fns.insert(*i, text);
    }
    Lowered { fns, uses_int }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(kind: NodeKind) -> Node {
        Node { kind, ..Default::default() }
    }
    fn id(name: &str) -> Node {
        Node { kind: NodeKind::ExprIdentifier, name: name.into(), ..Default::default() }
    }
    fn lit(v: &str) -> Node {
        Node { kind: NodeKind::ExprLiteral, value: v.into(), ..Default::default() }
    }
    fn bin(op: &str, a: Node, b: Node) -> Node {
        Node { kind: NodeKind::ExprBinary, extra_op: op.into(), children: vec![a, b], ..Default::default() }
    }
    fn not(a: Node) -> Node {
        Node { kind: NodeKind::ExprUnary, extra_op: "!".into(), children: vec![a], ..Default::default() }
    }
    fn ret(e: Node) -> Node {
        Node { kind: NodeKind::ExprReturn, children: vec![e], ..Default::default() }
    }
    fn func(name: &str, params: &[(&str, &str)], rty: &str, body: Vec<Node>) -> Node {
        Node {
            kind: NodeKind::FnDecl,
            name: name.into(),
            extra_return_type: rty.into(),
            params: params.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
            children: body,
            ..Default::default()
        }
    }
    fn module(children: Vec<Node>) -> Node {
        Node { kind: NodeKind::Module, children, ..Default::default() }
    }
    fn only(ast: &Node, consts: &[&str]) -> Result<String, String> {
        let set = consts.iter().map(|s| s.to_string()).collect();
        lower_module(ast, &set).fns.into_values().next().unwrap()
    }

    #[test]
    fn and_or_not_lower_to_their_javascript_operators() {
        let body = bin("or", id("a"), bin("and", not(id("b")), id("c")));
        let ast = module(vec![func("f", &[("a", "bool"), ("b", "bool"), ("c", "bool")], "bool", vec![ret(body)])]);
        assert_eq!(
            only(&ast, &[]).unwrap(),
            "export function f(a: boolean, b: boolean, c: boolean): boolean {\n  return a || ((!b) && c);\n}\n"
        );
    }

    #[test]
    fn a_guard_chain_lowers_to_ifs_and_a_final_return() {
        let then = Node { kind: NodeKind::Module, name: "then".into(), children: vec![ret(lit("2"))], ..Default::default() };
        let guard = Node { kind: NodeKind::StmtIf, children: vec![id("t"), then], ..Default::default() };
        let ast = module(vec![func("pick", &[("t", "bool")], "u8", vec![guard, ret(lit("0"))])]);
        assert_eq!(
            only(&ast, &[]).unwrap(),
            "export function pick(t: boolean): number {\n  if (t) {\n    return 2;\n  }\n  return 0;\n}\n"
        );
    }

    #[test]
    fn equality_is_strict_and_a_const_is_read_by_name() {
        let mut c = n(NodeKind::ConstDecl);
        c.name = "LIMIT".into();
        c.extra_type = "u32".into();
        let ast = module(vec![c, func("hit", &[("x", "u32")], "bool", vec![ret(bin("==", id("x"), id("LIMIT")))])]);
        assert!(only(&ast, &["LIMIT"]).unwrap().contains("return x === LIMIT;"));
        // A const the artifact did not export cannot be read.
        assert!(only(&ast, &[]).is_err());
    }

    #[test]
    fn integer_arithmetic_is_checked_against_its_width() {
        let ast = module(vec![func("add", &[("a", "u8"), ("b", "u8")], "u8", vec![ret(bin("+", id("a"), id("b")))])]);
        let lowered = lower_module(&ast, &BTreeSet::new());
        assert!(lowered.uses_int);
        assert_eq!(
            lowered.fns[&0].as_ref().unwrap(),
            "export function add(a: number, b: number): number {\n  return __t27_int(a + b, 0, 255, \"u8\");\n}\n"
        );
    }

    #[test]
    fn integer_division_truncates() {
        let ast = module(vec![func("half", &[("a", "i32")], "i32", vec![ret(bin("/", id("a"), lit("2")))])]);
        assert!(only(&ast, &[]).unwrap().contains("__t27_int(Math.trunc(a / 2), -2147483648, 2147483647, \"i32\")"));
    }

    #[test]
    fn a_loop_refuses_the_fn_and_its_callers() {
        let lp = Node { kind: NodeKind::StmtWhile, line: 7, ..Default::default() };
        let callee = func("spin", &[], "bool", vec![lp, ret(lit("true"))]);
        let call = Node { kind: NodeKind::ExprCall, name: "spin".into(), ..Default::default() };
        let caller = func("uses", &[], "bool", vec![ret(call)]);
        let lowered = lower_module(&module(vec![callee, caller]), &BTreeSet::new());
        assert!(lowered.fns[&0].as_ref().unwrap_err().contains("StmtWhile"));
        assert!(lowered.fns[&1].as_ref().unwrap_err().contains("names no lowered fn"));
    }

    #[test]
    fn wide_and_mismatched_types_are_refused() {
        let wide = module(vec![func("w", &[("a", "u64")], "bool", vec![ret(lit("true"))])]);
        assert!(only(&wide, &[]).unwrap_err().contains("wider than"));
        let mixed = module(vec![func("m", &[("a", "u8"), ("b", "u32")], "bool", vec![ret(bin("<", id("a"), id("b")))])]);
        assert!(only(&mixed, &[]).unwrap_err().contains("does not convert"));
        let range = module(vec![func("r", &[], "u8", vec![ret(lit("256"))])]);
        assert!(only(&range, &[]).unwrap_err().contains("where u8 is declared"));
        let ordered = module(vec![func("s", &[("a", "str"), ("b", "str")], "bool", vec![ret(bin("<", id("a"), id("b")))])]);
        assert!(only(&ordered, &[]).is_err(), "strings take part in == and != only");
    }

    #[test]
    fn bare_drops_only_an_enclosing_pair() {
        assert_eq!(bare("(a && b)"), "a && b");
        assert_eq!(bare("(a) && (b)"), "(a) && (b)");
        assert_eq!(bare("(\")\" === s)"), "\")\" === s");
    }

    #[test]
    fn a_param_with_a_module_declarations_name_is_refused() {
        let k = Node { kind: NodeKind::ConstDecl, name: "k".into(), extra_type: "u8".into(), ..Default::default() };
        let ast = module(vec![k, func("f", &[("k", "u8")], "u8", vec![ret(id("k"))])]);
        let lowered = lower_module(&ast, &["k".to_string()].into_iter().collect());
        let why = lowered.fns.into_values().next().unwrap().unwrap_err();
        assert!(why.contains("has the name of a module declaration"), "{why}");
    }

    #[test]
    fn a_name_that_hides_a_relied_global_or_a_helper_is_refused() {
        for name in [
            "Math", "Number", "RangeError", "__t27_int", "eval", "arguments", "interface",
            "__NOT_EMITTED__", "__DECL_ORDER__", "__STRUCT_ORDER__",
        ] {
            let as_param = module(vec![func("f", &[(name, "i32")], "i32", vec![ret(lit("1"))])]);
            let why = only(&as_param, &[]).unwrap_err();
            assert!(why.starts_with("the parameter") && why.contains(name), "{name}: {why}");
            let as_fn = module(vec![func(name, &[], "bool", vec![ret(lit("true"))])]);
            // A fn is a module declaration too: the hiding names are caught
            // for the whole module, the strict-only ones by the fn's own name.
            let want = if STRICT_ONLY.contains(&name) { "the fn name" } else { "the module declares" };
            let why = only(&as_fn, &[]).unwrap_err();
            assert!(why.contains(want) && why.contains(name), "{name}: {why}");
        }
    }

    #[test]
    fn a_module_name_that_shadows_a_global_refuses_every_fn() {
        for (kind, name) in [
            (NodeKind::ConstDecl, "Number"),
            (NodeKind::StructDecl, "RangeError"),
            (NodeKind::EnumDecl, "Math"),
            (NodeKind::ConstDecl, "__t27_int"),
            (NodeKind::FnDecl, "Math"),
            (NodeKind::ConstDecl, "__NOT_EMITTED__"),
            (NodeKind::StructDecl, "__STRUCT_ORDER__"),
            (NodeKind::FnDecl, "__DECL_ORDER__"),
        ] {
            let decl = Node { kind, name: name.into(), extra_type: "u8".into(), ..Default::default() };
            let add = func("add", &[("a", "u8"), ("b", "u8")], "u8", vec![ret(bin("+", id("a"), id("b")))]);
            let ast = module(vec![decl, add]);
            let lowered = lower_module(&ast, &BTreeSet::new());
            assert!(lowered.fns.values().all(|r| r.is_err()), "{name}");
            let why = lowered.fns.values().last().unwrap().clone().unwrap_err();
            assert!(why.contains("the module declares") && why.contains(name), "{name}: {why}");
        }
    }

    #[test]
    fn an_integer_beyond_two_to_the_53_is_refused() {
        let big = module(vec![func("b", &[], "bool", vec![ret(bin("==", lit("9007199254740993"), lit("9007199254740992")))])]);
        assert!(only(&big, &[]).unwrap_err().contains("beyond 2^53 - 1"));
        // The literal carries no line of its own; the refusal names its return's.
        let mut r = ret(lit("9007199254740993"));
        r.line = 4;
        let lined = module(vec![func("l", &[], "bool", vec![r])]);
        assert!(only(&lined, &[]).unwrap_err().contains("at line 4 is beyond"));
        let folded = module(vec![func("c", &[], "bool", vec![ret(bin("==", bin("*", lit("9007199254740991"), lit("2")), lit("0")))])]);
        assert!(only(&folded, &[]).unwrap_err().contains("beyond 2^53 - 1"));
        let edge = module(vec![func("d", &[], "bool", vec![ret(bin("==", lit("9007199254740991"), lit("9007199254740991")))])]);
        assert!(only(&edge, &[]).is_ok());
    }
}

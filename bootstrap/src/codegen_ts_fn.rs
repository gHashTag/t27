//! Function bodies for `t27c gen-ts`: the pure subset, lowered; the rest,
//! announced.
//!
//! Why this exists (owner, 2026-10-08: "the functionality 100% from .t27").
//! Until now `gen-ts` lowered declarations only, so every host that used a
//! spec's rule wrote the rule a second time by hand and pinned the copy with
//! the spec's test vectors. The copy is exactly what a spec is meant to
//! remove: two homes for one rule. A spec's `fn` is the rule; a host should
//! import it.
//!
//! **The subset is chosen so that nothing runs at import and every value means
//! what it means in t27.** A function definition does not run when its module
//! loads, so the promise of the module -- importing it executes nothing -- still
//! holds. Inside, only shapes whose TypeScript meaning is the t27 meaning:
//!
//! - parameters and results of type `bool`, an integer, or `str`;
//! - `return`, locals, assignments (`=`, `+=`, `-=`, `*=`), `if`/`else`,
//!   `while`, `for i in a..b`, `break`, `continue`;
//! - literals, the parameters and locals, the module's constants and the
//!   module's own functions, indexing, `!`, unary `-`, `and`/`or`, comparisons
//!   (`==` as `===`), `+ - * %`, and integer `/` as `Math.trunc(a / b)`.
//!
//! Anything else -- floats, structs, pointers, a call into another module, a
//! name nobody declared, Zig's wrapping operators -- refuses the whole function
//! with a sentence, and `gen-ts` prints that sentence where the function would
//! have been. A function half lowered would be a rule that is wrong in the
//! cases nobody tested, which is worse than a rule that is visibly absent.

use crate::codegen_js::{is_number, js_name, js_string, TS};
use crate::compiler::{Node, NodeKind};
use std::collections::BTreeSet;

/// What a body may name besides its own parameters and locals.
pub(crate) struct Names<'a> {
    pub consts: &'a BTreeSet<String>,
    pub fns: &'a BTreeSet<String>,
    /// The declared type of each constant, for the cast below.
    pub const_types: &'a std::collections::BTreeMap<String, String>,
}

/// The constants a body may read: declared with a value. `var x = undefined;`
/// at module level is not emitted as a value, so a body naming it would be a
/// ReferenceError at the first call.
pub(crate) fn readable_consts(
    ast: &Node,
) -> (BTreeSet<String>, std::collections::BTreeMap<String, String>) {
    let mut names = BTreeSet::new();
    let mut types = std::collections::BTreeMap::new();
    for n in ast.children.iter().filter(|n| n.kind == NodeKind::ConstDecl && !n.name.is_empty()) {
        let undefined = n
            .children
            .first()
            .map_or(true, |c| c.kind == NodeKind::ExprIdentifier && c.name == "undefined");
        if !undefined {
            names.insert(n.name.clone());
            types.insert(n.name.clone(), n.extra_type.clone());
        }
    }
    (names, types)
}

fn scalar(ty: &str) -> Result<&'static str, String> {
    let t = ty.trim();
    match t {
        "bool" => Ok("boolean"),
        "str" | "[]const u8" => Ok("string"),
        "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize" => {
            Ok("number")
        }
        _ => Err(format!(
            "gen-ts: type {:?} is outside the lowered subset (bool, an integer, str)",
            t
        )),
    }
}

struct Body<'a> {
    names: &'a Names<'a>,
    locals: Vec<BTreeSet<String>>,
    out: String,
    depth: usize,
}

impl<'a> Body<'a> {
    fn line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn known_local(&self, name: &str) -> bool {
        self.locals.iter().any(|s| s.contains(name))
    }

    fn block(&mut self, stmts: &[Node]) -> Result<(), String> {
        self.locals.push(BTreeSet::new());
        self.depth += 1;
        for s in stmts {
            self.stmt(s)?;
        }
        self.depth -= 1;
        self.locals.pop();
        Ok(())
    }

    fn stmt(&mut self, s: &Node) -> Result<(), String> {
        match s.kind {
            NodeKind::ExprReturn => {
                if s.children.is_empty() {
                    self.line("return;");
                } else {
                    let v = self.expr(&s.children[0])?;
                    self.line(&format!("return {};", v));
                }
            }
            NodeKind::StmtLocal => {
                if s.name.is_empty() {
                    return Err(format!("gen-ts: a destructuring local at line {} is not lowered", s.line));
                }
                if !s.extra_type.is_empty() {
                    scalar(&s.extra_type)?;
                }
                let name = js_name(&TS, &s.name)?;
                let init = match s.children.first() {
                    Some(c) => self.expr(c)?,
                    None => return Err(format!("gen-ts: local {:?} at line {} has no value", s.name, s.line)),
                };
                // `var base = base;` shadows a parameter in t27; in TS it is a
                // second declaration of one name in one function. Refused.
                if self.known_local(&s.name) {
                    return Err(format!(
                        "gen-ts: local {:?} at line {} reuses a name already in scope",
                        s.name, s.line
                    ));
                }
                self.locals.last_mut().expect("a scope").insert(s.name.clone());
                self.line(&format!("let {} = {};", name, init));
            }
            NodeKind::StmtAssign => {
                if s.children.len() < 2 {
                    return Err(format!("gen-ts: an assignment at line {} has no value", s.line));
                }
                let target = &s.children[0];
                if target.kind != NodeKind::ExprIdentifier || !self.known_local(&target.name) {
                    return Err(format!(
                        "gen-ts: only a local may be assigned in a lowered body (line {})",
                        s.line
                    ));
                }
                let op = match s.extra_op.trim() {
                    "" | "=" => "=",
                    "+=" => "+=",
                    "-=" => "-=",
                    "*=" => "*=",
                    other => {
                        return Err(format!("gen-ts: assignment {:?} at line {} is not lowered", other, s.line))
                    }
                };
                let value = self.expr(&s.children[1])?;
                let name = js_name(&TS, &target.name)?;
                self.line(&format!("{} {} {};", name, op, value));
            }
            NodeKind::StmtIf => {
                let cond = self.expr(s.children.first().ok_or("gen-ts: an if without a condition")?)?;
                self.line(&format!("if ({}) {{", cond));
                if let Some(then) = s.children.get(1) {
                    self.block(&then.children)?;
                }
                if let Some(other) = s.children.get(2) {
                    self.line("} else {");
                    self.block(&other.children)?;
                }
                self.line("}");
            }
            NodeKind::StmtWhile => {
                if s.children.len() != 2 {
                    return Err(format!("gen-ts: a while with a step at line {} is not lowered", s.line));
                }
                let cond = self.expr(&s.children[0])?;
                self.line(&format!("while ({}) {{", cond));
                self.block(&s.children[1].children)?;
                self.line("}");
            }
            NodeKind::StmtForRange => {
                if s.children.len() < 3 || s.name.is_empty() {
                    return Err(format!("gen-ts: a range loop at line {} has an unexpected shape", s.line));
                }
                let start = self.expr(&s.children[0])?;
                let end = self.expr(&s.children[1])?;
                let var = js_name(&TS, &s.name)?;
                self.line(&format!("for (let {v} = {}; {v} < {}; {v}++) {{", start, end, v = var));
                self.locals.push(BTreeSet::from([s.name.clone()]));
                let r = self.block(&s.children[2].children);
                self.locals.pop();
                r?;
                self.line("}");
            }
            NodeKind::StmtBreak => self.line("break;"),
            NodeKind::StmtContinue => self.line("continue;"),
            _ => {
                return Err(format!(
                    "gen-ts: a {:?} statement at line {} is outside the lowered subset",
                    s.kind, s.line
                ))
            }
        }
        Ok(())
    }

    fn expr(&self, n: &Node) -> Result<String, String> {
        match n.kind {
            NodeKind::ExprLiteral => {
                if n.extra_kind == "string" {
                    return Ok(js_string(&n.value));
                }
                if n.extra_kind == "char" {
                    return Err(format!("gen-ts: a char literal at line {} is not lowered", n.line));
                }
                if is_number(&n.value) {
                    if n.value.contains('.') || n.value.contains('e') || n.value.contains('E') {
                        return Err(format!("gen-ts: a float literal at line {} is not lowered", n.line));
                    }
                    return Ok(n.value.clone());
                }
                match n.value.as_str() {
                    "true" | "false" => Ok(n.value.clone()),
                    other => Err(format!("gen-ts: literal {:?} at line {} is not lowered", other, n.line)),
                }
            }
            NodeKind::ExprIdentifier => match n.name.as_str() {
                "true" | "false" => Ok(n.name.clone()),
                name if self.known_local(name) => js_name(&TS, name),
                // A spec constant carries its LITERAL type in TS (`50 satisfies
                // number` is the type 50), so `CLK === 0` is refused by tsc as
                // comparing two literals that cannot overlap. Read in a body, it
                // is widened to its declared type, which is what t27 means.
                name if self.names.consts.contains(name) => {
                    let js = js_name(&TS, name)?;
                    match self.names.const_types.get(name).map(|t| scalar(t)) {
                        Some(Ok(t)) => Ok(format!("({} as {})", js, t)),
                        _ => Ok(js),
                    }
                }
                name => Err(format!(
                    "gen-ts: {:?} at line {} is not a parameter, local or constant of this spec",
                    name, n.line
                )),
            },
            NodeKind::ExprUnary => {
                let v = self.expr(n.children.first().ok_or("gen-ts: a unary without an operand")?)?;
                match n.extra_op.as_str() {
                    "!" | "not" => Ok(format!("!{}", v)),
                    "-" => Ok(format!("-{}", v)),
                    other => Err(format!("gen-ts: unary {:?} at line {} is not lowered", other, n.line)),
                }
            }
            NodeKind::ExprBinary => {
                if n.children.len() < 2 {
                    return Err(format!("gen-ts: a binary at line {} lacks an operand", n.line));
                }
                let a = self.expr(&n.children[0])?;
                let b = self.expr(&n.children[1])?;
                let op = match n.extra_op.as_str() {
                    "and" | "&&" => "&&",
                    "or" | "||" => "||",
                    "==" => "===",
                    "!=" => "!==",
                    "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "%" => n.extra_op.as_str(),
                    "/" => return Ok(format!("Math.trunc({} / {})", a, b)),
                    other => {
                        return Err(format!("gen-ts: operator {:?} at line {} is not lowered", other, n.line))
                    }
                };
                Ok(format!("({} {} {})", a, op, b))
            }
            NodeKind::ExprIndex => {
                if n.children.len() < 2 {
                    return Err(format!("gen-ts: an index at line {} lacks a part", n.line));
                }
                // Only into a module constant (an array of the spec). Indexing
                // a `str` gives a byte in t27 and a one-letter string in TS: the
                // same text, two meanings, so it is not lowered.
                let base = &n.children[0];
                if base.kind != NodeKind::ExprIdentifier
                    || self.known_local(&base.name)
                    || !self.names.consts.contains(&base.name)
                {
                    return Err(format!(
                        "gen-ts: an index at line {} is into something other than a constant of this spec",
                        n.line
                    ));
                }
                Ok(format!("{}[{}]", self.expr(&n.children[0])?, self.expr(&n.children[1])?))
            }
            NodeKind::ExprCall => {
                if !self.names.fns.contains(&n.name) {
                    return Err(format!(
                        "gen-ts: a call to {:?} at line {} names a function that is not lowered here",
                        n.name, n.line
                    ));
                }
                let args: Result<Vec<String>, String> = n.children.iter().map(|c| self.expr(c)).collect();
                Ok(format!("{}({})", js_name(&TS, &n.name)?, args?.join(", ")))
            }
            _ => Err(format!(
                "gen-ts: a {:?} expression at line {} is outside the lowered subset",
                n.kind, n.line
            )),
        }
    }
}

/// The functions that lower, given that a function calling one that does not
/// lower does not lower either: removed until nothing more falls out.
pub(crate) fn lowerable(
    ast: &Node,
    consts: &BTreeSet<String>,
    const_types: &std::collections::BTreeMap<String, String>,
) -> BTreeSet<String> {
    let mut fns: BTreeSet<String> = ast
        .children
        .iter()
        .filter(|n| n.kind == NodeKind::FnDecl && !n.children.is_empty())
        .map(|n| n.name.clone())
        .collect();
    loop {
        let names = Names { consts, fns: &fns, const_types };
        let failed: Vec<String> = ast
            .children
            .iter()
            .filter(|n| n.kind == NodeKind::FnDecl && fns.contains(&n.name))
            .filter(|n| lower_fn(n, &names).is_err())
            .map(|n| n.name.clone())
            .collect();
        if failed.is_empty() {
            return fns;
        }
        for f in failed {
            fns.remove(&f);
        }
    }
}

/// `export function name(...): T { ... }`, or why not.
pub(crate) fn lower_fn(node: &Node, names: &Names) -> Result<String, String> {
    if node.children.is_empty() {
        return Err("gen-ts: the function has no body in the spec".to_string());
    }
    let name = js_name(&TS, &node.name)?;
    let mut params = Vec::new();
    let mut scope = BTreeSet::new();
    for (p, ty) in &node.params {
        params.push(format!("{}: {}", js_name(&TS, p)?, scalar(ty)?));
        scope.insert(p.clone());
    }
    let ret = if node.extra_return_type.trim().is_empty() || node.extra_return_type.trim() == "void" {
        "void"
    } else {
        scalar(&node.extra_return_type)?
    };
    let mut body = Body { names, locals: vec![scope], out: String::new(), depth: 0 };
    body.block(&node.children)?;
    Ok(format!(
        "export function {}({}): {} {{\n{}}}\n",
        name,
        params.join(", "),
        ret,
        body.out
    ))
}

#[cfg(test)]
mod tests {
    use crate::compiler::Compiler;

    fn ts(src: &str) -> String {
        let ast = Compiler::parse_ast_strict(src).expect("parses");
        crate::codegen_ts::generate(&ast, "t.t27").expect("generates")
    }

    #[test]
    fn a_pure_rule_becomes_a_function() {
        let out = ts("module m { pub const CORE : u32 = 1;\n fn joins(index: u32, named: bool) -> bool { return index < CORE || named; } }");
        assert!(out.contains("export function joins(index: number, named: boolean): boolean {"), "{out}");
        assert!(out.contains("return ((index < (CORE as number)) || named);"), "{out}");
    }

    #[test]
    fn an_if_a_loop_and_a_call_are_lowered() {
        let out = ts("module m { pub const N : u32 = 3;\n pub const XS : [3]u32 = [1, 2, 3];\n fn has(c: u32) -> bool { for i in 0..N { if XS[i] == c { return true; } } return false; }\n fn cap(n: u32) -> u32 { if n > N { return N; } return n; }\n fn both(c: u32) -> bool { return has(c) and cap(c) == c; } }");
        assert!(out.contains("for (let i = 0; i < (N as number); i++) {"), "{out}");
        assert!(out.contains("if ((XS[i] === c)) {"), "{out}");
        assert!(out.contains("return (has(c) && (cap(c) === c));"), "{out}");
    }

    #[test]
    fn integer_division_truncates_as_t27_does() {
        let out = ts("module m { fn half(n: u32) -> u32 { return n / 2; } }");
        assert!(out.contains("return Math.trunc(n / 2);"), "{out}");
    }

    #[test]
    fn a_float_is_announced_not_guessed() {
        let out = ts("module m { fn f(x: f64) -> f64 { return x; } }");
        assert!(!out.contains("export function f"), "{out}");
        assert!(out.contains("fn f"), "{out}");
        assert!(out.contains("f64"), "{out}");
    }

    #[test]
    fn a_caller_of_an_unlowered_function_is_not_lowered_either() {
        let out = ts("module m { fn f(x: f64) -> f64 { return x; }\n fn g(x: u32) -> u32 { return h(x); }\n fn h(x: u32) -> u32 { return x; } }");
        assert!(out.contains("export function h("), "{out}");
        assert!(out.contains("export function g("), "{out}");
        let out = ts("module m { fn f(x: f64) -> f64 { return x; }\n fn g(x: u32) -> bool { return f(x) > 1; } }");
        assert!(!out.contains("export function g("), "{out}");
    }

    #[test]
    fn indexing_a_string_or_shadowing_a_parameter_is_not_lowered() {
        let out = ts("module m { fn first(s: str) -> u32 { return s[0]; } }");
        assert!(!out.contains("export function first("), "{out}");
        let out = ts("module m { fn f(base: u32) -> u32 { var base = base; return base; } }");
        assert!(!out.contains("export function f("), "{out}");
    }

    #[test]
    fn a_module_var_left_undefined_is_not_read() {
        let out = ts("module m { var table : [4]u8 = undefined;\n fn f(i: u32) -> u32 { return table[i]; } }");
        assert!(!out.contains("export function f("), "{out}");
    }

    #[test]
    fn a_name_from_nowhere_refuses_the_whole_function() {
        let out = ts("module m { fn f(x: u32) -> bool { return x < LIMIT; } }");
        assert!(!out.contains("export function f"), "{out}");
        assert!(out.contains("LIMIT"), "{out}");
    }
}

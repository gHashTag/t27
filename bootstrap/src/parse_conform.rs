//! W577: a parser conformance table.
//!
//! W576 gave the lexer 26 written-down behaviours. The parser had none, and its
//! failure mode is worse: it does not produce a wrong token, it produces a
//! **smaller program**. Three defects of exactly that shape have been found in
//! this chain —
//!
//! * a body discarded past a stray `}` (W569, 29 specs, 16,792 lines),
//! * a receiver dropped from `f(x).len()` (W572, 198 call sites),
//! * a clause block falling back to nothing when one shape is unrecognised
//!   (W570).
//!
//! Every one is an input that should have been an **error** and instead
//! produced a program the author did not write. So this table records a
//! *verdict* per input, and the verdict that matters is the middle one:
//!
//! * `Full`      — parses, and consumes the whole input.
//! * `Truncated` — parses, and does NOT consume the whole input. **This is
//!   never acceptable.** It is the shape that reports success while discarding
//!   the author's code.
//! * `Rejected`  — refuses to parse. For malformed input this is the CORRECT
//!   outcome, and the table says so explicitly rather than leaving it to
//!   whatever the parser happens to do.
//!
//! Declaration counts are recorded alongside, because "parses fully" is not the
//! same as "parsed everything": a body loop that swallows a declaration still
//! reaches Eof.

use crate::compiler::Compiler;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Verdict {
    Full,
    Truncated,
    Rejected,
}

pub struct Case {
    pub name: &'static str,
    pub input: &'static str,
    pub verdict: Verdict,
    /// Expected number of top-level declarations, when the input parses.
    pub decls: Option<usize>,
    /// Expected count of top-level tokens that recovery DISCARDED. `None` says
    /// the case does not pin it; `Some(0)` says nothing may be dropped.
    ///
    /// Reaching EOF is not the same as reading everything. Without this field a
    /// case could only demand accept-or-reject, so `stray_closing_brace` was
    /// written as Rejected -- the only way the table could say "this input is
    /// not clean". The parser meanwhile stopped ending the file at a stray `}`
    /// and started counting it instead, and the row was never restated in the
    /// terms that became available.
    pub discards: Option<usize>,
    pub note: &'static str,
}

pub struct Outcome {
    pub name: String,
    pub expected: String,
    pub actual: String,
    pub note: String,
}

fn evaluate(src: &str) -> (Verdict, usize, usize) {
    let discards = Compiler::parse_ast_accounted(src).map(|(_, d)| d).unwrap_or(0);
    match Compiler::parse_ast(src) {
        Err(_) => (Verdict::Rejected, 0, discards),
        Ok(ast) => match Compiler::parse_ast_strict(src) {
            Ok(a) => (Verdict::Full, a.children.len(), discards),
            Err(_) => (Verdict::Truncated, ast.children.len(), discards),
        },
    }
}

pub const CASES: &[Case] = &[
    Case {
        name: "two_fns",
        input: "module m\n\nfn a() -> u32 { return 1; }\n\nfn b() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "the baseline: both declarations reach the AST",
    },
    Case {
        name: "stray_closing_brace",
        input: "module m\n\nfn a() -> u32 { return 1; }\n\n}\n\nfn b() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(1),
        note: "W569: a `}` with nothing to close must never be a QUIET end of file. \
It is no longer quiet and no longer an end: recovery keeps `fn b`, which a \
rejection would have thrown away, and counts the brace. This row asserted \
Rejected because the table had no way to say `accepted, and one token dropped` \
until `discards` existed -- and it had been failing ever since the parser was \
fixed, which is why the requirement now sits on the count. The corpus-wide \
version of this is the parse-no-discard suite phase.",
    },
    Case {
        name: "unterminated_string",
        input: "module m\n\nconst S = \"oops\n\nfn a() -> u32 { return 1; }\n",
        verdict: Verdict::Rejected,
        decls: None,
        discards: Some(0),
        note: "an unterminated string used to swallow the file and report success",
    },
    Case {
        name: "struct_with_method",
        input: "module m\n\npub const S = struct {\n    x: u32,\n    pub fn get(self: S) u32 {\n        return self.x;\n    }\n};\n\nfn after() -> u32 { return 7; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W577: a method's closing brace used to end the struct AND the module -- jit.t27 lost 797 of 875 lines",
    },
    Case {
        name: "second_module_header",
        input: "module a;\n\nfn one() -> u32 { return 1; }\n\nmodule b;\n\nfn two() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W577: attention.t27 appends a second module at line 640 of 922",
    },
    Case {
        name: "braced_module_then_more",
        input: "module a {\n    fn one() -> u32 { return 1; }\n}\n\nmodule b;\n\nfn two() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W577: the braced form used to RETURN at its closing brace, discarding the rest",
    },
    Case {
        name: "method_call_on_call_result",
        input: "module m\n\nfn mk() -> []u32 { return [1, 2]; }\n\ntest t\n    then mk().len() == 2\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W572: the receiver of `f(x).len()` must survive -- it used to be dropped silently",
    },
    Case {
        name: "array_literal_in_given",
        input: "module m\n\ntest t\n    given a = [1, 2, 3]\n    then a.len() == 3\n",
        verdict: Verdict::Full,
        decls: Some(1),
        discards: Some(0),
        note: "W570: the clause block must lower, not fall back to an empty test",
    },
    Case {
        name: "bare_assert_in_brace_body",
        input: "module m\n\ntest t {\n    assert true\n}\n\nfn after() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W569: `assert <expr>` as a statement; 3,682 occurrences",
    },
    Case {
        name: "bare_array_const_then_fn",
        input: "module m\n\nconst A : [3]u32 = [1, 2, 3]\n\nfn g() -> u32 { return A[0]; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W568: the const value collector ran to the next SEMICOLON and ate the file",
    },
    Case {
        name: "struct_field_slice_type",
        input: "module m\n\nstruct S {\n    a: []const u8,\n    b: std.mem.Allocator,\n}\n\nfn g() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W568: `[]const u8` and dotted types in struct fields",
    },
    Case {
        name: "invariant_then_decl",
        input: "module m\n\ninvariant inv\n    assert true\n\nfn after() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W567: a keyword-form invariant must not swallow what follows",
    },
    Case {
        name: "slice_expression",
        input: "module m\n\nfn head(s: []const u8) -> []const u8 {\n    return s[0:5];\n}\n\nfn after() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W605: `x[a:b]` is a slice -- 33 sites in code, every one in IGLA CODER; eval.t27 failed on it at line 1394",
    },
    Case {
        name: "index_still_parses",
        input: "module m\n\nfn first(s: []const u8) -> u8 {\n    return s[0];\n}\n\nfn after() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "W605: adding slices must not break ordinary indexing",
    },
    Case {
        name: "braceless_block_survives_a_comment",
        input: "module m\n\ninvariant i\n    // a comment between the header and the body\n    const a = f(1);\n\nfn b() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "A comment is not a blank line. The boundary asked `line <= last_line + 1`, so a comment read as a gap and the whole body went to the top-level discard -- silently, 33,777 tokens across 87 specs",
    },
    Case {
        name: "then_clause_takes_a_statement",
        input: "module m\n\ninvariant i\n    then for (x in 0..3) {\n        assert x == x;\n    }\n\nfn b() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "`then` parsed its body with parse_expr, so `then for (...) { ... }` failed and took the asserts INSIDE the loop to the discard with it. `then <expr>` alone always worked, which is why this survived",
    },
    Case {
        name: "array_type_const_is_not_a_boundary",
        input: "module m\n\ninvariant i\n    const b = [_][]const u8{ \"z\" };\n    assert true;\n\nfn b2() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: None,
        note: "the `const` inside `[]const u8` is a block boundary token, so the half-parsed `[_][]` was minted as a CLEAN statement while the parser stood mid-type. The next turn read `const u8{` as a second statement, failed, and left it at module level -- where a const is a HARD error, and one already-lowered statement meant the whole-block fallback never fired",
    },
    Case {
        name: "indent_outranks_a_blank_line_inside_a_block",
        input: "module m\n\ninvariant i\n    var a : [2]i32 = [_]i32{ 1, 2 };\n\n    const b = a[0];\n    assert b == 1;\n\nfn f() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "a blank line used to end the block, so the var lowered INSIDE the invariant's comptime block and the const hoisted OUTSIDE it -- emitting a reference to a name that no longer existed, and colliding with the next invariant's identically-named const. Once a statement has lowered, the block's column is known and a statement at or deeper than it is a continuation",
    },
    Case {
        name: "a_bare_call_is_a_statement_not_a_clause_head",
        input: "module m\n\n    test t\n        var a : usize = 1;\n        setup(&a);\n        assert a == 1\n\nfn f() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "`setup` is an Ident, so it was read as a CLAUSE HEAD and its argument list as the clause's value; the lowering failed and the whole block fell to the discard, taking the assertion after it. specs/isa/ternary_deque.t27 lost 1873 tokens this way -- and the calls are what SET UP the state the assertions check",
    },
    Case {
        name: "a_fn_spelled_as_a_test_keeps_its_clauses",
        input: "module m\n\nfn g(x: u32) -> bool { return x == 0; }\n\n    fn plain_test()\n        given a = 1\n        then g(a) == false\n",
        verdict: Verdict::Full,
        decls: Some(2),
        discards: Some(0),
        note: "the arm RECOGNISED this shape by name -- `fn name() given ...` -- and then called skip_to_next_top_level on it, throwing away every clause. The shared clause parser already serves test, invariant and bench; detection is unchanged, only the action is",
    },
    Case {
        name: "a_statement_may_stand_where_a_clause_head_is_expected",
        input: "module m\n\n    test t\n        var n : usize = 0;\n        for (i in 0..3) {\n            n = n + 1;\n        }\n        assert n == 3\n",
        verdict: Verdict::Full,
        decls: Some(1),
        discards: Some(0),
        note: "`for` and `while` are keywords, so they are neither a clause head nor a block boundary and the loop fell through to the whole-block fallback -- taking the assertion after it. 110 fallback events across ~25 specs, the largest non-`forall` shape in the distribution",
    },
    Case {
        name: "one_clause_may_bind_several_names",
        input: "module m\n\n    test t\n        given clk = true, rst_n = false, angle = 4096\n        then clk == true\n",
        verdict: Verdict::Full,
        decls: Some(1),
        discards: Some(0),
        note: "the loop's own comment named this shape -- `given clk = true, rst_n = false` -- as the reason a block can stop mid-clause, and nothing acted on it. 19 fallback events in one spec, each taking a whole block of clauses with it",
    },
    Case {
        name: "a_clause_binding_may_carry_a_type",
        input: "module m\n\n    test t\n        given crossings : [i32] = [1]\n        then crossings[0] == 1\n",
        verdict: Verdict::Full,
        decls: Some(1),
        discards: Some(0),
        note: "the binding arm peeked for `=` immediately after the name, so an annotation between them read as `not a binding` and the whole block went to the discard",
    },
    Case {
        name: "unterminated_fn_body",
        input: "module m\n\nfn a() -> u32 { return 1;\n",
        verdict: Verdict::Rejected,
        decls: None,
        discards: Some(0),
        note: "a body with no closing brace is an error, not a truncation",
    },
];

pub fn run() -> Vec<Outcome> {
    let mut failures = Vec::new();
    for c in CASES {
        let (verdict, decls, discards) = evaluate(c.input);
        let decl_ok = match (c.decls, verdict) {
            (Some(n), Verdict::Full) => decls == n,
            _ => true,
        };
        let discard_ok = c.discards.map(|n| discards == n).unwrap_or(true);
        if verdict != c.verdict || !decl_ok || !discard_ok {
            failures.push(Outcome {
                name: c.name.to_string(),
                expected: match c.decls {
                    Some(n) => format!(
                        "{:?} with {} decl(s), {} discarded",
                        c.verdict,
                        n,
                        c.discards.map(|d| d.to_string()).unwrap_or("any".into())
                    ),
                    None => format!(
                        "{:?}, {} discarded",
                        c.verdict,
                        c.discards.map(|d| d.to_string()).unwrap_or("any".into())
                    ),
                },
                actual: format!(
                    "{:?} with {} decl(s), {} discarded",
                    verdict, decls, discards
                ),
                note: c.note.to_string(),
            });
        }
    }
    failures
}

pub fn total() -> usize {
    CASES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_matches_its_conformance_table() {
        let failures = run();
        if !failures.is_empty() {
            let mut msg = String::from("parser conformance failures:\n");
            for f in &failures {
                msg.push_str(&format!(
                    "  {}\n    expected: {}\n    actual:   {}\n    note: {}\n",
                    f.name, f.expected, f.actual, f.note
                ));
            }
            panic!("{}", msg);
        }
    }
}

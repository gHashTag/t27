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
    pub note: &'static str,
}

pub struct Outcome {
    pub name: String,
    pub expected: String,
    pub actual: String,
    pub note: String,
}

fn evaluate(src: &str) -> (Verdict, usize) {
    match Compiler::parse_ast(src) {
        Err(_) => (Verdict::Rejected, 0),
        Ok(ast) => match Compiler::parse_ast_strict(src) {
            Ok(a) => (Verdict::Full, a.children.len()),
            Err(_) => (Verdict::Truncated, ast.children.len()),
        },
    }
}

pub const CASES: &[Case] = &[
    Case {
        name: "two_fns",
        input: "module m\n\nfn a() -> u32 { return 1; }\n\nfn b() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "the baseline: both declarations reach the AST",
    },
    Case {
        name: "stray_closing_brace",
        input: "module m\n\nfn a() -> u32 { return 1; }\n\n}\n\nfn b() -> u32 { return 2; }\n",
        verdict: Verdict::Rejected,
        decls: None,
        note: "W569: a `}` with nothing to close must be an ERROR, never a quiet end of file",
    },
    Case {
        name: "unterminated_string",
        input: "module m\n\nconst S = \"oops\n\nfn a() -> u32 { return 1; }\n",
        verdict: Verdict::Rejected,
        decls: None,
        note: "an unterminated string used to swallow the file and report success",
    },
    Case {
        name: "struct_with_method",
        input: "module m\n\npub const S = struct {\n    x: u32,\n    pub fn get(self: S) u32 {\n        return self.x;\n    }\n};\n\nfn after() -> u32 { return 7; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W577: a method's closing brace used to end the struct AND the module -- jit.t27 lost 797 of 875 lines",
    },
    Case {
        name: "second_module_header",
        input: "module a;\n\nfn one() -> u32 { return 1; }\n\nmodule b;\n\nfn two() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W577: attention.t27 appends a second module at line 640 of 922",
    },
    Case {
        name: "braced_module_then_more",
        input: "module a {\n    fn one() -> u32 { return 1; }\n}\n\nmodule b;\n\nfn two() -> u32 { return 2; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W577: the braced form used to RETURN at its closing brace, discarding the rest",
    },
    Case {
        name: "method_call_on_call_result",
        input: "module m\n\nfn mk() -> []u32 { return [1, 2]; }\n\ntest t\n    then mk().len() == 2\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W572: the receiver of `f(x).len()` must survive -- it used to be dropped silently",
    },
    Case {
        name: "array_literal_in_given",
        input: "module m\n\ntest t\n    given a = [1, 2, 3]\n    then a.len() == 3\n",
        verdict: Verdict::Full,
        decls: Some(1),
        note: "W570: the clause block must lower, not fall back to an empty test",
    },
    Case {
        name: "bare_assert_in_brace_body",
        input: "module m\n\ntest t {\n    assert true\n}\n\nfn after() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W569: `assert <expr>` as a statement; 3,682 occurrences",
    },
    Case {
        name: "bare_array_const_then_fn",
        input: "module m\n\nconst A : [3]u32 = [1, 2, 3]\n\nfn g() -> u32 { return A[0]; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W568: the const value collector ran to the next SEMICOLON and ate the file",
    },
    Case {
        name: "struct_field_slice_type",
        input: "module m\n\nstruct S {\n    a: []const u8,\n    b: std.mem.Allocator,\n}\n\nfn g() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W568: `[]const u8` and dotted types in struct fields",
    },
    Case {
        name: "invariant_then_decl",
        input: "module m\n\ninvariant inv\n    assert true\n\nfn after() -> u32 { return 1; }\n",
        verdict: Verdict::Full,
        decls: Some(2),
        note: "W567: a keyword-form invariant must not swallow what follows",
    },
    Case {
        name: "unterminated_fn_body",
        input: "module m\n\nfn a() -> u32 { return 1;\n",
        verdict: Verdict::Rejected,
        decls: None,
        note: "a body with no closing brace is an error, not a truncation",
    },
];

pub fn run() -> Vec<Outcome> {
    let mut failures = Vec::new();
    for c in CASES {
        let (verdict, decls) = evaluate(c.input);
        let decl_ok = match (c.decls, verdict) {
            (Some(n), Verdict::Full) => decls == n,
            _ => true,
        };
        if verdict != c.verdict || !decl_ok {
            failures.push(Outcome {
                name: c.name.to_string(),
                expected: match c.decls {
                    Some(n) => format!("{:?} with {} decl(s)", c.verdict, n),
                    None => format!("{:?}", c.verdict),
                },
                actual: format!("{:?} with {} decl(s)", verdict, decls),
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

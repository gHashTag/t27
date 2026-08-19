//! W576: a lexer conformance table.
//!
//! The lexer is the component this project had the least instrumentation for,
//! and the one where a defect survives longest: a mis-lexed *value* produces no
//! error, no warning and no diagnostic — it silently changes what a spec
//! asserts. `1e6` lexed as the number `1` followed by the identifier `e6` for
//! the entire life of the project (486 occurrences, 62 specs) and was found in
//! W575 only by accident, by an arity checker built for something else.
//!
//! This table states, for a set of inputs, the exact token sequence the lexer
//! must produce. It is deliberately written as data rather than as assertions
//! scattered through tests, so that adding a case costs one line and the
//! failures read as a diff of expectation against reality.
//!
//! Every case is either
//!
//! * a **contract** — a form the corpus uses and depends on, or
//! * a **boundary** — an input whose current tokenisation was measured rather
//!   than designed, recorded so a change to it is visible rather than silent.
//!
//! Boundary cases are marked as such. A boundary case failing does not mean the
//! lexer is wrong; it means someone changed behaviour nobody had written down.

use crate::compiler::{Lexer, TokenKind};

pub struct Case {
    pub input: &'static str,
    /// The expected `(kind, lexeme)` sequence, excluding the final `Eof`.
    pub expect: &'static [(TokenKind, &'static str)],
    pub note: &'static str,
    pub boundary: bool,
}

pub struct Outcome {
    pub input: String,
    pub expected: String,
    pub actual: String,
    pub note: String,
    pub boundary: bool,
}

fn tokenize(src: &str) -> Vec<(TokenKind, String)> {
    let mut lexer = Lexer::new(src);
    let mut out = Vec::new();
    loop {
        let t = lexer.next_token();
        if t.kind == TokenKind::Eof {
            break;
        }
        out.push((t.kind.clone(), t.lexeme.clone()));
        if out.len() > 64 {
            break; // a runaway lexer must not hang the check
        }
    }
    out
}

fn render(toks: &[(TokenKind, String)]) -> String {
    toks.iter()
        .map(|(k, l)| format!("{:?}({})", k, l))
        .collect::<Vec<_>>()
        .join(" ")
}

use TokenKind::{Dot, Ident, Minus, Number, Star};

pub const CASES: &[Case] = &[
    // ---- numbers: the class that produced the W575 defect -----------------
    Case {
        input: "42",
        expect: &[(Number, "42")],
        note: "plain decimal",
        boundary: false,
    },
    Case {
        input: "1e6",
        expect: &[(Number, "1e6")],
        note: "positive exponent -- was Number(1) Ident(e6) before W575",
        boundary: false,
    },
    Case {
        input: "2.5e-3",
        expect: &[(Number, "2.5e-3")],
        note: "negative exponent -- was Number(2.5) Ident(e) Minus Number(3)",
        boundary: false,
    },
    Case {
        input: "1E9",
        expect: &[(Number, "1E9")],
        note: "capital exponent marker",
        boundary: false,
    },
    Case {
        input: "1e+6",
        expect: &[(Number, "1e+6")],
        note: "explicit positive sign in the exponent",
        boundary: false,
    },
    Case {
        input: "0x1e",
        expect: &[(Number, "0x1e")],
        note: "hex, NOT an exponent -- the `e` is a hex digit",
        boundary: false,
    },
    Case {
        input: "0xFF",
        expect: &[(Number, "0xFF")],
        note: "hex with upper-case digits",
        boundary: false,
    },
    Case {
        input: "0b1010",
        expect: &[(Number, "0b1010")],
        note: "binary literal",
        boundary: false,
    },
    Case {
        input: "1_000_000",
        expect: &[(Number, "1_000_000")],
        note: "digit separators",
        boundary: false,
    },
    Case {
        input: "3.14",
        expect: &[(Number, "3.14")],
        note: "decimal point",
        boundary: false,
    },
    Case {
        input: "1..3",
        expect: &[(Number, "1"), (TokenKind::DotDot, ".."), (Number, "3")],
        note: "a range operator must not be eaten as a decimal point",
        boundary: false,
    },
    Case {
        input: "e6",
        expect: &[(Ident, "e6")],
        note: "a bare identifier that looks like an exponent stays an identifier",
        boundary: false,
    },
    // ---- boundaries: measured, not designed -------------------------------
    Case {
        input: "1x2",
        expect: &[(Number, "1x2")],
        note: "BOUNDARY: `x` is accepted anywhere in a number, not just as a 0x prefix",
        boundary: true,
    },
    Case {
        input: "0b12",
        expect: &[(Number, "0b12")],
        note: "BOUNDARY: binary literal with a non-binary digit is not rejected",
        boundary: true,
    },
    Case {
        input: "1.2.3",
        expect: &[(Number, "1.2.3")],
        note: "BOUNDARY: two decimal points lex as one number",
        boundary: true,
    },
    // ---- operators the corpus depends on ----------------------------------
    Case {
        input: "a +% b",
        expect: &[
            (Ident, "a"),
            (TokenKind::PlusPercent, "+%"),
            (Ident, "b"),
        ],
        note: "wrapping add -- W573 depends on this being one token",
        boundary: false,
    },
    Case {
        input: "a -% b",
        expect: &[
            (Ident, "a"),
            (TokenKind::MinusPercent, "-%"),
            (Ident, "b"),
        ],
        note: "wrapping subtract",
        boundary: false,
    },
    Case {
        input: "a *% b",
        expect: &[(Ident, "a"), (TokenKind::StarPercent, "*%"), (Ident, "b")],
        note: "wrapping multiply",
        boundary: false,
    },
    Case {
        input: "a.b.c",
        expect: &[
            (Ident, "a"),
            (Dot, "."),
            (Ident, "b"),
            (Dot, "."),
            (Ident, "c"),
        ],
        note: "dotted path -- W568's type-annotation fix depends on the dots being separate",
        boundary: false,
    },
    Case {
        input: "x.*",
        expect: &[(Ident, "x"), (Dot, "."), (Star, "*")],
        note: "pointer dereference",
        boundary: false,
    },
    Case {
        input: "-5",
        expect: &[(Minus, "-"), (Number, "5")],
        note: "a negative literal is a unary minus, not part of the number",
        boundary: false,
    },
    // ---- comments and strings --------------------------------------------
    Case {
        input: "1 // 2\n3",
        expect: &[(Number, "1"), (Number, "3")],
        note: "line comment",
        boundary: false,
    },
    Case {
        input: "1 /* 2 */ 3",
        expect: &[(Number, "1"), (Number, "3")],
        note: "block comment",
        boundary: false,
    },
    Case {
        input: "\"a\\\"b\"",
        expect: &[(TokenKind::String, "a\"b")],
        note: "escaped quote inside a string -- the lexeme is UNESCAPED and unquoted",
        boundary: false,
    },
    Case {
        input: "\"\"",
        expect: &[(TokenKind::String, "")],
        note: "empty string -- distinguishable from a missing token only by kind",
        boundary: false,
    },
    // ---- single quotes: W604 -------------------------------------------
    Case {
        input: "'c'",
        expect: &[(TokenKind::CharLiteral, "c")],
        note: "a one-character single-quoted literal is a CHAR -- 69 sites in 19 specs",
        boundary: false,
    },
    Case {
        input: "'\\n'",
        expect: &[(TokenKind::CharLiteral, "\\n")],
        note: "an escape is still a char literal",
        boundary: false,
    },
    Case {
        input: "'abc'",
        expect: &[(TokenKind::String, "abc")],
        note: "W604: a MULTI-character single-quoted literal is a STRING -- 120 sites in 10 specs. Before W604 this lexed as CharLiteral(a) followed by loose tokens",
        boundary: false,
    },
    Case {
        input: "'{\"m\": [2,2]}'",
        expect: &[(TokenKind::String, "{\"m\": [2,2]}")],
        note: "W604: the exact shape that cost weights.t27 77% of its lines -- the `}` used to end the enclosing module",
        boundary: false,
    },
    Case {
        input: "'abc",
        expect: &[(TokenKind::UnterminatedString, "abc")],
        note: "W604: an unterminated single quote is an ERROR, not silent garbage (W577's rule, one layer down)",
        boundary: false,
    },
    // ---- silently dropped input ------------------------------------------
    Case {
        input: "1 # 2",
        expect: &[(Number, "1"), (Number, "2")],
        note: "BOUNDARY: the lexer DISCARDS an unrecognised character with no diagnostic -- `#` vanishes",
        boundary: true,
    },
    Case {
        input: "#[test]",
        expect: &[
            (TokenKind::LBracket, "["),
            (TokenKind::KwTest, "test"),
            (TokenKind::RBracket, "]"),
        ],
        note: "BOUNDARY: consequence of the above -- a Rust attribute arrives as a bare bracket group, and `test` inside it is the KEYWORD (W579)",
        boundary: true,
    },
    Case {
        input: "1 $ 2",
        expect: &[(Number, "1"), (Number, "2")],
        note: "BOUNDARY: same silent drop for any unrecognised byte, not just `#`",
        boundary: true,
    },
    Case {
        input: "\"a\\nb\"",
        expect: &[(TokenKind::String, "a\nb")],
        note: "BOUNDARY: \\n is unescaped in the lexeme, so a backend must RE-escape it",
        boundary: true,
    },
];

/// Run every case; return only those whose tokenisation differs.
pub fn run() -> Vec<Outcome> {
    let mut failures = Vec::new();
    for c in CASES {
        let actual = tokenize(c.input);
        let expected: Vec<(TokenKind, String)> = c
            .expect
            .iter()
            .map(|(k, l)| (k.clone(), (*l).to_string()))
            .collect();
        if actual != expected {
            failures.push(Outcome {
                input: c.input.to_string(),
                expected: render(&expected),
                actual: render(&actual),
                note: c.note.to_string(),
                boundary: c.boundary,
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
    fn lexer_matches_its_conformance_table() {
        let failures = run();
        if !failures.is_empty() {
            let mut msg = String::from("lexer conformance failures:\n");
            for f in &failures {
                msg.push_str(&format!(
                    "  input {:?}\n    expected: {}\n    actual:   {}\n    note: {}\n",
                    f.input, f.expected, f.actual, f.note
                ));
            }
            panic!("{}", msg);
        }
    }
}

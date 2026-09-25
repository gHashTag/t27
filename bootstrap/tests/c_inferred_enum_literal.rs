//! `.neg` is an enum member too -- the THIRD spelling, and the one the
//! campaign had filed as unfixable.
//!
//! `Trit.neg` was repaired in #3468 and `Trit::neg` in #3470. This is Zig's
//! INFERRED literal, whose type comes from context:
//!
//! ```t27
//! return switch (a) { .neg => .pos, .zero => .zero, .pos => .neg };
//! ```
//!
//! C received `POS`, `NEG`, `ZERO` -- the member upper-cased with no type
//! prefix, which nothing declares. The label carried for eight passes was
//! *"those bare names appear nowhere in the specs"*. True of the text, false of
//! its origin: they come from `.pos` and `.neg`, and the owner is one lookup
//! away.
//!
//! THE OWNER IS LOOKED UP, NOT GUESSED. A variant declared by exactly one enum
//! in its translation unit takes that enum's prefix; one declared by two --
//! `ERROR` in two enums, `PTR`, `SRV`, `TXT` in the corpus -- is left as it
//! was, because choosing between two owners would be a guess and the loud form
//! names the real ambiguity.
//!
//! BOTH HALVES OF THE SWITCH. The arm's VALUE and the case LABEL are the same
//! literal, and fixing only the value leaves `(a == NEG) ? (TRIT_POS)` -- half
//! a repair, and the half that still does not compile.

mod common;
use common::{cc_syntax_args};

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-inferred-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c")).arg("gen-c").arg(&p).output().expect("t27c");
    let h = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!h.is_empty(), "gen-c produced an EMPTY header");
    (h, d)
}

fn errors(h: &str, d: &std::path::Path) -> usize {
    let p = d.join("h.h");
    std::fs::write(&p, h).expect("write");
    let out = Command::new("cc")
        .args(cc_syntax_args())
        .arg(&p)
        .output()
        .expect("cc");
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| {
            let mut it = l.splitn(4, ':');
            it.next().is_some()
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim_start().starts_with("error: "))
        })
        .count()
}

const NEGATE: &str = "module P {\n    enum Trit { neg, zero, pos }\n\
    \x20   pub fn trit_negate(a: Trit) Trit {\n\
    \x20       return switch (a) {\n\
    \x20           .neg => .pos,\n\
    \x20           .zero => .zero,\n\
    \x20           .pos => .neg,\n\
    \x20       };\n    }\n}\n";

#[test]
fn an_inferred_literal_takes_its_owners_prefix() {
    let (h, d) = gen_c(NEGATE, "value");
    assert!(h.contains("(TRIT_POS)"), "the arm's value:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn the_case_label_takes_it_too() {
    // THE HALF THAT WAS MISSED FIRST. Repairing the value alone gives
    // `(a == NEG) ? (TRIT_POS)`, which still does not compile -- the label is
    // the same literal in the same switch.
    let (h, _d) = gen_c(NEGATE, "label");
    assert!(h.contains("a == TRIT_NEG"), "the case label:\n{h}");
    assert!(!h.contains("a == NEG)"), "no bare label may survive:\n{h}");
}

#[test]
fn a_variant_two_enums_declare_is_left_alone() {
    // THE DISCRIMINATING CASE. An inferred literal carries no type, so picking
    // one of two owners would be a guess; the loud form names the ambiguity.
    let (h, _d) = gen_c(
        "module P {\n    enum A { red, shared }\n    enum B { blue, shared }\n\
         \x20   fn f(x: A) A { return switch (x) { .red => .shared, .shared => .red }; }\n}\n",
        "ambiguous",
    );
    // The assertion is about the USE, not the header: `A_SHARED` and
    // `B_SHARED` are both DECLARED a few lines above, so a whole-file search
    // for them fails on a correct compiler. The first version did exactly that.
    let ret = h
        .lines()
        .find(|l| l.trim_start().starts_with("return (x =="))
        .expect("the lowered switch")
        .to_string();
    assert!(!ret.contains("A_SHARED"), "two owners is not a choice to make:\n{ret}");
    assert!(!ret.contains("B_SHARED"), "and neither owner may be picked:\n{ret}");
    assert!(ret.contains("SHARED"), "the bare form stays, and stays loud:\n{ret}");
    assert!(ret.contains("A_RED"), "while an unambiguous variant beside it is prefixed:\n{ret}");
}

#[test]
fn a_known_variant_is_prefixed_and_nothing_is_invented() {
    let (h, _d) = gen_c(
        "module P {\n    enum A { red, green }\n\
         \x20   fn f(x: A) A { return switch (x) { .red => .green, .green => .red }; }\n}\n",
        "known",
    );
    assert!(h.contains("A_GREEN"), "a known variant is prefixed:\n{h}");
    assert!(!h.contains("A_PURPLE"), "and nothing is invented:\n{h}");
}

#[test]
fn a_numeric_case_label_is_untouched() {
    // The switch lowering also carries integer labels, and a number must not
    // acquire an enum prefix.
    let (h, d) = gen_c(
        "module P {\n    fn f(x: i32) i32 { return switch (x) { 0 => 1, 1 => 0, else => 2 }; }\n}\n",
        "numeric",
    );
    assert!(h.contains("x == 0"), "a numeric label stays numeric:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it compiles:\n{h}");
    }
}

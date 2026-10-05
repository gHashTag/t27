//! Array and struct parameters and locals the body assigns. The sources are the
//! conformance spec `specs/tri/t27b/conformance/value_params.t27` and a
//! refusal case; each was run under `t27c test-report` first.

mod common;
use common::{names_ok, rejected, run};

/// The conformance spec: `t27c test-report` gives 10 pass.
#[test]
fn conformance_spec_passes() {
    let src = include_str!("../../../specs/tri/t27b/conformance/value_params.t27");
    let ran = run(src);
    let got = names_ok(&ran);
    assert_eq!(got.len(), 10);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// A parameter written only through a field's element is not renamed by the
/// reference, which then refuses the write: `t27c test-report` says
/// "cannot assign to constant". t27b refuses it too.
#[test]
fn a_write_only_deeper_is_refused() {
    let src = r#"module e;

struct Counter {
    hits: [4]u32,
    len: usize,
}

fn first(c: Counter) -> u32 {
    c.hits[0] = 1;
    return c.hits[0];
}

test deep {
    const c = Counter { hits: [0, 0, 0, 0], len: 0 };
    assert(first(c) == 1);
}
"#;
    let r = rejected(src);
    assert!(r.contains("StmtAssign") && r.contains("assignment through a constant"), "{}", r);
}

/// The same for a local: `c.hits[0] = 1` leaves `c` a `const` in the
/// reference ("cannot assign to constant"), so t27b refuses it.
#[test]
fn a_local_written_only_deeper_is_refused() {
    let src = r#"module e;

struct Counter {
    hits: [4]u32,
    len: usize,
}

test deep {
    let c = Counter { hits: [0, 0, 0, 0], len: 0 };
    c.hits[0] = 1;
    assert(c.hits[0] == 1);
}
"#;
    let r = rejected(src);
    assert!(r.contains("StmtAssign") && r.contains("assignment through a constant"), "{}", r);
}

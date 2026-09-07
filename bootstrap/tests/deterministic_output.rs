//! The same spec, the same binary, twice: the same bytes.
//!
//! `use_resolve` walked its frontier as a `HashSet` and picked among agreeing
//! declarations out of a `HashMap`, and Rust randomises both per process. Four
//! specs therefore emitted different output on different runs of one binary --
//! same line count every time, a pure permutation, with an imported function
//! landing in a different place.
//!
//! Measured before the repair, six runs each:
//!
//!   igla/coder/_tmp_pipeline_import      3 distinct (10 runs), gen-c 4 of 5
//!   physics/sacred_verification          3 distinct
//!   physics/zamolodchikov_4d_conjecture  2 distinct
//!   igla/coder/pipeline                  varied
//!
//! The code claimed otherwise. `pulled.sort_by(..)` carries the comment
//! "Deterministic order: by origin, then by name, so regenerating a spec twice
//! produces byte-identical output" -- and the two random walks upstream of it
//! decided WHICH declaration reached that sort.
//!
//! This matters beyond tidiness: a project whose stated property is that four
//! backends agree cannot check that claim against output that changes between
//! runs, and no seal over generated code means anything if the code is not a
//! function of its input.

use std::process::Command;

/// Every backend that lowers a whole spec. `gen-verilog` was already
/// deterministic on these inputs and is here so a future regression in it is
/// caught too.
const BACKENDS: [&str; 4] = ["gen", "gen-rust", "gen-c", "gen-verilog"];

/// Specs that actually reproduced the defect. A synthetic fixture would have to
/// recreate an ambiguous cross-module import to exercise the same path, and a
/// fixture that failed to would pass against a compiler that never fixed it.
const SPECS: [&str; 3] = [
    "specs/igla/coder/_tmp_pipeline_import.t27",
    "specs/physics/sacred_verification.t27",
    "specs/physics/zamolodchikov_4d_conjecture.t27",
];

fn repo_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR is bootstrap/; the specs live beside it.
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("bootstrap has a parent")
        .to_path_buf()
}

#[test]
fn generating_the_same_spec_twice_gives_the_same_bytes() {
    let root = repo_root();
    let mut checked = 0;
    for spec in SPECS {
        let path = root.join(spec);
        if !path.exists() {
            // Loudly, not quietly: an absent input is not a passing test.
            eprintln!("SKIP {spec}: not in this tree");
            continue;
        }
        for backend in BACKENDS {
            let mut seen: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
            for _ in 0..6 {
                let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
                    .arg(backend)
                    .arg(&path)
                    .output()
                    .expect("run t27c");
                seen.insert(out.stdout);
            }
            assert_eq!(
                seen.len(),
                1,
                "{backend} on {spec} produced {} distinct outputs in 6 runs",
                seen.len()
            );
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "no spec was checked -- the test measured nothing"
    );
}

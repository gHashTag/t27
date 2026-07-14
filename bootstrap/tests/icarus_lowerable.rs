// ============================================================================
// W534 — structural Icarus lowerability boundary integration test.
//
// Exercises the new `t27c icarus-lowerable` subcommand against the W534
// adversarial negative witnesses and a handful of known-lowerable W5xx/W3xx
// positive witnesses.  The structural classifier must reject non-lowerable
// source constructs (host-only helpers, non-lowerable types, unresolved
// imports, unbounded while(true)) and accept the lowerable subset.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// Closes #1505
// ============================================================================

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn scratch_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../specs/scratch"))
}

fn run_icarus_lowerable(path: &std::path::Path) -> (bool, String) {
    let out = Command::new(bin())
        .args(["icarus-lowerable", "--json", &path.to_string_lossy()])
        .output()
        .expect("failed to spawn t27c icarus-lowerable");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let verdict: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("invalid JSON verdict: {}", stdout));
    let lowerable = verdict
        .get("lowerable")
        .and_then(|v| v.as_bool())
        .expect("missing lowerable boolean");
    (lowerable, stdout)
}

#[test]
fn rejects_w534_negative_witnesses() {
    let dir = scratch_dir();
    let mut witnesses: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("failed to read specs/scratch")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("w534_negative_"))
                .unwrap_or(false)
        })
        .collect();
    witnesses.sort();
    assert!(
        !witnesses.is_empty(),
        "expected W534 negative witnesses in specs/scratch"
    );
    for p in &witnesses {
        let (lowerable, json) = run_icarus_lowerable(p);
        assert!(
            !lowerable,
            "expected {} to be rejected, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_known_lowerable_witnesses() {
    let dir = scratch_dir();
    let positive = [
        "w532_signed_struct_array_field_2d_copy.t27",
        "w533_module_scalar_struct_return.t27",
        "w528_function_2d_struct_array_param.t27",
    ];
    for name in &positive {
        let p = dir.join(name);
        assert!(p.exists(), "missing positive witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

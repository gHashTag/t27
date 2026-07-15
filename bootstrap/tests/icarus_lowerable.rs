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
fn rejects_w537_undefined_struct_witness() {
    let dir = scratch_dir();
    let p = dir.join("w537_negative_undefined_struct.t27");
    assert!(p.exists(), "missing W537 negative witness {}", p.display());
    let (lowerable, json) = run_icarus_lowerable(&p);
    assert!(
        !lowerable,
        "expected {} to be rejected because Pt is not declared, got: {}",
        p.display(),
        json
    );
}

#[test]
fn rejects_w543_nonlowerable_call_init_witness() {
    let dir = scratch_dir();
    let p = dir.join("w543_negative_nonlowerable_call_init.t27");
    assert!(p.exists(), "missing W543 negative witness {}", p.display());
    let (lowerable, json) = run_icarus_lowerable(&p);
    assert!(
        !lowerable,
        "expected {} to be rejected because String is not lowerable, got: {}",
        p.display(),
        json
    );
}

#[test]
fn accepts_known_lowerable_witnesses() {
    let dir = scratch_dir();
    let positive = [
        "w532_signed_struct_array_field_2d_copy.t27",
        "w533_module_scalar_struct_return.t27",
        "w528_function_2d_struct_array_param.t27",
        "w543_module_scalar_call_init.t27",
        "w543_module_struct_call_init.t27",
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

/// W537 regression: every corpus spec in `Trinity.IcarusLowerable.Completeness`
/// must have a theorem whose verdict matches the Rust structural classifier.
#[test]
fn corpus_classifier_matches_lean_completeness() {
    let repo = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    let completeness = repo.join("proofs/lean4/Trinity/IcarusLowerable/Completeness.lean");
    let text = std::fs::read_to_string(&completeness)
        .expect("failed to read Completeness.lean");

    // theorem foo_lowerable : Module.isLowerable foo_env foo_module = true := by native_decide
    let theorem_re = regex::Regex::new(
        r"theorem\s+(\w+)_lowerable\s*:\s*Module\.isLowerable\s+(\w+)_env\s+(\w+)_module\s*=\s*(true|false)\s*:=\s*by\s+native_decide",
    )
    .unwrap();

    // Build env-name -> spec-path map the same way the Completeness.lean envs are named.
    let specs_dir = repo.join("specs");
    let mut env_to_spec: std::collections::HashMap<String, PathBuf> =
        std::collections::HashMap::new();
    for entry in walkdir::WalkDir::new(&specs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("t27") {
            continue;
        }
        let rel = p.strip_prefix(&specs_dir).expect("spec under specs/");
        let env_name = rel
            .with_extension("")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "_");
        env_to_spec.insert(env_name, p.to_path_buf());
    }

    let mut checked = 0usize;
    let mut missing_specs = Vec::new();
    for cap in theorem_re.captures_iter(&text) {
        let theorem_name = cap[1].to_string();
        let env_name = cap[2].to_string();
        let module_name = cap[3].to_string();
        assert_eq!(
            theorem_name, env_name,
            "theorem/env name mismatch: {} vs {}",
            theorem_name, env_name
        );
        assert_eq!(
            env_name, module_name,
            "env/module name mismatch: {} vs {}",
            env_name, module_name
        );
        let lean_verdict = &cap[4] == "true";
        let Some(spec) = env_to_spec.get(&env_name) else {
            missing_specs.push(env_name);
            continue;
        };
        let (rust_verdict, json) = run_icarus_lowerable(spec);
        assert_eq!(
            rust_verdict, lean_verdict,
            "Rust/Lean lowerability mismatch for {}: Rust={}, Lean theorem={}\n{}",
            spec.display(), rust_verdict, lean_verdict, json
        );
        checked += 1;
    }

    // A handful of envs are Lean-only formal witnesses with no matching .t27 file.
    let expected_missing = [
        "automation_wrapup_auto",
        "igla_w521_2d_aos_param_soundness",
        "igla_w524_2d_packed_aos_param_module",
        "physics_gamma_conflict",
    ];
    for name in &expected_missing {
        assert!(
            missing_specs.contains(&name.to_string()),
            "expected {} to be a Lean-only witness without a matching spec",
            name
        );
    }
    assert_eq!(
        missing_specs.len(),
        expected_missing.len(),
        "unexpected envs without matching specs: {:?}",
        missing_specs
    );

    assert!(
        checked >= 245,
        "expected at least 245 corpus agreement checks, got {}",
        checked
    );
}

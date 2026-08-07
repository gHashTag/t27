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
fn rejects_w561_nonlowerable_struct_return_witnesses() {
    let dir = scratch_dir();
    let mut witnesses: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("failed to read specs/scratch")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("w561_negative_struct_return_"))
                .unwrap_or(false)
        })
        .collect();
    witnesses.sort();
    assert!(
        !witnesses.is_empty(),
        "expected W561 negative struct-return witnesses in specs/scratch"
    );
    for p in &witnesses {
        let (lowerable, json) = run_icarus_lowerable(p);
        assert!(
            !lowerable,
            "expected {} to be rejected because its return struct is not lowerable, got: {}",
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
fn accepts_w545_primitive_scalar_array_return() {
    let dir = scratch_dir();
    for name in &[
        "w545_call_init_returns_array.t27",
        "w545_var_call_init_returns_array.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W545 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w547_signed_primitive_scalar_array_return() {
    let dir = scratch_dir();
    for name in &[
        "w547_signed_call_init_returns_array.t27",
        "w547_signed_element_compare.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W547 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w548_multi_dimensional_primitive_scalar_array_return() {
    let dir = scratch_dir();
    for name in &[
        "w548_2d_call_init_returns_array.t27",
        "w548_2d_signed_element_read.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W548 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w549_three_dimensional_primitive_scalar_array_return() {
    let dir = scratch_dir();
    for name in &[
        "w549_3d_call_init_returns_array.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W549 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w550_four_dimensional_primitive_scalar_array_return() {
    let dir = scratch_dir();
    for name in &[
        "w550_4d_call_init_returns_array.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W550 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w551_bench_block_cross_check() {
    let dir = scratch_dir();
    for name in &[
        "w551_bench_scalar_call_cross_check.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W551 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w552_bench_wide_cross_check() {
    let dir = scratch_dir();
    for name in &[
        "w552_bench_wide_packed_struct.t27",
        "w552_bench_module_wide_struct.t27",
        "w552_bench_2d_array_return.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W552 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w553_bench_signed_cross_check() {
    let dir = scratch_dir();
    for name in &[
        "w553_bench_signed_scalar_return.t27",
        "w553_bench_signed_struct_field.t27",
        "w553_bench_signed_array_element.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W553 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w554_bench_local_array_cross_check() {
    let dir = scratch_dir();
    for name in &[
        "w554_bench_local_array_unsigned.t27",
        "w554_bench_local_array_signed.t27",
        "w554_bench_local_array_2d.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W554 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w555_bench_whole_array_cross_check() {
    let dir = scratch_dir();
    for name in &[
        "w555_bench_whole_array_unsigned.t27",
        "w555_bench_whole_array_signed.t27",
        "w555_bench_whole_array_nested_call.t27",
        "w555_bench_whole_array_wide.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W555 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w556_bench_multi_site_array_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w556_bench_multi_site_array_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W556 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w557_bench_scalar_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w557_bench_scalar_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W557 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w558_bench_scalar_call_expected_side_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w558_bench_scalar_call_expected_side_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W558 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w559_bench_whole_array_higher_rank_signed() {
    let dir = scratch_dir();
    for name in &[
        "w559_bench_whole_array_3d_signed.t27",
        "w559_bench_whole_array_3d_signed_direct_call.t27",
        "w559_bench_whole_array_4d_signed.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W559 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w560_bench_scalar_struct_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w560_bench_scalar_struct_call_dedup.t27",
        "w560_bench_scalar_struct_call_dedup_both_sides.t27",
        "w560_bench_scalar_struct_call_dedup_nested.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W560 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w562_bench_struct_array_field() {
    let dir = scratch_dir();
    for name in &[
        "w562_bench_struct_array_field.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W562 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w563_bench_array_of_struct_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w563_bench_array_of_struct_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W563 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w564_bench_whole_aos_1d() {
    let dir = scratch_dir();
    for name in &[
        "w564_bench_whole_aos_1d.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W564 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w565_bench_multi_site_whole_aos() {
    let dir = scratch_dir();
    for name in &[
        "w565_bench_multi_site_whole_aos.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W565 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w566_bench_2d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w566_bench_2d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W566 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w567_bench_3d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w567_bench_3d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W567 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w568_bench_4d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w568_bench_4d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W568 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w569_bench_4d_aos_call_dedup_nonp2() {
    let dir = scratch_dir();
    for name in &[
        "w569_bench_4d_aos_call_dedup_nonp2.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W569 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w570_bench_5d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w570_bench_5d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W570 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w571_bench_5d_aos_call_dedup_nonp2() {
    let dir = scratch_dir();
    for name in &[
        "w571_bench_5d_aos_call_dedup_nonp2.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W571 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w572_bench_6d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w572_bench_6d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W572 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w573_bench_7d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w573_bench_7d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W573 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w574_bench_8d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w574_bench_8d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W574 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w575_bench_9d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w575_bench_9d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W575 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w576_bench_10d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w576_bench_10d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W576 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w577_bench_11d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w577_bench_11d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W577 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w578_bench_12d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w578_bench_12d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W578 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w579_bench_13d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w579_bench_13d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W579 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w580_bench_14d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w580_bench_14d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W580 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w581_bench_15d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w581_bench_15d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W581 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w582_bench_16d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w582_bench_16d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W582 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w583_bench_module_3d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w583_bench_module_3d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W583 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w584_bench_17d_aos_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w584_bench_17d_aos_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W584 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w585_bench_module_7d_aos_var_call_dedup() {
    let dir = scratch_dir();
    for name in &[
        "w585_bench_module_7d_aos_var_call_dedup.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W585 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w586_bench_module_8d_aos_var_write() {
    let dir = scratch_dir();
    for name in &[
        "w586_bench_module_8d_aos_var_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W586 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w587_bench_module_8d_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w587_bench_module_8d_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W587 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w588_bench_module_9d_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w588_bench_module_9d_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W588 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w589_bench_module_17d_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w589_bench_module_17d_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W589 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w590_bench_module_17d_aos_var_call_reassign() {
    let dir = scratch_dir();
    for name in &[
        "w590_bench_module_17d_aos_var_call_reassign.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W590 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w591_bench_module_17d_aos_var_literal_reassign() {
    let dir = scratch_dir();
    for name in &[
        "w591_bench_module_17d_aos_var_literal_reassign.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W591 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w592_bench_module_3x2p15_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w592_bench_module_3x2p15_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W592 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w593_bench_module_5x2p15_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w593_bench_module_5x2p15_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W593 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w594_bench_module_7x2p14_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w594_bench_module_7x2p14_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W594 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w595_bench_module_9x2p13_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w595_bench_module_9x2p13_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W595 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w596_bench_module_11x2p12_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w596_bench_module_11x2p12_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W596 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w597_bench_module_13x2p11_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w597_bench_module_13x2p11_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W597 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w598_bench_module_15x2p10_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w598_bench_module_15x2p10_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W598 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w599_bench_module_17x2p9_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w599_bench_module_17x2p9_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W599 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w600_bench_module_19x2p8_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w600_bench_module_19x2p8_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W600 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w601_bench_module_21x2p7_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w601_bench_module_21x2p7_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W601 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w602_bench_module_23x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w602_bench_module_23x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W602 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w603_bench_module_25x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w603_bench_module_25x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W603 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w604_bench_module_27x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w604_bench_module_27x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W604 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w605_bench_module_29x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w605_bench_module_29x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W605 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w606_bench_module_31x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w606_bench_module_31x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W606 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w607_bench_module_33x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w607_bench_module_33x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W607 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w608_bench_module_35x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w608_bench_module_35x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W608 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w609_bench_module_37x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w609_bench_module_37x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W609 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w610_bench_module_39x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w610_bench_module_39x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W610 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w611_bench_module_41x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w611_bench_module_41x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W611 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w612_bench_module_43x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w612_bench_module_43x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W612 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w613_bench_module_45x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w613_bench_module_45x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W613 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w614_bench_module_47x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w614_bench_module_47x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W614 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w615_bench_module_49x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w615_bench_module_49x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W615 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w616_bench_module_51x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w616_bench_module_51x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W616 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w617_bench_module_53x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w617_bench_module_53x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W617 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w618_bench_module_55x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w618_bench_module_55x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W618 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w619_bench_module_57x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w619_bench_module_57x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W619 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w620_bench_module_59x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w620_bench_module_59x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W620 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w621_bench_module_61x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w621_bench_module_61x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W621 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w622_bench_module_63x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w622_bench_module_63x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W622 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w623_bench_module_65x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w623_bench_module_65x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W623 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w624_bench_module_67x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w624_bench_module_67x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W624 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w625_bench_module_69x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w625_bench_module_69x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W625 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w626_bench_module_71x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w626_bench_module_71x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W626 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w627_bench_module_73x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w627_bench_module_73x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W627 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w628_bench_module_75x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w628_bench_module_75x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W628 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w629_bench_module_77x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w629_bench_module_77x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W629 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w630_bench_module_79x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w630_bench_module_79x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W630 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w631_bench_module_81x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w631_bench_module_81x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W631 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w632_bench_module_83x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w632_bench_module_83x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W632 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w633_bench_module_85x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w633_bench_module_85x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W633 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w634_bench_module_87x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w634_bench_module_87x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W634 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w635_bench_module_89x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w635_bench_module_89x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W635 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w636_bench_module_91x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w636_bench_module_91x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W636 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w637_bench_module_93x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w637_bench_module_93x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W637 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w638_bench_module_95x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w638_bench_module_95x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W638 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w639_bench_module_97x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w639_bench_module_97x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W639 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w640_bench_module_99x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w640_bench_module_99x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W640 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w641_bench_module_101x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w641_bench_module_101x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W641 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w642_bench_module_103x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w642_bench_module_103x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W642 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w643_bench_module_105x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w643_bench_module_105x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W643 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w644_bench_module_107x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w644_bench_module_107x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W644 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w645_bench_module_109x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w645_bench_module_109x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W645 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w646_bench_module_111x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w646_bench_module_111x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W646 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w647_bench_module_113x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w647_bench_module_113x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W647 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w648_bench_module_115x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w648_bench_module_115x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W648 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w649_bench_module_117x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w649_bench_module_117x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W649 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w650_bench_module_119x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w650_bench_module_119x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W650 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w651_bench_module_121x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w651_bench_module_121x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W651 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w652_bench_module_123x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w652_bench_module_123x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W652 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w653_bench_module_125x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w653_bench_module_125x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W653 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w654_bench_module_127x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w654_bench_module_127x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W654 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w655_bench_module_129x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w655_bench_module_129x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W655 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w656_bench_module_131x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w656_bench_module_131x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W656 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w657_bench_module_133x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w657_bench_module_133x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W657 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w658_bench_module_135x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w658_bench_module_135x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W658 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w659_bench_module_137x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w659_bench_module_137x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W659 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w660_bench_module_139x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w660_bench_module_139x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W660 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w661_bench_module_141x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w661_bench_module_141x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W661 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w662_bench_module_143x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w662_bench_module_143x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W662 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w663_bench_module_145x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w663_bench_module_145x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W663 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w664_bench_module_147x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w664_bench_module_147x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W664 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w665_bench_module_149x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w665_bench_module_149x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W665 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w666_bench_module_151x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w666_bench_module_151x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W666 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w667_bench_module_153x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w667_bench_module_153x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W667 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w668_bench_module_155x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w668_bench_module_155x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W668 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w669_bench_module_157x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w669_bench_module_157x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W669 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w670_bench_module_159x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w670_bench_module_159x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W670 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w671_bench_module_161x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w671_bench_module_161x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W671 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w672_bench_module_163x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w672_bench_module_163x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W672 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w673_bench_module_165x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w673_bench_module_165x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W673 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w674_bench_module_167x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w674_bench_module_167x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W674 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w675_bench_module_169x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w675_bench_module_169x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W675 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w676_bench_module_171x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w676_bench_module_171x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W676 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w677_bench_module_173x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w677_bench_module_173x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W677 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w678_bench_module_175x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w678_bench_module_175x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W678 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w679_bench_module_177x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w679_bench_module_177x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W679 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w680_bench_module_179x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w680_bench_module_179x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W680 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w681_bench_module_181x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w681_bench_module_181x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W681 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w682_bench_module_183x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w682_bench_module_183x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W682 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w683_bench_module_185x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w683_bench_module_185x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W683 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w684_bench_module_187x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w684_bench_module_187x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W684 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w685_bench_module_189x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w685_bench_module_189x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W685 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w686_bench_module_191x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w686_bench_module_191x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W686 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w687_bench_module_193x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w687_bench_module_193x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W687 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w688_bench_module_195x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w688_bench_module_195x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W688 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w689_bench_module_197x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w689_bench_module_197x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W689 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w690_bench_module_199x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w690_bench_module_199x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W690 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w691_bench_module_201x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w691_bench_module_201x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W691 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w692_bench_module_203x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w692_bench_module_203x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W692 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w693_bench_module_205x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w693_bench_module_205x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W693 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w694_bench_module_207x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w694_bench_module_207x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W694 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w695_bench_module_209x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w695_bench_module_209x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W695 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w696_bench_module_211x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w696_bench_module_211x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W696 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w697_bench_module_213x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w697_bench_module_213x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W697 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w698_bench_module_215x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w698_bench_module_215x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W698 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w699_bench_module_217x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w699_bench_module_217x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W699 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w700_bench_module_219x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w700_bench_module_219x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W700 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w701_bench_module_221x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w701_bench_module_221x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W701 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w702_bench_module_223x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w702_bench_module_223x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W702 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w703_bench_module_225x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w703_bench_module_225x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W703 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w704_bench_module_227x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w704_bench_module_227x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W704 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w705_bench_module_229x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w705_bench_module_229x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W705 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w706_bench_module_231x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w706_bench_module_231x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W706 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w707_bench_module_233x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w707_bench_module_233x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W707 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w708_bench_module_235x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w708_bench_module_235x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W708 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w709_bench_module_237x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w709_bench_module_237x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W709 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w710_bench_module_239x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w710_bench_module_239x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W710 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w711_bench_module_241x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w711_bench_module_241x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W711 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w712_bench_module_243x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w712_bench_module_243x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W712 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w713_bench_module_245x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w713_bench_module_245x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W713 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w714_bench_module_247x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w714_bench_module_247x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W714 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w715_bench_module_249x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w715_bench_module_249x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W715 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w716_bench_module_251x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w716_bench_module_251x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W716 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w717_bench_module_253x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w717_bench_module_253x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W717 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w718_bench_module_255x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w718_bench_module_255x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W718 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w719_bench_module_257x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w719_bench_module_257x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W719 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w720_bench_module_259x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w720_bench_module_259x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W720 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w721_bench_module_261x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w721_bench_module_261x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W721 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w722_bench_module_263x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w722_bench_module_263x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W722 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w723_bench_module_265x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w723_bench_module_265x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W723 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w724_bench_module_267x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w724_bench_module_267x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W724 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w725_bench_module_269x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w725_bench_module_269x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W725 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w726_bench_module_271x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w726_bench_module_271x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W726 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w727_bench_module_273x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w727_bench_module_273x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W727 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w728_bench_module_275x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w728_bench_module_275x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W728 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w729_bench_module_277x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w729_bench_module_277x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W729 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w730_bench_module_279x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w730_bench_module_279x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W730 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w731_bench_module_281x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w731_bench_module_281x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W731 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w732_bench_module_283x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w732_bench_module_283x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W732 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w733_bench_module_285x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w733_bench_module_285x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W733 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w734_bench_module_287x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w734_bench_module_287x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W734 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w735_bench_module_289x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w735_bench_module_289x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W735 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w736_bench_module_291x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w736_bench_module_291x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W736 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w737_bench_module_293x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w737_bench_module_293x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W737 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w738_bench_module_295x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w738_bench_module_295x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W738 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w739_bench_module_297x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w739_bench_module_297x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W739 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w740_bench_module_299x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w740_bench_module_299x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W740 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w741_bench_module_301x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w741_bench_module_301x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W741 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w742_bench_module_303x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w742_bench_module_303x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W742 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w743_bench_module_305x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w743_bench_module_305x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W743 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w744_bench_module_307x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w744_bench_module_307x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W744 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w745_bench_module_309x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w745_bench_module_309x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W745 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w746_bench_module_311x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w746_bench_module_311x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W746 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w747_bench_module_313x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w747_bench_module_313x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W747 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w748_bench_module_315x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w748_bench_module_315x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W748 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w749_bench_module_317x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w749_bench_module_317x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W749 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w750_bench_module_319x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w750_bench_module_319x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W750 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w751_bench_module_321x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w751_bench_module_321x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W751 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w752_bench_module_323x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w752_bench_module_323x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W752 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w753_bench_module_325x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w753_bench_module_325x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W753 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w754_bench_module_327x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w754_bench_module_327x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W754 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w755_bench_module_329x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w755_bench_module_329x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W755 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w756_bench_module_331x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w756_bench_module_331x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W756 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w757_bench_module_333x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w757_bench_module_333x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W757 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w758_bench_module_335x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w758_bench_module_335x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W758 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w759_bench_module_337x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w759_bench_module_337x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W759 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w760_bench_module_339x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w760_bench_module_339x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W760 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w761_bench_module_341x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w761_bench_module_341x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W761 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w762_bench_module_343x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w762_bench_module_343x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W762 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w763_bench_module_345x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w763_bench_module_345x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W763 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w764_bench_module_347x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w764_bench_module_347x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W764 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w765_bench_module_349x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w765_bench_module_349x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W765 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w766_bench_module_351x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w766_bench_module_351x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W766 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w767_bench_module_353x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w767_bench_module_353x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W767 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w768_bench_module_355x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w768_bench_module_355x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W768 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w769_bench_module_357x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w769_bench_module_357x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W769 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w770_bench_module_359x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w770_bench_module_359x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W770 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w771_bench_module_361x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w771_bench_module_361x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W771 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w772_bench_module_363x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w772_bench_module_363x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W772 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w773_bench_module_365x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w773_bench_module_365x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W773 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w774_bench_module_367x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w774_bench_module_367x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W774 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w775_bench_module_369x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w775_bench_module_369x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W775 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w776_bench_module_371x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w776_bench_module_371x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W776 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w777_bench_module_373x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w777_bench_module_373x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W777 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w778_bench_module_375x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w778_bench_module_375x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W778 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w779_bench_module_377x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w779_bench_module_377x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W779 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w780_bench_module_379x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w780_bench_module_379x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W780 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w781_bench_module_381x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w781_bench_module_381x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W781 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w782_bench_module_383x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w782_bench_module_383x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W782 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w783_bench_module_385x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w783_bench_module_385x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W783 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w784_bench_module_387x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w784_bench_module_387x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W784 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w785_bench_module_389x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w785_bench_module_389x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W785 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w786_bench_module_391x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w786_bench_module_391x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W786 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w787_bench_module_393x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w787_bench_module_393x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W787 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w788_bench_module_395x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w788_bench_module_395x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W788 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w789_bench_module_397x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w789_bench_module_397x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W789 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w790_bench_module_399x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w790_bench_module_399x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W790 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w791_bench_module_401x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w791_bench_module_401x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W791 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w792_bench_module_403x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w792_bench_module_403x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W792 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w793_bench_module_405x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w793_bench_module_405x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W793 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w794_bench_module_407x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w794_bench_module_407x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W794 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w795_bench_module_409x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w795_bench_module_409x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W795 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w796_bench_module_411x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w796_bench_module_411x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W796 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w797_bench_module_413x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w797_bench_module_413x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W797 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w798_bench_module_415x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w798_bench_module_415x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W798 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w799_bench_module_417x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w799_bench_module_417x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W799 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w800_bench_module_419x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w800_bench_module_419x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W800 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w801_bench_module_421x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w801_bench_module_421x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W801 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w802_bench_module_423x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w802_bench_module_423x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W802 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w803_bench_module_425x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w803_bench_module_425x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W803 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w804_bench_module_427x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w804_bench_module_427x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W804 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w805_bench_module_429x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w805_bench_module_429x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W805 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w806_bench_module_431x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w806_bench_module_431x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W806 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w807_bench_module_433x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w807_bench_module_433x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W807 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w808_bench_module_435x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w808_bench_module_435x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W808 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w809_bench_module_437x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w809_bench_module_437x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W809 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w810_bench_module_439x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w810_bench_module_439x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W810 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w811_bench_module_441x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w811_bench_module_441x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W811 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w812_bench_module_443x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w812_bench_module_443x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W812 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w813_bench_module_445x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w813_bench_module_445x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W813 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w814_bench_module_447x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w814_bench_module_447x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W814 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w815_bench_module_449x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w815_bench_module_449x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W815 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w816_bench_module_451x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w816_bench_module_451x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W816 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w817_bench_module_453x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w817_bench_module_453x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W817 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w818_bench_module_455x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w818_bench_module_455x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W818 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w819_bench_module_457x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w819_bench_module_457x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W819 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w820_bench_module_459x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w820_bench_module_459x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W820 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w821_bench_module_461x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w821_bench_module_461x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W821 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w822_bench_module_463x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w822_bench_module_463x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W822 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w823_bench_module_465x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w823_bench_module_465x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W823 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w824_bench_module_467x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w824_bench_module_467x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W824 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w825_bench_module_469x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w825_bench_module_469x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W825 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w826_bench_module_471x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w826_bench_module_471x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W826 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w827_bench_module_473x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w827_bench_module_473x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W827 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w828_bench_module_475x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w828_bench_module_475x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W828 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w829_bench_module_477x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w829_bench_module_477x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W829 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w830_bench_module_479x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w830_bench_module_479x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W830 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w831_bench_module_481x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w831_bench_module_481x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W831 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w832_bench_module_483x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w832_bench_module_483x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W832 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w833_bench_module_485x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w833_bench_module_485x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W833 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w834_bench_module_487x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w834_bench_module_487x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W834 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w835_bench_module_489x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w835_bench_module_489x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W835 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w836_bench_module_491x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w836_bench_module_491x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W836 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w837_bench_module_493x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w837_bench_module_493x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W837 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w838_bench_module_495x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w838_bench_module_495x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W838 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w839_bench_module_497x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w839_bench_module_497x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W839 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w840_bench_module_499x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w840_bench_module_499x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W840 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w841_bench_module_501x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w841_bench_module_501x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W841 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w842_bench_module_503x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w842_bench_module_503x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W842 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w843_bench_module_505x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w843_bench_module_505x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W843 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w844_bench_module_507x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w844_bench_module_507x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W844 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w845_bench_module_509x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w845_bench_module_509x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W845 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w846_bench_module_511x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w846_bench_module_511x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W846 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w847_bench_module_513x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w847_bench_module_513x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W847 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w848_bench_module_515x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w848_bench_module_515x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W848 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w849_bench_module_517x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w849_bench_module_517x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W849 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w850_bench_module_519x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w850_bench_module_519x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W850 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w851_bench_module_521x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w851_bench_module_521x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W851 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w852_bench_module_523x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w852_bench_module_523x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W852 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w853_bench_module_525x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w853_bench_module_525x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W853 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w854_bench_module_527x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w854_bench_module_527x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W854 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w855_bench_module_529x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w855_bench_module_529x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W855 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w856_bench_module_531x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w856_bench_module_531x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W856 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w857_bench_module_533x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w857_bench_module_533x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W857 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w858_bench_module_535x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w858_bench_module_535x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W858 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w859_bench_module_537x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w859_bench_module_537x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W859 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w860_bench_module_539x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w860_bench_module_539x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W860 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w861_bench_module_541x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w861_bench_module_541x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W861 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w862_bench_module_543x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w862_bench_module_543x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W862 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w863_bench_module_545x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w863_bench_module_545x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W863 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w864_bench_module_547x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w864_bench_module_547x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W864 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w865_bench_module_549x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w865_bench_module_549x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W865 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w866_bench_module_551x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w866_bench_module_551x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W866 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w867_bench_module_553x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w867_bench_module_553x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W867 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w868_bench_module_555x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w868_bench_module_555x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W868 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w869_bench_module_557x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w869_bench_module_557x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W869 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w870_bench_module_559x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w870_bench_module_559x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W870 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w871_bench_module_561x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w871_bench_module_561x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W871 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w872_bench_module_563x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w872_bench_module_563x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W872 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w873_bench_module_565x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w873_bench_module_565x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W873 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w874_bench_module_567x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w874_bench_module_567x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W874 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w875_bench_module_569x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w875_bench_module_569x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W875 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w876_bench_module_571x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w876_bench_module_571x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W876 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w877_bench_module_573x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w877_bench_module_573x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W877 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w878_bench_module_575x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w878_bench_module_575x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W878 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w879_bench_module_577x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w879_bench_module_577x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W879 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w880_bench_module_579x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w880_bench_module_579x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W880 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w881_bench_module_581x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w881_bench_module_581x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W881 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w882_bench_module_583x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w882_bench_module_583x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W882 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w883_bench_module_585x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w883_bench_module_585x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W883 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w884_bench_module_587x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w884_bench_module_587x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W884 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w885_bench_module_589x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w885_bench_module_589x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W885 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w886_bench_module_591x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w886_bench_module_591x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W886 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w887_bench_module_593x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w887_bench_module_593x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W887 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w888_bench_module_595x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w888_bench_module_595x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W888 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w889_bench_module_597x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w889_bench_module_597x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W889 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w890_bench_module_599x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w890_bench_module_599x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W890 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w891_bench_module_601x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w891_bench_module_601x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W891 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w892_bench_module_603x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w892_bench_module_603x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W892 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w893_bench_module_605x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w893_bench_module_605x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W893 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w894_bench_module_607x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w894_bench_module_607x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W894 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w895_bench_module_609x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w895_bench_module_609x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W895 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
            p.display(),
            json
        );
    }
}

#[test]
fn accepts_w896_bench_module_611x2p6_aos_var_call_write() {
    let dir = scratch_dir();
    for name in &[
        "w896_bench_module_611x2p6_aos_var_call_write.t27",
    ] {
        let p = dir.join(name);
        assert!(p.exists(), "missing W896 witness {}", p.display());
        let (lowerable, json) = run_icarus_lowerable(&p);
        assert!(
            lowerable,
            "expected {} to be lowerable, got: {}",
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
        "w543_module_scalar_call_init.t27",
        "w543_module_struct_call_init.t27",
        "w544_module_var_scalar_call_init.t27",
        "w544_module_var_struct_call_assign.t27",
        "w544_nested_call_init.t27",
        "w544_call_init_depends_on_const.t27",
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

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

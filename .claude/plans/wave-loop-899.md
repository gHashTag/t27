# Wave Loop 899 Plan

**Issue:** #1901 — feat(igla): Wave Loop 899 — module-scope `[617][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes
**Branch:** `wave-loop-899`
**Previous:** Wave Loop 898 (#1859, PR #1900)

---

## Goal

Add the next mechanical rung of the packed-vector array-of-struct ladder:
- Module-scope `[617][2]^6 Pt` variable initialized from a pure `make_grid(0)` call.
- Indexed signed field writes and `assert_eq` read-back inside a `bench` block.
- 39,488 elements, 1,263,616-bit packed vector, ~1.206 MiBit.
- Zero compiler / reference-model / `FROZEN_HASH` changes expected.

---

## File Checklist

- [ ] Copy `scripts/gen_w898.py` → `scripts/gen_w899.py`
  - Update `OUTER = 617`
  - `MID_IDX = 617 // 2` → 308
  - Update `DST` path to `specs/scratch/w899_bench_module_617x2p6_aos_var_call_write.t27`
  - Update header f-string to `module w899_bench_module_617x2p6_aos_var_call_write`
- [ ] Run `python3 scripts/gen_w899.py`
- [ ] Grep generator + spec for stale `615`, `307`, `w898` references
- [ ] `t27c parse specs/scratch/w899_bench_module_617x2p6_aos_var_call_write.t27` → PASS
- [ ] `t27c icarus-lowerable ...` → `lowerable`
- [ ] `t27c icarus-simulate ...` → `PASSED`
- [ ] `t27c icarus-cocotb ...` → reference-model OK
- [ ] `t27c seal --save ...` + `seal --verify ...` → MATCH
- [ ] Add `accepts_w899_bench_module_617x2p6_aos_var_call_write` test to `bootstrap/tests/icarus_lowerable.rs`
- [ ] `cargo test --release --test icarus_lowerable accepts_w899_bench_module_617x2p6_aos_var_call_write` → PASS
- [ ] `cargo test --release --test icarus_lowerable` → expected 358 passed / 1 pre-existing failure
- [ ] Commit with `Closes #1901`
- [ ] Push branch, open PR, enable auto-merge
- [ ] Update `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`, `.claude/skills/`, persistent memory

---

## Risk Notes

- Disk space: cocotb temporary directories can grow large; clean old `/tmp/t27c_cocotb_w*` directories if `ENOSPC` recurs.
- Pre-existing failure: `corpus_classifier_matches_lean_completeness` for `specs/cloud/railway_deploy.t27` is unrelated; do not block on it.
- Copy hazard: previous waves showed stale `OUTER`/`MID_IDX` in header, path, and `MID_IDX` comment; always grep after copy.

phi^2 + 1/phi^2 = 3 | TRINITY

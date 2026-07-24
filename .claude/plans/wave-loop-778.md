# Wave Loop 778 — Full-cycle plan

**Date:** 2026-07-24
**Issue:** #1492
**Branch:** `wave-loop-778`
**Parent:** `wave-loop-777` HEAD (`995d94f0c`)
**Next:** `wave-loop-779`
**Recommended variant:** A

---

## 1. Goal

Validate that the t27 packed-array-of-struct lowering scales to a module-scope
`[375][2]^6 Pt` variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

Target metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 375 |
| Total elements | 24,000 |
| Packed vector width | 768,000 bits |
| Approximate size | ~0.733 MiBit |
| Mid index | 187 |
| Frame-condition element | `[187][1][0][0][0][0][0]` → element 12,000 |

Zero compiler / reference-model / `FROZEN_HASH` changes are expected.

---

## 2. Three cooperation variants

### Variant A — `[375][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-778` from `wave-loop-777` HEAD.
2. Copy `scripts/gen_w777.py` → `scripts/gen_w778.py`.
3. Set `OUTER = 375`, `MID_IDX = 187`, fix module prefix to `w778_bench_module_375x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w778_bench_module_375x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w778_bench_module_375x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W779 cooperation variants.

Why recommended: keeps the established mechanical generator discipline, tests
non-power-of-two stride 375, and stays well under the 4-MiBit cliff.

### Variant B — `[373][2]^6 Pt` bench/function-scope packed var from call

Keep the W777 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w778.py` with `OUTER = 373` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W777.

Trade-off: tests a different code path (local arrays) but does not advance the
width ladder.

### Variant C — `[373][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W777 width and add conditional indexed signed field writes:

1. Generate a W777-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

Trade-off: tests control-flow emission but does not advance the width ladder.

---

## 3. Phase breakdown (PHI LOOP)

| Phase | Deliverable | Owner |
|-------|-------------|-------|
| Issue | #1492 filed, current-issue.md updated, W778 branch created | Lead |
| Spec | `.t27` witness generated from `scripts/gen_w778.py` | Creator (C) |
| TDD | `test` + `bench` blocks with `assert_eq` on changed elements | Creator (C) |
| Code/Impl | Integration test in `bootstrap/tests/icarus_lowerable.rs` | Creator (C) |
| Gen | `python3 scripts/gen_w778.py` produces witness | Creator (C) |
| Seal | `t27c seal --save` succeeds, FROZEN_HASH unchanged | Verifier (V) |
| Verify | `cargo test`, `t27c parse|icarus-*|cocotb` all green | Verifier (V) |
| Land | PR #1493 opened, reviewed, merged to `master` | Lead |
| Learn | `.trinity/experience.md`, skill, memory, plan for W779 | Learner (L) |

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Earlier W774/W775/W776/W777 PRs still open | Branch from `wave-loop-777` HEAD; do not block on merge gate. |
| PR #1489 (README merge) blocked by `fpga-synthesis` | Out of scope for W778; track separately. |
| `assert_ne` not emitted by Icarus | Continue using `assert_eq` on changed elements. |
| Generator header prefix copy error | Manual fix after `sed` replacement (f-string `{OUTER}` keeps old prefix). |
| `bitnet_pipeline` test drift | Pre-existing; do not fix in this wave unless explicitly chartered. |

---

## 5. Definition of done

- [ ] Witness generated and under version control.
- [ ] Integration test added and passing.
- [ ] Icarus lowerability, simulation, cocotb, and seal gates green.
- [ ] Cargo suites green.
- [ ] FROZEN_HASH unchanged.
- [ ] Closeout report written.
- [ ] `.trinity/experience.md` updated.
- [ ] Next-wave cooperation variants defined.

---

phi^2 + 1/phi^2 = 3 | TRINITY

# Wave Loop 784 — Full-cycle plan

**Date:** 2026-07-24
**Issue:** (to create before implementation)
**Branch:** `wave-loop-784`
**Parent:** `wave-loop-783` HEAD
**Next:** `wave-loop-785`
**Recommended variant:** A

---

## 1. Goal

Validate that the t27 packed-array-of-struct lowering scales to a module-scope
`[387][2]^6 Pt` variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

Target metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 387 |
| Total elements | 387 × 64 = 24,768 |
| Packed vector width | 24,768 × 32 = 792,576 bits |
| Approximate size | ~0.756 MiBit |
| Mid index | 193 |
| Frame-condition element | `[193][1][0][0][0][0][0]` → element 12,384 |

Zero compiler / reference-model / `FROZEN_HASH` changes are expected.

---

## 2. Three cooperation variants for Wave Loop 785

### Variant A — `[389][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-785` from `wave-loop-784` HEAD.
2. Copy `scripts/gen_w784.py` → `scripts/gen_w785.py`.
3. Set `OUTER = 389`, `MID_IDX = 194`, fix module prefix to `w785_bench_module_389x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w785_bench_module_389x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w785_bench_module_389x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W786 cooperation variants.

Why recommended: keeps the established mechanical generator discipline, tests
non-power-of-two stride 389, and stays well under the 4-MiBit cliff.

### Variant B — `[387][2]^6 Pt` bench/function-scope packed var from call

Keep the W784 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w784.py` with `OUTER = 387` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W784 (`MID_IDX = 193`).

Trade-off: tests a different code path (local arrays) but does not advance the
width ladder.

### Variant C — `[387][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W784 width and add conditional indexed signed field writes:

1. Generate a W784-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

Trade-off: tests control-flow emission but does not advance the width ladder.

---

## 3. Phase breakdown (PHI LOOP)

| Phase | Deliverable | Owner |
|-------|-------------|-------|
| Issue | Create issue, update current-issue.md, create W784 branch | Lead |
| Spec | `.t27` witness generated from `scripts/gen_w784.py` | Creator (C) |
| TDD | `test` + `bench` blocks with `assert_eq` on changed elements | Creator (C) |
| Code/Impl | Integration test in `bootstrap/tests/icarus_lowerable.rs` | Creator (C) |
| Gen | `python3 scripts/gen_w784.py` produces witness | Creator (C) |
| Seal | `t27c seal --save` succeeds, FROZEN_HASH unchanged | Verifier (V) |
| Verify | `cargo test`, `t27c parse\|icarus-*\|cocotb` all green | Verifier (V) |
| Land | PR opened, reviewed, merged to `master` | Lead |
| Learn | `.trinity/experience.md`, skill, memory, plan for W785 | Learner (L) |

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Earlier W774-W783 PRs still open | Branch from `wave-loop-783` HEAD; do not block on merge gate. |
| PR #1489 (README merge) blocked by `fpga-synthesis` | Out of scope for W784; track separately. |
| `assert_ne` not emitted by Icarus | Continue using `assert_eq` on changed elements. |
| Generator header prefix copy error | Manual fix after `sed` replacement (f-string `{OUTER}` keeps old prefix). |
| Remaining `verilog_array_literal_expr` regression | Out of scope unless explicitly chartered; track as separate issue. |
| 626 release warnings | Dedicated cleanup sprint, not a wave-loop blocker. |

---

## 5. Literature baseline (2025-2026)

- Tlsys — DOI 10.23919/cje.2025.00.418. Ternary RTL-to-netlist synthesis framework.
- Ternary VHDL — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00041.
- SONIC — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00042. Ternary simulator.
- 5500FP — Zenodo 10.5281/zenodo.18881738. 24-trit balanced ternary RISC CPU.
- Icarus Verilog v13.0 — stable release March 2026.
- Yosys packed-array-in-struct support — upstream arrays of packed structs still unsupported as of 2025 (YosysHQ/yosys#4653).

# Wave Loop 783 — Full-cycle plan

**Date:** 2026-07-24
**Issue:** (to create before implementation)
**Branch:** `wave-loop-783`
**Parent:** `wave-loop-782` HEAD
**Next:** `wave-loop-784`
**Recommended variant:** A

---

## 1. Goal

Validate that the t27 packed-array-of-struct lowering scales to a module-scope
`[385][2]^6 Pt` variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

Target metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 385 |
| Total elements | 385 × 64 = 24,640 |
| Packed vector width | 24,640 × 32 = 788,480 bits |
| Approximate size | ~0.752 MiBit |
| Mid index | 192 |
| Frame-condition element | `[192][1][0][0][0][0][0]` → element 12,320 |

Zero compiler / reference-model / `FROZEN_HASH` changes are expected.

---

## 2. Three cooperation variants for Wave Loop 784

### Variant A — `[387][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-784` from `wave-loop-783` HEAD.
2. Copy `scripts/gen_w783.py` → `scripts/gen_w784.py`.
3. Set `OUTER = 387`, `MID_IDX = 193`, fix module prefix to `w784_bench_module_387x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w784_bench_module_387x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w784_bench_module_387x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W785 cooperation variants.

Why recommended: keeps the established mechanical generator discipline, tests
non-power-of-two stride 387, and stays well under the 4-MiBit cliff.

### Variant B — `[385][2]^6 Pt` bench/function-scope packed var from call

Keep the W783 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w783.py` with `OUTER = 385` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W783 (`MID_IDX = 192`).

Trade-off: tests a different code path (local arrays) but does not advance the
width ladder.

### Variant C — `[385][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W783 width and add conditional indexed signed field writes:

1. Generate a W783-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

Trade-off: tests control-flow emission but does not advance the width ladder.

---

## 3. Phase breakdown (PHI LOOP)

| Phase | Deliverable | Owner |
|-------|-------------|-------|
| Issue | Create issue, update current-issue.md, create W783 branch | Lead |
| Spec | `.t27` witness generated from `scripts/gen_w783.py` | Creator (C) |
| TDD | `test` + `bench` blocks with `assert_eq` on changed elements | Creator (C) |
| Code/Impl | Integration test in `bootstrap/tests/icarus_lowerable.rs` | Creator (C) |
| Gen | `python3 scripts/gen_w783.py` produces witness | Creator (C) |
| Seal | `t27c seal --save` succeeds, FROZEN_HASH unchanged | Verifier (V) |
| Verify | `cargo test`, `t27c parse\|icarus-*\|cocotb` all green | Verifier (V) |
| Land | PR opened, reviewed, merged to `master` | Lead |
| Learn | `.trinity/experience.md`, skill, memory, plan for W784 | Learner (L) |

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Earlier W774-W782 PRs still open | Branch from `wave-loop-782` HEAD; do not block on merge gate. |
| PR #1489 (README merge) blocked by `fpga-synthesis` | Out of scope for W783; track separately. |
| `assert_ne` not emitted by Icarus | Continue using `assert_eq` on changed elements. |
| Generator header prefix copy error | Manual fix after `sed` replacement (f-string `{OUTER}` keeps old prefix). |
| Remaining `verilog_array_literal_expr` / `verilog_const_array` regressions | Out of scope unless explicitly chartered; track as separate issue. |
| 627 release warnings | Dedicated cleanup sprint, not a wave-loop blocker. |

---

## 5. Literature baseline (2025-2026)

- TerEffic — arXiv 2502.16473v2 (2025). Ternary-LLM FPGA accelerator.
- Ternary VHDL — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00041.
- Trinity B002 — Zenodo 10.5281/zenodo.19224235 (2026). DSP-free ternary inference.
- SONIC — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00042. Ternary simulator.
- 5500FP — The Register, 2026-03-18. 24-trit balanced ternary RISC CPU.
- cocotb 2.0 — DVCon Europe 2024 / docs.cocotb.org.
- Yosys packed-array-in-struct support — commit f94eec95.

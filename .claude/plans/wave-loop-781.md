# Wave Loop 781 — Full-cycle plan

**Date:** 2026-07-24
**Issue:** #1492
**Branch:** `wave-loop-781`
**Parent:** `wave-loop-780` HEAD
**Next:** `wave-loop-782`
**Recommended variant:** A

---

## 1. Goal

Validate that the t27 packed-array-of-struct lowering scales to a module-scope
`[381][2]^6 Pt` variable initialized from a function call, with indexed signed
field writes and `assert_eq` read-back in a `bench` block.

In addition, close out the three actionable weak points discovered in the
2026-07-24 audit so that workspace tests, clippy, and the bitnet pipeline gate
are green again.

Target metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 381 |
| Total elements | 381 × 64 = 24,384 |
| Packed vector width | 24,384 × 32 = 780,288 bits |
| Approximate size | ~0.745 MiBit |
| Mid index | 190 |
| Frame-condition element | `[190][1][0][0][0][0][0]` → element 12,192 |

Zero compiler / reference-model / `FROZEN_HASH` changes are expected for the
witness itself.

---

## 2. Three cooperation variants for Wave Loop 782

### Variant A — `[383][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder to 383:

1. Create `wave-loop-782` from `wave-loop-781` HEAD.
2. Copy `scripts/gen_w781.py` → `scripts/gen_w782.py`.
3. Set `OUTER = 383`, `MID_IDX = 191`, fix module prefix to `w782_bench_module_383x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w782_bench_module_383x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w782_bench_module_383x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W783 cooperation variants.

Why recommended: keeps the established mechanical generator discipline, tests
non-power-of-two stride 383, and stays well under the 4-MiBit cliff.

### Variant B — `[381][2]^6 Pt` bench/function-scope packed var from call

Keep the W781 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w781.py` with `OUTER = 381` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W781 (MID_IDX = 190).

Trade-off: tests a different code path (local arrays) but does not advance the
width ladder.

### Variant C — `[381][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W781 width and add conditional indexed signed field writes:

1. Generate a W781-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

Trade-off: tests control-flow emission but does not advance the width ladder.

---

## 3. Phase breakdown (PHI LOOP)

| Phase | Deliverable | Owner |
|-------|-------------|-------|
| Issue | #1492 created, current-issue.md updated, W781 branch created | Lead |
| Spec | `.t27` witness generated from `scripts/gen_w781.py` | Creator (C) |
| TDD | `test` + `bench` blocks with `assert_eq` on changed elements | Creator (C) |
| Code/Impl | Integration test in `bootstrap/tests/icarus_lowerable.rs` | Creator (C) |
| Hygiene | Fix `flash-spi` `FlashOpts`, clippy `approx_constant`, bitnet IDLE substring | Creator (C) |
| Gen | `python3 scripts/gen_w781.py` produces witness | Creator (C) |
| Seal | `t27c seal --save` succeeds, FROZEN_HASH unchanged | Verifier (V) |
| Verify | `cargo test --workspace`, `cargo clippy -p t27c`, `t27c parse\|icarus-*\|cocotb` all green | Verifier (V) |
| Land | PR opened, reviewed, merged to `master` | Lead |
| Learn | `.trinity/experience.md`, skill, memory, plan for W782 | Learner (L) |

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Earlier W774-W780 PRs still open | Branch from `wave-loop-780` HEAD; do not block on merge gate. |
| PR #1489 (README merge) blocked by `fpga-synthesis` | Out of scope for W781; track separately. |
| `assert_ne` not emitted by Icarus | Continue using `assert_eq` on changed elements. |
| Generator header prefix copy error | Manual fix after `sed` replacement (f-string `{OUTER}` keeps old prefix). |
| flash-spi/clippy/bitnet fixes could mask real regressions | Verify each fix with the specific gate it unblocks. |

---

## 5. Literature baseline (2025-2026)

- TerEffic — arXiv 2502.16473v2 (2025). Ternary-LLM FPGA accelerator with TMat core and 1.6-bit weight compression.
- Ternary VHDL — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00041. Balanced ternary extension to VHDL-2008 for VLSI/FPGA.
- Trinity B002 — Zenodo 10.5281/zenodo.19224235 (2026). DSP-free FPGA ternary inference with OpenXC7/Yosys.
- SONIC — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00042. Event-driven ternary gate-level simulator exporting BCT Verilog.
- 5500FP — The Register, 2026-03-18. 24-trit balanced ternary RISC CPU on conventional FPGA.
- cocotb 2.0 — DVCon Europe 2024. Python testbench framework, Icarus 11+ support, Python Runner flow.

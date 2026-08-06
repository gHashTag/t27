# FPGA Loop Closeout — Wave Loop 783

**Date:** 2026-07-24
**Issue:** #1495
**Branch:** `wave-loop-783`
**Parent:** `wave-loop-782` HEAD (`753197599`)
**Cooperation variant:** A (recommended)
**Next wave:** `wave-loop-784`

---

## 1. Summary

Wave Loop 783 closed the next rung of the module-scope packed-array-of-struct
ladder: a `[385][2]^6 Pt` variable initialized from a function call, exercised
with indexed signed field writes and `assert_eq` read-back. The witness is
788,480 bits (~0.752 MiBit), still well below the 4-MiBit packed-vector cliff,
and required **zero compiler, reference-model, or `FROZEN_HASH` changes**.

In addition to the witness, this closeout fixed one actionable weak point
discovered in the 2026-07-24 audit: a stale expected TODO string in
`bootstrap/tests/verilog_const_array.rs:166` that no longer matched the richer
message emitted by the real `mac.t27` array-literal initializer path. The test now
accepts any TODO marker containing `TODO: array literal` or `TODO: struct literal`.
The deeper `verilog_array_literal_expr` regression remains pre-existing and out of
scope.

---

## 2. What was implemented

### 2.1 Witness `[385][2]^6 Pt`

- Generator: `scripts/gen_w783.py` (copied from `scripts/gen_w782.py`, updated to
  `OUTER = 385`, `MID_IDX = 192`, and module prefix `w783_bench_module_385x2p6_aos_var_call_write`).
- Generated spec: `specs/scratch/w783_bench_module_385x2p6_aos_var_call_write.t27`
  (~1,687 KB, ~73,211 lines, 24,640 elements, 788,480-bit packed vector).
- Integration test: `accepts_w783_bench_module_385x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Frame-condition element: `[192][1][0][0][0][0][0]` → element
  `192*64 + 32 = 12,320`.
- Period-identity check: `make_grid(32768)` because `32768 ≡ 0 (mod 32768)`.

### 2.2 Weak-point fix

| File | Problem | Fix |
|------|---------|-----|
| `bootstrap/tests/verilog_const_array.rs:166` | Expected exact stale TODO strings (`"TODO: array literal initializer not yet lowered"` / `"TODO: struct literal initializer not yet lowered"`) that no longer matched the richer emitter output. | Relaxed check to any substring `TODO: array literal` or `TODO: struct literal`. |

### 2.3 Remaining weak points (not fixed)

- `bootstrap/tests/verilog_array_literal_expr.rs::r_ca_2_synthetic_no_comment_only_call_argument`
  still fails because the synthetic spec no longer exercises the expected placeholder
  path. This is a deeper compiler lowering issue and should be tracked as its own issue.
- FPGA E2E CI remains red (`sby` missing, Yosys static-cast error in generated `uart.v`)
  across multiple recent branches; toolchain-wide and out of scope for the wave-loop
  witness ladder.
- 626 release warnings and 780 clippy warnings remain; too large for a single wave
  closeout.
- Root `NOW.md` and `README.md` status cells are partially stale.

---

## 3. Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 243 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W783 | PASS |
| `t27c icarus-lowerable` W783 | PASS (`lowerable`) |
| `t27c icarus-simulate` W783 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W783 | PASS (`reference-model OK`) |
| `t27c seal --save` W783 | PASS |
| `FROZEN_HASH` | Unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |
| `cargo test --workspace` | Fails only on pre-existing `verilog_array_literal_expr` regression |

---

## 4. Weak-point audit (2026-07-24)

### 4.1 Fixed in this wave

1. **`verilog_const_array` stale TODO expectation** — test now matches the current
   emitter diagnostic format.

### 4.2 Remaining medium/low risks

1. **`verilog_array_literal_expr` regression** — deeper compiler lowering gap; track as
   separate issue.
2. **FPGA E2E CI red** — `sby` dependency missing + Yosys Verilog-2005 static-cast
   issue; toolchain-wide.
3. **626/780 warnings** — dedicated cleanup sprint needed.
4. **Docs staleness** — root `NOW.md` and `README.md` status tables need refresh;
   license badge may need Apache-2.0 alignment via PR #1437.
5. **Open PR stack** — W774-W782 PRs remain open; W783 branched from `wave-loop-782`
   HEAD to keep the sequence unblocked.

### 4.3 Hygiene checks

- No secrets found in working tree or `.env.example` files.
- `icarus_lowerable` coverage: 243 tests pass.
- 59 of 889 `.t27` specs lack `test`/`invariant`/`bench` (≈6.64%).
- 19 `scripts/*.sh` files remain under `scripts/`.

---

## 5. 2025-2026 literature scan

Selected recent publications and artifacts relevant to t27 / ternary / MVL / FPGA:

- **Tlsys** — Chinese Journal of Electronics 2026, DOI 10.23919/cje.2025.00.418.
  First ternary RTL-to-CNFET-netlist synthesis framework; includes verification
  methodology and reports netlists over 500,000 gates. Reinforces the value of
  source-to-netlist tooling for ternary designs.
- **Ternary VHDL** — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00041.
  Balanced-ternary extension to IEEE 1076-2008 with GHDL simulation support;
  relevant to t27's language-level EDA interoperability.
- **SONIC** — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00042.
  Event-driven gate-level ternary simulator with BCT Verilog export; useful
  reference for future t27 simulator backends.
- **5500FP** — Zenodo 10.5281/zenodo.18881738 / Open MIND 2026. A 24-trit
  balanced-ternary RISC CPU on an Efinix FPGA at 20 MHz with genuine ±3.3 V
  ternary I/O and a 120-instruction ISA. Demonstrates practical, programmable
  balanced-ternary hardware is now available off-the-shelf.
- **Icarus Verilog v13.0** — stable release March 2026. Improved VPI reliability and
  conformance; compatible with cocotb 2.0 and the t27 Icarus reference-model gate.
- **Yosys packed-array-in-struct support** — upstream still does not support arrays
  of packed structs/unions as of 2025 (YosysHQ/yosys#4653), making t27's flattening
  approach the safer open-source path.

---

## 6. Cooperation variants for Wave Loop 784

### Variant A — `[387][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-784` from `wave-loop-783` HEAD.
2. Copy `scripts/gen_w783.py` → `scripts/gen_w784.py`.
3. Set `OUTER = 387`, `MID_IDX = 193`, fix module prefix to `w784_bench_module_387x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w784_bench_module_387x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w784_bench_module_387x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W785 cooperation variants.

**Why recommended:** keeps the established mechanical generator discipline, tests
non-power-of-two stride 387, and stays well under the 4-MiBit cliff.

### Variant B — `[385][2]^6 Pt` bench/function-scope packed var from call

Keep the W783 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w783.py` with `OUTER = 385` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W783 (`MID_IDX = 192`).

**Trade-off:** tests a different code path (local arrays) but does not advance
the width ladder.

### Variant C — `[385][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W783 width and add conditional indexed signed field writes:

1. Generate a W783-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 7. Next steps

1. Open PR for `wave-loop-783` against `master` (or stack after earlier waves land).
2. Link PR body to issue #1495 with `Closes #1495`.
3. After merge, create `wave-loop-784` from `wave-loop-783` HEAD and execute
   Variant A unless the ring selects B or C.

---

φ² + 1/φ² = 3 | TRINITY

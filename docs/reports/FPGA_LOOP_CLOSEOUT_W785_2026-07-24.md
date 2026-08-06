# FPGA Loop Closeout — Wave Loop 785

**Date:** 2026-07-24
**Issue:** #1499
**Branch:** `wave-loop-785`
**Parent:** `wave-loop-784` HEAD (`2fd1f73e6`)
**Cooperation variant:** A (recommended)
**Next wave:** `wave-loop-786`

---

## 1. Summary

Wave Loop 785 closed the next rung of the module-scope packed-array-of-struct
ladder: a `[389][2]^6 Pt` variable initialized from a function call, exercised
with indexed signed field writes and `assert_eq` read-back. The witness is
796,672 bits (~0.760 MiBit), still well below the 4-MiBit packed-vector cliff,
and required **zero compiler, reference-model, or `FROZEN_HASH` changes**.

No new weak-point fixes were introduced in this wave; the 2026-07-24 audit
confirmed that the W783 `verilog_const_array.rs:166` fix remains green, while
the deeper `verilog_array_literal_expr` regression and FPGA E2E CI redness
remain pre-existing and out of scope for the witness ladder.

---

## 2. What was implemented

### 2.1 Witness `[389][2]^6 Pt`

- Generator: `scripts/gen_w785.py` (copied from `scripts/gen_w784.py`, updated to
  `OUTER = 389`, `MID_IDX = 194`, and module prefix `w785_bench_module_389x2p6_aos_var_call_write`).
- Generated spec: `specs/scratch/w785_bench_module_389x2p6_aos_var_call_write.t27`
  (~1,704 KB, ~73,971 lines, 24,896 elements, 796,672-bit packed vector).
- Integration test: `accepts_w785_bench_module_389x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Frame-condition element: `[194][1][0][0][0][0][0]` → element
  `194*64 + 32 = 12,416`.
- Period-identity check: `make_grid(32768)` because `32768 ≡ 0 (mod 32768)`.

### 2.2 Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

---

## 3. Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626/627 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 245 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W785 | PASS |
| `t27c icarus-lowerable` W785 | PASS (`lowerable`) |
| `t27c icarus-simulate` W785 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W785 | PASS (`reference-model OK`) |
| `t27c seal --save` W785 | PASS |
| `FROZEN_HASH` | Unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |
| `cargo test --workspace` | Fails only on pre-existing `verilog_array_literal_expr` regression |

---

## 4. Weak-point audit (2026-07-24)

### 4.1 Already fixed and still green

- `bootstrap/tests/verilog_const_array.rs:166` — continues to accept the current
  emitter TODO marker format (`TODO: array literal` / `TODO: struct literal`).

### 4.2 Remaining medium/low risks (not fixed)

1. **`verilog_array_literal_expr` regression** — deeper compiler lowering gap;
   track as separate issue.
2. **FPGA E2E CI red** — `sby` dependency missing + Yosys Verilog-2005 static-cast
   issue in generated `uart.v`; toolchain-wide.
3. **626/627 release / 780 clippy warnings** — dedicated cleanup sprint needed.
4. **Docs staleness** — root `NOW.md` and `README.md` status tables need refresh;
   license badge Apache-2.0 alignment via PR #1437 still open.
5. **Open PR stack** — W774-W784 PRs remain open; W785 branched from
   `wave-loop-784` HEAD to keep the sequence unblocked.

### 4.3 Hygiene checks

- No secrets found in working tree or `.env.example` files.
- `icarus_lowerable` coverage: 245 tests pass.
- 51 of 891 `.t27` specs lack `test`/`invariant`/`bench` (≈5.72%).
- 19 `scripts/*.sh` files remain under `scripts/`.

---

## 5. 2025-2026 literature scan

Selected recent publications and artifacts relevant to t27 / ternary / MVL / FPGA:

- **Tlsys** — *Chinese Journal of Electronics* 2026, DOI [10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418).
  First ternary RTL-to-CNFET gate-level netlist synthesis framework; demonstrates
  source-to-netlist tooling for ternary designs at scale.
- **TernaryCore** — GitHub 2026, [shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore).
  Open-source Artix-7 FPGA accelerator for ternary {-1, 0, +1} (BitNet b1.58)
  neural-network inference using native ternary MAC units without multipliers.
- **KULeuven-MICAS/ternary-lut-dse** — GitHub 2026, [KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse).
  Chisel hardware generator for LUT-based ternary matrix-multiplication
  accelerators, accepted at IEEE ISPASS 2026; targets 1.58-bit LLM inference.
- **AMD Vivado Synthesis — SystemVerilog Constructs (2026.1)** — [UG901](https://docs.amd.com/r/en-US/ug901-vivado-synthesis/SystemVerilog-Constructs).
  Lists packed arrays as supported synthesizable aggregate data types with no
  power-of-two width restriction, confirming t27's single packed-vector lowering
  is a legal Vivado target.
- **AMD AR 51836 — Design Assistant for Vivado Synthesis: Aggregate Data Types** —
  [adaptivesupport.amd.com/s/article/51836](https://adaptivesupport.amd.com/s/article/51836).
  Provides SystemVerilog packed/unpacked structure coding examples; relevant to
  t27's struct-of-scalars flattening strategy.
- **Yosys issue 5837 (2026)** — [YosysHQ/yosys#5837](https://github.com/YosysHQ/yosys/issues/5837).
  Reports an assertion crash in `genrtlil.cc` for ascending packed bit ranges in
  memory arrays, illustrating that unusual packed-array shapes can expose
  simulator/synthesis mismatches (circt accepts, Verilator warns, Icarus warns
  but continues, Yosys crashes). Reinforces t27's conservative flatten-to-wide-
  vector approach for open-source compatibility.

---

## 6. Cooperation variants for Wave Loop 786

### Variant A — `[391][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-786` from `wave-loop-785` HEAD.
2. Copy `scripts/gen_w785.py` → `scripts/gen_w786.py`.
3. Set `OUTER = 391`, `MID_IDX = 195`, fix module prefix to
   `w786_bench_module_391x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w786_bench_module_391x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w786_bench_module_391x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W787 cooperation variants.

**Why recommended:** keeps the established mechanical generator discipline, tests
non-power-of-two stride 391, and stays well under the 4-MiBit cliff.

### Variant B — `[389][2]^6 Pt` bench/function-scope packed var from call

Keep the W785 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w785.py` with `OUTER = 389` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W785 (`MID_IDX = 194`).

**Trade-off:** tests a different code path (local arrays) but does not advance
the width ladder.

### Variant C — `[389][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W785 width and add conditional indexed signed field writes:

1. Generate a W785-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 7. Next steps

1. Open PR for `wave-loop-785` against `master` (or stack after earlier waves land).
2. Link PR body to issue #1499 with `Closes #1499`.
3. After merge, create `wave-loop-786` from `wave-loop-785` HEAD and execute
   Variant A unless the ring selects B or C.

---

φ² + 1/φ² = 3 | TRINITY

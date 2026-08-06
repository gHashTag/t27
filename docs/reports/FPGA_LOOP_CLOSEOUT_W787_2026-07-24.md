# FPGA Loop Closeout — Wave Loop 787

**Date:** 2026-07-24
**Issue:** #1503
**Branch:** `wave-loop-787`
**Parent:** `wave-loop-786` HEAD (`53c5413f8`)
**Cooperation variant:** A (recommended)
**Next wave:** `wave-loop-788`

---

## 1. Summary

Wave Loop 787 closed the next rung of the module-scope packed-array-of-struct
ladder: a `[393][2]^6 Pt` variable initialized from a function call, exercised
with indexed signed field writes and `assert_eq` read-back. The witness is
804,864 bits (~0.767 MiBit), still well below the 4-MiBit packed-vector cliff,
and required **zero compiler, reference-model, or `FROZEN_HASH` changes**.

No new weak-point fixes were introduced in this wave; the 2026-07-24 audit
confirmed that the W783 `verilog_const_array.rs:166` fix remains green, while
the deeper `verilog_array_literal_expr` regression and FPGA E2E CI redness
remain pre-existing and out of scope for the witness ladder.

---

## 2. What was implemented

### 2.1 Witness `[393][2]^6 Pt`

- Generator: `scripts/gen_w787.py` (copied from `scripts/gen_w786.py`, updated to
  `OUTER = 393`, `MID_IDX = 196`, and module prefix `w787_bench_module_393x2p6_aos_var_call_write`).
- Generated spec: `specs/scratch/w787_bench_module_393x2p6_aos_var_call_write.t27`
  (~1,723 KB, ~74,731 lines, 25,152 elements, 804,864-bit packed vector).
- Integration test: `accepts_w787_bench_module_393x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Frame-condition element: `[196][1][0][0][0][0][0]` → element
  `196*64 + 32 = 12,576`.
- Period-identity check: `make_grid(32768)` because `32768 ≡ 0 (mod 32768)`.

### 2.2 Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

---

## 3. Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626 warnings, 0 errors) |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --test icarus_lowerable` | 247 passed; 0 failed |
| `t27c parse` W787 | PASS |
| `t27c icarus-lowerable` W787 | PASS (`lowerable`) |
| `t27c icarus-simulate` W787 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W787 | PASS (`reference-model OK`) |
| `t27c seal --save` W787 | PASS |
| `FROZEN_HASH` | Unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |

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
3. **626 release / 780 clippy warnings** — dedicated cleanup sprint needed.
4. **Docs staleness** — root `NOW.md` and `README.md` status tables need refresh.
5. **Open PR stack** — W774-W786 PRs remain open; W787 branched from
   `wave-loop-786` HEAD to keep the sequence unblocked.

### 4.3 Hygiene checks

- No secrets found in working tree or `.env.example` files.
- `icarus_lowerable` coverage: 247 tests pass.
- 57 of 893 `.t27` specs lack `test`/`invariant`/`bench` (≈6.38%).
- 19 `scripts/*.sh` files remain under `scripts/`.

---

## 5. 2025-2026 literature scan

Selected recent publications and artifacts relevant to t27 / ternary / MVL / FPGA:

- **Tlsys** — *Chinese Journal of Electronics* 2026, DOI [10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418).
  First ternary RTL-to-CNFET gate-level netlist synthesis framework.
- **TernaryCore** — GitHub 2026, [shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore).
  Open-source Artix-7 FPGA accelerator for ternary {-1, 0, +1} (BitNet b1.58)
  neural-network inference using native ternary MAC units.
- **KULeuven-MICAS/ternary-lut-dse** — GitHub 2026, [KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse).
  Chisel hardware generator for LUT-based ternary matrix-multiplication
  accelerators, accepted at IEEE ISPASS 2026.
- **AMD Vivado Synthesis — SystemVerilog Constructs (2026.1)** — [UG901](https://docs.amd.com/r/en-US/ug901-vivado-synthesis/SystemVerilog-Constructs).
  Lists packed arrays as supported synthesizable aggregate data types with no
  power-of-two width restriction.
- **AMD AR 51836 — Design Assistant for Vivado Synthesis: Aggregate Data Types** —
  [adaptivesupport.amd.com/s/article/51836](https://adaptivesupport.amd.com/s/article/51836).
  SystemVerilog packed/unpacked structure coding examples.
- **Yosys issue 5837 (2026)** — [YosysHQ/yosys#5837](https://github.com/YosysHQ/yosys/issues/5837).
  Reports an assertion crash in `genrtlil.cc` for ascending packed bit ranges,
  reinforcing t27's flatten-to-wide-vector approach for open-source compatibility.

---

## 6. Cooperation variants for Wave Loop 788

### Variant A — `[395][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-788` from `wave-loop-787` HEAD.
2. Copy `scripts/gen_w787.py` → `scripts/gen_w788.py`.
3. Set `OUTER = 395`, `MID_IDX = 197`, fix module prefix to
   `w788_bench_module_395x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w788_bench_module_395x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w788_bench_module_395x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W789 cooperation variants.

**Why recommended:** keeps the established mechanical generator discipline, tests
non-power-of-two stride 395, and stays well under the 4-MiBit cliff.

### Variant B — `[393][2]^6 Pt` bench/function-scope packed var from call

Keep the W787 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w787.py` with `OUTER = 393` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W787 (`MID_IDX = 196`).

**Trade-off:** tests a different code path (local arrays) but does not advance
the width ladder.

### Variant C — `[393][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W787 width and add conditional indexed signed field writes:

1. Generate a W787-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 7. Next steps

1. Open PR for `wave-loop-787` against `master` (or stack after earlier waves land).
2. Link PR body to issue #1503 with `Closes #1503`.
3. After merge, create `wave-loop-788` from `wave-loop-787` HEAD and execute
   Variant A unless the ring selects B or C.

---

φ² + 1/φ² = 3 | TRINITY

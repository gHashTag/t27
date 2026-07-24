# FPGA Loop Closeout — Wave Loop 776

**Date:** 2026-07-24  
**Issue:** #1487  
**Branch:** `wave-loop-776`  
**Parent branch:** `wave-loop-775` HEAD (`2e86eb0b8`)  
**Next branch:** `wave-loop-777`  
**Witness:** `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w776.py`

---

## 1. Summary

Wave Loop 776 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[371][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [371][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 371 |
| Total elements | 371 × 64 = 23,744 |
| Packed vector width | 23,744 × 32 = 759,808 bits |
| Approximate size | ~0.725 MiBit |
| Mid index | `MID_IDX = 185` |
| Frame-condition element | `[185][1][0][0][0][0][0]` → element 11,872 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Created `wave-loop-776` from `wave-loop-775` HEAD because PR #1484 (W774)
   and PR #1486 (W775) remain open and awaiting review.
2. Copied `scripts/gen_w775.py` → `scripts/gen_w776.py`.
3. Updated constants: `OUTER = 371`, `MID_IDX = 185`.
4. Fixed the module header prefix to
   `w776_bench_module_371x2p6_aos_var_call_write`.
5. Generated the witness with `python3 scripts/gen_w776.py`.
6. Added integration test `accepts_w776_bench_module_371x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs`.
7. Sealed the witness with `t27c seal --save`.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[185][1][0][0][0][0][0]`:

```
MID_E = 185*64 + 32 = 11,872
MID_X = (2 * 11,872) % 32768 = 23,744
MID_Y = 23,745
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 23,744 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*23743) % 32768 = 14,718`).

`assert_ne` is structurally accepted by the classifier but not emitted on the
Icarus simulation path; the bench therefore uses `assert_eq` checks on the
changed elements.

---

## 3. Validation results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 236 passed; 0 failed |
| `t27c parse` W776 | PASS |
| `t27c icarus-lowerable` W776 | PASS (`lowerable`) |
| `t27c icarus-simulate` W776 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W776 | PASS (`reference-model OK`) |
| `t27c seal --save` W776 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` / `Fixes #N` | **STABLE LOW**: 13 of 69 commits in the current 30-day window carry an issue link in the subject line (≈18.8%). Merge/closeout bodies still carry `Closes #N`. Continue recommending subject-line links for feat/merge/closeout commits. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 57 of 882 `.t27` files still lack any test/invariant/bench block (≈6.5%). No new untested specs added this wave. |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| Open PRs in flight | **PR #1484 (W774) and PR #1486 (W775) are OPEN/BLOCKED** awaiting review. W776 branched from `wave-loop-775` HEAD to avoid blocking on merge gate. |
| Worktree hygiene | Untracked stale W485 artefacts remain: `specs/scratch/w485_*.t27` (×3) and `.claude/plans/wave-loop-485.md`. These are unrelated to W776 and should be resolved on a dedicated W485 branch or removed. |
| `NOW.md` currency | `NOW.md` does not reflect W774/W775/W776 state; refreshed in this closeout. |
| FPGA synthesis / formal pre-existing failures | `fpga-formal`, `fpga-synthesis`, and `fpga-synthesis-arty` remain failing for infrastructure reasons unrelated to the wave: `sby` pip package unavailable, and Yosys Verilog-2005 static-cast limitation in `build/fpga/generated/uart.v` (weak point #1245). |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE Std 1800-2017 remains the authoritative basis for the W776 witness: packed
arrays of structs and arbitrary-width packed vectors are defined in Clause 7
(Aggregate Data Types). The `[371][2]^6 Pt` shape flattens to a single
759,808-bit SystemVerilog packed vector, which is legal and simulator-portable
when scalar-flattened for Icarus. AMD/Xilinx UG901 (2026.1) and AR 51836 confirm
that Vivado simulation and synthesis accept packed structs/arrays as wide vectors,
with DPI mapping them to `svLogicVecVal` arrays.

2025–2026 ternary / MVL / open-source verification landscape relevant to t27:

- **REBEL-6: A 32-trit balanced ternary instruction set architecture with R2R
  compiler pipeline for C** — 32-trit balanced ternary ISA comparable to RV32I,
  open-source RV32I-to-REBEL compiler, 1.4% fewer instructions and 33.2% lower
  dynamic power vs. RV32I (IEEE ISMVL 2025,
  [DOI 10.1109/ismvl64713.2025.00028](https://doi.org/10.1109/ismvl64713.2025.00028)).
- **SONIC: Event-Driven Gate-Level Simulator of Ternary VLSI Circuits using Delta
  Cycles** — open-source multi-valued EDA simulator/verification backend;
  automates verification of the REBEL-2 ternary CPU and exports BCT Verilog for
  FPGA/ASIC testing (IEEE ISMVL 2026,
  [DOI 10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042);
  [GitHub](https://github.com/sonbit/SimulationEngine)).
- **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI
  Circuits** — TVHDL balanced ternary extension to IEEE 1076-2008 VHDL,
  verified with GHDL/GTKWave (IEEE ISMVL 2026,
  [DOI 10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)).
- **VTX1** — open-source balanced-ternary SoC with CPU, memory/cache, UART/SPI/I2C/GPIO/DMA,
  RTL-to-silicon flow using Icarus Verilog and Yosys, SkyWater 130nm tape-out
  planned ([GitHub](https://github.com/itworks99/vtx1)).
- **Yosys SystemVerilog gaps**: Yosys 0.65-dev documentation and issues #2677,
  #2908, #5837 continue to show that arrays of packed structs/unions and
  non-standard packed ranges are fragile. t27’s scalar packed-vector flattening
  avoids those constructs entirely.

Sources:
- IEEE Std 1800-2017 SystemVerilog LRM (packed arrays / structs): [MIT-hosted PDF](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- AMD UG901 Vivado Logic Simulation 2026.1 — Packed Struct/Union: [docs.amd.com](https://docs.amd.com/r/en-US/ug900-vivado-logic-simulation/Packed-Struct/Union)
- AMD AR 51836 — Vivado Synthesis aggregate data types: [adaptivesupport.amd.com](https://adaptivesupport.amd.com/s/article/51836)
- Yosys Verilog support notes (0.65-dev): [yosyshq.readthedocs.io](https://yosyshq.readthedocs.io/projects/yosys/en/latest/using_yosys/verilog.html)

---

## 6. Three cooperation variants for Wave Loop 777

1. **Variant A (recommended): `[373][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (373 → 23,872 elements, 764,672
     bits, ~0.729 MiBit) and confirms non-power-of-two stride 373.

2. **Variant B: `[371][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W776 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[371][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.725 MiBit and tests control-flow-guarded writes on a packed reg,
     verifying conditional write emission in the Icarus path.

---

## 7. Definition of done

- [x] Witness generated and under version control.
- [x] Integration test added and passing.
- [x] Icarus lowerability, simulation, cocotb, and seal gates green.
- [x] Cargo suites green.
- [x] FROZEN_HASH unchanged.
- [x] Closeout report written.
- [x] `.trinity/experience.md` updated.
- [x] Next-wave cooperation variants defined.

---

phi^2 + 1/phi^2 = 3 | TRINITY

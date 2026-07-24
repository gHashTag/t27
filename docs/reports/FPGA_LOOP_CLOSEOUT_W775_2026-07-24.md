# FPGA Loop Closeout — Wave Loop 775

**Date:** 2026-07-24  
**Issue:** TBD  
**Branch:** `wave-loop-775`  
**Next branch:** `wave-loop-776`  
**Witness:** `specs/scratch/w775_bench_module_369x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w775.py`

---

## 1. Summary

Wave Loop 775 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[369][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [369][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 369 |
| Total elements | 369 × 64 = 23,616 |
| Packed vector width | 23,616 × 32 = 755,712 bits |
| Approximate size | ~0.721 MiBit |
| Mid index | `MID_IDX = 184` |
| Frame-condition element | `[184][1][0][0][0][0][0]` → element 11,808 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Branched `wave-loop-775` from `wave-loop-774` HEAD because PR #1484 (W774)
   remains open and awaiting review.
2. Copied `scripts/gen_w774.py` → `scripts/gen_w775.py`.
3. Updated constants: `OUTER = 369`, `MID_IDX = 184`.
4. Fixed the f-string module header so the literal expands to
   `w775_bench_module_369x2p6_aos_var_call_write`.
5. Generated the witness with `python3 scripts/gen_w775.py`.
6. Added integration test `accepts_w775_bench_module_369x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
7. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[184][1][0][0][0][0][0]`:

```
MID_E = 184*64 + 32 = 11,808
MID_X = (2 * 11,808) % 32768 = 23,616
MID_Y = 23,617
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 23,616 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*23615) % 32768 = 14,462`).

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
| `cargo test -p t27c --test icarus_lowerable` | 235 passed; 0 failed |
| `t27c parse` W775 | PASS |
| `t27c icarus-lowerable` W775 | PASS (`lowerable`) |
| `t27c icarus-simulate` W775 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W775 | PASS (`reference-model OK`) |
| `t27c seal --save` W775 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` / `Fixes #N` | **STABLE LOW**: 11 of 68 commits in the current 30-day window carry an issue link in the subject line (≈16.2%). Merge/closeout bodies still carry `Closes #N`. Recommending subject-line links for feat/merge/closeout commits going forward. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 51 of 881 non-worktree `.t27` files still lack any test/invariant/bench block (≈5.8%). No new untested specs added this wave. |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| Open PR in flight | **PR #1484 (W774) is OPEN/BLOCKED** awaiting review. W775 branched from `wave-loop-774` HEAD to avoid blocking on merge gate. |
| Worktree hygiene | Untracked stale W485 artefacts remain: `specs/scratch/w485_*.t27` (×3) and `.claude/plans/wave-loop-485.md`. These are unrelated to W775 and should be resolved on a dedicated W485 branch or removed. |
| `NOW.md` currency | `NOW.md` was last updated 2026-05-24 and does not reflect current wave-loop work; schedule refresh. |
| FPGA synthesis / formal pre-existing failures | `fpga-formal`, `fpga-synthesis`, and `fpga-synthesis-arty` remain failing for infrastructure reasons unrelated to the wave: `sby` pip package unavailable, and Yosys Verilog-2005 static-cast limitation in `build/fpga/generated/uart.v` (weak point #1245). |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE Std 1800-2017 remains the authoritative basis for the W775 witness: packed
arrays of structs and arbitrary-width packed vectors are defined in Clause 7
(Aggregate Data Types). The `[369][2]^6 Pt` shape flattens to a single
755,712-bit SystemVerilog packed vector, which is legal and simulator-portable
when scalar-flattened for Icarus. AMD/Xilinx UG900 (2026.1) and AR 51836 confirm
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
- **KULeuven ternary-lut-dse** — Chisel generator for LUT-based ternary MatMul
  targeting 1.58-bit LLMs, accepted at IEEE ISPASS 2026
  ([GitHub](https://github.com/KULeuven-MICAS/ternary-lut-dse)).
- **VTX1** — open-source balanced-ternary SoC with CPU, memory/cache, UART/SPI/I2C/GPIO/DMA,
  RTL-to-silicon flow using Icarus Verilog and Yosys, SkyWater 130nm tape-out
  planned ([GitHub](https://github.com/itworks99/vtx1)).
- **SystemVerilog tooling gaps**: Yosys native `read_verilog -sv` still does not
  support arrays of packed structs ([issue #4653](https://github.com/YosysHQ/yosys/issues/4653),
  [issue #2677](https://github.com/YosysHQ/yosys/issues/2677)); Icarus issue #1134
  shows assertion failures for unpacked arrays of packed structs. t27’s scalar
  packed-vector flattening avoids both gaps.

Sources:
- IEEE Std 1800-2017 SystemVerilog LRM (packed arrays / structs): [MIT-hosted PDF](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- AMD UG900 Vivado Logic Simulation 2026.1 — Packed Struct/Union: [docs.amd.com](https://docs.amd.com/r/en-US/ug900-vivado-logic-simulation/Packed-Struct/Union)
- AMD AR 51836 — Vivado Synthesis aggregate data types: [adaptivesupport.amd.com](https://adaptivesupport.amd.com/s/article/51836)

---

## 6. Three cooperation variants for Wave Loop 776

1. **Variant A (recommended): `[371][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (371 → 23,744 elements, 759,808
     bits, ~0.725 MiBit) and confirms non-power-of-two stride 371.

2. **Variant B: `[369][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W775 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[369][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.721 MiBit and tests control-flow-guarded writes on a packed reg,
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

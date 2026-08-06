# FPGA Loop Closeout — Wave Loop 779

**Date:** 2026-07-24
**Issue:** #1494
**Branch:** `wave-loop-779`
**Parent branch:** `wave-loop-778` HEAD (`0c856f5f4`)
**Next branch:** `wave-loop-780`
**Witness:** `specs/scratch/w779_bench_module_377x2p6_aos_var_call_write.t27`
**Generator:** `scripts/gen_w779.py`
**PR:** #1495

---

## 1. Summary

Wave Loop 779 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[377][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [377][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 377 |
| Total elements | 377 × 64 = 24,128 |
| Packed vector width | 24,128 × 32 = 772,096 bits |
| Approximate size | ~0.737 MiBit |
| Mid index | `MID_IDX = 188` |
| Frame-condition element | `[188][1][0][0][0][0][0]` → element 12,064 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Created `wave-loop-779` from `wave-loop-778` HEAD because PR #1484 (W774),
   PR #1486 (W775), PR #1488 (W776), PR #1489 (README/W774-W776 merge),
   PR #1491 (W777), and PR #1493 (W778) remain open and/or unstable.
2. Copied `scripts/gen_w778.py` → `scripts/gen_w779.py`.
3. Updated constants: `OUTER = 377`, `MID_IDX = 188`.
4. Fixed the module header prefix to
   `w779_bench_module_377x2p6_aos_var_call_write`.
5. Generated the witness with `python3 scripts/gen_w779.py`.
6. Added integration test `accepts_w779_bench_module_377x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs`.
7. Sealed the witness with `t27c seal --save`.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[188][1][0][0][0][0][0]`:

```
MID_E = 188*64 + 32 = 12,064
MID_X = (2 * 12,064) % 32768 = 24,128
MID_Y = 24,129
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 24,128 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*24127) % 32768 = 15,486`).

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
| `cargo test -p t27c --test icarus_lowerable` | 239 passed; 0 failed |
| `t27c parse` W779 | PASS |
| `t27c icarus-lowerable` W779 | PASS (`lowerable`) |
| `t27c icarus-simulate` W779 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W779 | PASS (`reference-model OK`) |
| `t27c seal --save` W779 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` / `Fixes #N` | **~17.6%** (13 of 74 commits). Essentially flat versus W778; still far below the prior ~80% local spike. Sustained effort is needed. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 57 of 885 `.t27` files still lack any test/invariant/bench block (≈6.44%). One new scratch witness (W779) was added and contains both `test` and `bench` blocks. |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| Open PRs in flight | **PR #1484 (W774), #1486 (W775), #1488 (W776), #1489 (README update), #1491 (W777), and #1493 (W778) remain OPEN.** W779 branched from `wave-loop-778` HEAD to avoid blocking on the merge gate. |
| PR #1489 merge blocker | `fpga-synthesis` fails with pre-existing Yosys error: `Static cast is only supported in SystemVerilog mode` at `build/fpga/generated/uart.v:31`. `--admin` merge rejected while required checks are red. Out of scope for W779. |
| Test drift: `bitnet_pipeline::sequencer_idle_arms_on_start` | **STILL PRESENT**: `bootstrap/tests/bitnet_pipeline.rs:143` expects exact string `IDLE: if(start) begin ...` but actual `gen-layer-sequencer` output wraps it as `IDLE: begin done<=0; if(start) begin ... end end`. This pre-existing test-vs-emitter drift is unrelated to the wave-loop ladder. |
| Worktree hygiene | Untracked stale W485 artefacts remain: `specs/scratch/w485_*.t27` (×3) and `.claude/plans/wave-loop-485.md`. These are unrelated to W779 and should be resolved on a dedicated W485 branch or removed. |
| `main` vs `master` divergence | Separate `main` branch contains useful work (dlc10, Verilog struct access, TRI-NET) but has dozens of add/add conflicts against `master`; needs a dedicated merge/reconciliation session. |
| `NOW.md` currency | `NOW.md` refreshed to reflect W779 landed / W780 next state. |
| FPGA synthesis / formal pre-existing failures | `fpga-formal`, `fpga-synthesis`, and `fpga-synthesis-arty` remain failing for infrastructure reasons unrelated to the wave: `sby` pip package unavailable, and Yosys Verilog-2005 static-cast limitation in `build/fpga/generated/uart.v`. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE Std 1800-2017 remains the authoritative basis for the W779 witness: packed
arrays of structs and arbitrary-width packed vectors are defined in Clause 7
(Aggregate Data Types). The `[377][2]^6 Pt` shape flattens to a single
772,096-bit SystemVerilog packed vector, which is legal and simulator-portable
when scalar-flattened for Icarus. AMD UG901 (2026.1) and AR 51836 confirm that
Vivado simulation and synthesis accept packed structs/arrays as wide vectors,
with DPI mapping them to `svLogicVecVal` arrays.

2024–2026 ternary / MVL / open-source verification landscape relevant to t27:

- **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** (arXiv 2025,
  [https://arxiv.org/html/2502.16473v2](https://arxiv.org/html/2502.16473v2)) —
  packs 5 ternary weights into 8 bits because 3⁵ = 243 < 256, directly relevant
  to non-power-of-two packed ternary arrays.
- **TeLLMe: An Energy-Efficient Ternary LLM Accelerator for Prefilling and
  Decoding on Edge FPGAs** (arXiv 2025,
  [https://arxiv.org/pdf/2504.16266](https://arxiv.org/pdf/2504.16266)) — packs
  3 ternary values into a 5-bit index (27 combinations) for LUT-based ternary
  matrix multiplication on AMD KV260.
- **Hardware Generation and Exploration of Lookup Table-Based Accelerators
  for 1.58-bit LLM Inference** (arXiv 2026 / IEEE ISPASS 2026,
  [https://arxiv.org/html/2604.25183](https://arxiv.org/html/2604.25183)) —
  open-source Chisel generator for LUT-based ternary matrix multiplication,
  validated in TSMC 16nm synthesis.
- **ELiTeFormer: An Efficient Transformer for FPGAs** (arXiv 2026,
  [https://arxiv.org/html/2607.03652](https://arxiv.org/html/2607.03652)) —
  hybrid linear attention + BitNet b1.58 ternary projections on Xilinx VCK5000;
  mentions a 5/3 packing scheme using 30/32 bits per dataframe.
- **KULeuven-MICAS/ternary-lut-dse** (GitHub, IEEE ISPASS 2026,
  [https://github.com/KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)) —
  open-source Chisel generator for LUT-based ternary matrix multiplication with
  non-power-of-two LUT depths.
- **SONIC: Event-Driven Gate-Level Simulator of Ternary VLSI Circuits using
  Delta Cycles** (IEEE ISMVL 2026,
  [DOI 10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042);
  [GitHub](https://github.com/sonbit/SimulationEngine)) — open-source
  multi-valued EDA simulator/verification backend, 120,000×–1.6M× speed-up
  over prior MRCS, with Verilog/Basys3 FPGA export.
- **REBEL-6: A 32-trit balanced ternary instruction set architecture** (IEEE ISMVL
  2025, [DOI 10.1109/ismvl64713.2025.00028](https://doi.org/10.1109/ismvl64713.2025.00028);
  [GitHub](https://github.com/Soppe/RV32IToREBEL)) — 32-trit balanced ternary ISA
  with open-source RV32I-to-REBEL compiler.
- **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI
  Circuits** (IEEE ISMVL 2026,
  [DOI 10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)) —
  balanced ternary extension to VHDL, verified with GHDL/GTKWave.
- **Yosys SystemVerilog gaps**: YosysHQ 0.65-dev documentation still flags arrays
  of packed structs/unions as unsupported, though recent commits are adding
  packed multi-dimensional arrays inside packed structs. t27’s scalar
  packed-vector flattening avoids those constructs entirely.

Sources:
- IEEE Std 1800-2017 SystemVerilog LRM (packed arrays / structs):
  [MIT-hosted PDF](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- AMD UG901 Vivado Synthesis 2026.1 — SystemVerilog Constructs:
  [docs.amd.com](https://docs.amd.com/r/en-US/ug901-vivado-synthesis/SystemVerilog-Constructs)
- AMD UG900 Vivado Logic Simulation 2026.1 — Packed Struct/Union:
  [docs.amd.com](https://docs.amd.com/r/en-US/ug900-vivado-logic-simulation/Packed-Struct/Union)
- AMD AR 51836 — Vivado Synthesis aggregate data types:
  [adaptivesupport.amd.com](https://adaptivesupport.amd.com/s/article/51836)
- Yosys Verilog support notes (0.65-dev):
  [yosyshq.readthedocs.io](https://yosyshq.readthedocs.io/projects/yosys/en/latest/using_yosys/verilog.html)

---

## 6. Three cooperation variants for Wave Loop 780

1. **Variant A (recommended): `[379][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (379 → 24,256 elements, 776,192
     bits, ~0.741 MiBit) and confirms non-power-of-two stride 379.

2. **Variant B: `[377][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W779 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[377][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.737 MiBit and adds conditional indexed signed field writes,
     verifying control-flow-guarded write emission in the Icarus path.

---

## 7. Definition of done

- [x] Witness generated and under version control.
- [x] Integration test added and passing.
- [x] Icarus lowerability, simulation, cocotb, and seal gates green.
- [x] Cargo suites green (except pre-existing `bitnet_pipeline` drift).
- [x] FROZEN_HASH unchanged.
- [x] Closeout report written.
- [x] `.trinity/experience.md` updated.
- [x] Next-wave cooperation variants defined.

---

phi^2 + 1/phi^2 = 3 | TRINITY

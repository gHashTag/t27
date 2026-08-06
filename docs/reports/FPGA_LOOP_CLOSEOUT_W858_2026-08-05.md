# FPGA Loop Close-out — Wave Loop 858 (2026-08-05)

**Issue:** #1656  
**Branch:** `wave-loop-858` (from `wave-loop-857` HEAD)  
**PR:** TBD  
**Variant:** A — module-scope `[535][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Summary

Wave Loop 858 continued the mechanical packed-vector ladder in the 1-MiBit range.
The witness is a module-scope `[535][2]^6 Pt` variable initialized from a function
call and exercised with indexed signed field writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 535 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 34,240 |
| Packed vector width | 1,095,680 bits |
| Approximate size | ~1.045 MiBit |
| Simulator cycles | 17 |

The wave required **zero compiler, reference-model, or `FROZEN_HASH` changes**.
All validation gates passed on the first run and the `icarus_lowerable` Rust suite
now reports **318/0**.

## Artifacts

- `scripts/gen_w858.py` — generator copied from `gen_w857.py` and verified for the
  recurring copy hazard (destination path, module header f-string, `MID_IDX`
  comment).
- `specs/scratch/w858_bench_module_535x2p6_aos_var_call_write.t27` — generated
  witness (101,711 lines, 2.35 MB).
- `.trinity/seals/scratch_w858_bench_module_535x2p6_aos_var_call_write.json` —
  seal saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w858_bench_module_535x2p6_aos_var_call_write`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | exit 0 |
| `t27c icarus-lowerable` | `lowerable` |
| `t27c icarus-simulate` | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `cargo test --release --test icarus_lowerable` | 318 passed; 0 failed |
| `FROZEN_HASH` | unchanged |

## Weak points investigated

### 1. The 1-MiBit range remains soft

W858 packed width is 1,095,680 bits (~1.045 MiBit). No backend limit was hit.
The established 4-MiBit soft cliff remains the next meaningful watch-point.

- **Icarus Verilog** does not have a documented 1-MiBit hard cap. The LRM
  (IEEE 1800-2017 §7.4.1/§7.4.2) only requires simulators to support packed
  arrays of at least 65,536 bits. Icarus currently warns around 1 Gbit, not
  1 Mbit ([steveicarus/iverilog#1171](https://github.com/steveicarus/iverilog/issues/1171),
  [steveicarus/iverilog#526](https://github.com/steveicarus/iverilog/issues/526)).
  Recent commit `128c621` fixed a bound-normalization path that could accidentally
  produce billion-bit vectors
  ([steveicarus/iverilog@128c621](https://github.com/steveicarus/iverilog/commit/128c621e8540b0a68145094fa876dc5de073c9a6)).
- **Historical Icarus 0.8 ceiling:** older versions had an assertion in `vector.c`
  around ~256 K entries for contiguous packed vectors
  ([gEDA-user Mar 2005](https://archives.seul.org/geda/user/Mar-2005/msg00150.html)).
  Modern Icarus 12.x/13.x no longer exhibits this limit, which is consistent with
  the clean passage of 1-MiBit packed vectors in the Wave Loop.

### 2. Simulator performance scaling

- Simulation cycle count stayed flat at **17 cycles**, matching W854–W857.
- The `.t27` parse dump is ~335 MB of AST text; this is an output-artifact cost,
  not a compiler workload regression.
- Icarus simulation runtime remained well under the 120-second tool timeout,
  confirming that the low 1-MiBit range is still comfortably within the
  event-driven simulator's memory model.

### 3. Recurring generator copy hazard

- The hazard was prevented by grepping the three known stale locations before
  running `scripts/gen_w858.py`.
- All references were verified to read `w858`, `535`, and `267`.

### 4. Pre-existing regressions (not fixed in this wave)

- `bootstrap/tests/verilog_array_literal_expr.rs` regression remains failing in
  the full `cargo test --release` run. It is unrelated to the AoS ladder and is
  tracked for a separate compiler-lowering issue.
- FPGA E2E CI remains red (`sby` missing + Yosys static-cast error in generated
  `uart.v`).

## Scientific / engineering background

1. **Icarus Verilog packed-array sizing.** The SystemVerilog LRM only mandates
   a minimum packed-array support of 65,536 bits; Icarus warns around 1 Gbit. Its
   recent bound-normalization fix (`128c621`) shows that the real width hazards
   live in edge-case constant math, not in ordinary wide packed arrays. A
   historical Icarus 0.8 allocator assertion at ~256 K entries is no longer
   relevant for modern versions.
2. **FPGA Roofline model.** Siracusa et al. (IEEE TC 2021, DOI
   [10.1109/TC.2021.3111761](https://doi.org/10.1109/tc.2021.3111761)) model
   off-chip bandwidth with memory quanta `Q`, peak configuration bandwidth
   `BW_interface = min(f × Q, BW_bank × min(1, Q/W))`, and ceilings for random
   and data-dependent access. The Wave Loop ladder is effectively probing how
   large `Q` can grow before routing/host-memory costs dominate.
3. **Verified compilation to Verilog.** CompCert (Leroy, *A formally verified
   compiler back-end*, 2009, [PDF](https://xavierleroy.org/publi/compcert-backend.pdf))
   is the foundational verified compiler back-end. Vericert (Herklotz et al.,
   OOPSLA 2021, DOI [10.1145/3485494](https://doi.org/10.1145/3485494),
   [PDF](https://johnwickerson.github.io/papers/vericert_oopsla21.pdf)) extends
   CompCert with a Verilog backend and an end-to-end C-to-hardware correctness
   proof. Its bit-exact correctness criterion is the long-term analog of the
   t27 packed-vector identity checks in the Wave Loop witnesses.
4. **Vitis HLS aggregate packing.** Xilinx UG1399 documents `compact=bit` for
   interface structs and the internal AoS-to-SoA transformation. The t27 ladder
   keeps AoS layout at the language level while the generated Verilog uses
   packed vectors, analogous to the HLS `compact=bit` bit-field layout decision.

## Next wave cooperation variants (Wave Loop 859)

- **Variant A (recommended):** `[537][2]^6 Pt`, outer dimension += 2, `MID_IDX = 268`.
  Continues the mechanical packed-vector ladder with the smallest reviewable diff.
- **Variant B:** `[535][3]^6 Pt` — grow the second inner dimension from 2 to 3 to
  stress stride scaling and total element count (535 × 192 = 102,720 elements,
  ~3.13 MiBit), probing the approach to the 4-MiBit soft cliff.
- **Variant C:** `[535][2]^6 Pt` with negative-index writes to exercise wrap-around
  addressing and signed-index semantics in the reference model.

## Action items

1. Stage all W858 artifacts.
2. Commit with `Closes #1656` and push `wave-loop-858`.
3. Open PR to `master` with validation matrix and `Closes #1656`.
4. Create issue #1658 / branch `wave-loop-859` for Variant A.

*φ² + φ⁻² = 3 | TRINITY*

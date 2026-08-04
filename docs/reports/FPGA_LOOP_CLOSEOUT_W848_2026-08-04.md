# FPGA Loop Close-out — Wave Loop 848 (2026-08-04)

**Issue:** #1636  
**Branch:** `wave-loop-848` (from `wave-loop-847` HEAD)  
**PR:** #1637  
**Variant:** A — module-scope `[515][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Summary

Wave Loop 848 continued the mechanical packed-vector ladder past the 1-MiBit line.
The witness is a module-scope `[515][2]^6 Pt` variable initialized from a function
call and exercised with indexed signed field writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 515 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 32,960 |
| Packed vector width | 1,054,720 bits |
| Approximate size | ~1.006 MiBit |
| Simulator cycles | 17 |

The wave required **zero compiler, reference-model, or `FROZEN_HASH` changes**.
All validation gates passed on the first run and the `icarus_lowerable` Rust suite
now reports **308/0**.

## Artifacts

- `scripts/gen_w848.py` — generator copied from `gen_w847.py` and verified for the
  recurring copy hazard (destination path, module header f-string, `MID_IDX`
  comment).
- `specs/scratch/w848_bench_module_515x2p6_aos_var_call_write.t27` — generated
  witness (97,911 lines, 2.26 MB).
- `.trinity/seals/scratch_w848_bench_module_515x2p6_aos_var_call_write.json` —
  seal saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w848_bench_module_515x2p6_aos_var_call_write`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | exit 0 |
| `t27c icarus-lowerable` | `lowerable` |
| `t27c icarus-simulate` | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `cargo test --release --test icarus_lowerable` | 308 passed; 0 failed |
| `FROZEN_HASH` | unchanged |

## Weak points investigated

### 1. The 1-MiBit boundary is a soft threshold, not a cliff

W848 packed width is 1,054,720 bits (~1.006 MiBit). No backend limit was hit.
The established 4-MiBit soft cliff remains the next meaningful watch-point.

- **Icarus Verilog** does not have a documented 1-MiBit hard cap. The LRM
  (IEEE 1800-2017 §7.4.1/§7.4.2) only requires simulators to support packed
  arrays of at least 65,536 bits. Icarus currently warns around 1 Gbit, not
  1 Mbit ([steveicarus/iverilog#1171](https://github.com/steveicarus/iverilog/issues/1171),
  [steveicarus/iverilog#60](https://github.com/steveicarus/iverilog/issues/60)).
  Recent commit `128c621` fixed a pathological packed-array bound normalization
  bug where negative bounds could produce colossal widths
  ([steveicarus/iverilog@128c621](https://github.com/steveicarus/iverilog/commit/128c621e8540b0a68145094fa876dc5de073c9a6)).
- Practical limits are memory- and compile-time-bound rather than a clean width
  ceiling.

### 2. Simulator performance scaling

- Simulation cycle count stayed flat at **17 cycles**, matching W846 and W847.
- The `.t27` parse dump is ~320 MB of AST text; this is an output-artifact cost,
  not a compiler workload regression.
- Icarus simulation runtime remained well under the 120-second tool timeout,
  confirming that the low 1-MiBit range is still comfortably within the
  event-driven simulator's memory model.

### 3. Recurring generator copy hazard

- The hazard was prevented by grepping the three known stale locations before
  running `scripts/gen_w848.py`.
- All references were verified to read `w848`, `515`, and `257`.

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
   live in edge-case constant math, not in ordinary wide packed arrays.
2. **FPGA Roofline model.** Siracusa et al. (IEEE TC 2021, DOI
   [10.1109/TC.2021.3111761](https://doi.org/10.1109/tc.2021.3111761)) model
   off-chip bandwidth with memory quanta `Q`, peak configuration bandwidth
   `BW_interface = min(f × Q, BW_bank × min(1, Q/W))`, and ceilings for random
   and data-dependent access. The Wave Loop ladder is effectively probing how
   large `Q` can grow before routing/host-memory costs dominate.
3. **Verified HLS.** Vericert (Herklotz et al., OOPSLA 2021, DOI
   [10.1145/3485494](https://doi.org/10.1145/3485494)) provides a mechanically
   verified C-to-Verilog compiler. Its bit-exact correctness criterion is the
   long-term analog of the t27 packed-vector identity checks in the Wave Loop
   witnesses.
4. **Vitis HLS aggregate packing.** Xilinx UG1399 documents `compact=bit` for
   interface structs and the internal AoS-to-SoA transformation. The t27 ladder
   keeps AoS layout at the language level while the generated Verilog uses
   packed vectors, matching the interface-aggregation pattern.

## Cooperation variants for Wave Loop 849

- **A (recommended):** `[517][2]^6 Pt`, outer += 2, `MID_IDX = 258`.
  - 33,088 elements, 1,058,816 bits (~1.010 MiBit).
  - Continues the established mechanical increment.

- **B:** `[515][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
  - 98,880 elements, 3,164,160 bits (~3.016 MiBit).
  - Deliberately much wider; useful as a width-probe that may convert to a
    negative-boundary witness if a backend limit is hit.

- **C:** `[515][2]^6 Pt` with negative-index writes to exercise wrap-around / signed-index lowering.
  - Keeps the W848 outer dimension but replaces some positive indices with
    negative ones to stress the lowerer's signed-index path.

## Seal hashes

```text
spec_hash=sha256:852f3e63821201454fb105e8c129e1eeb0222a33fbdc1c345316350953a3b163
gen_hash_zig=sha256:bdda8e7e073a5648600ddfe6180f0e2b4227a29219e75b821a16fb7b0f1443ae
gen_hash_verilog=sha256:6bcfaf4883d2ac65b767d804102fcb82ef3500aaca2ccad97d369b82ef19cdb9
gen_hash_c=sha256:6b236f6bdf130216db551d90b77c2b4fd630dfcb4a1f3c3fae885d409f1eb3c9
gen_hash_rust=sha256:99f0b4cfd51f994a144fe93d9673aee1f09b04e1e0fe4ff91b172221e5685dbb
```

## Next steps

1. Open PR #1637 to `master` with `Closes #1636`.
2. Create GitHub issue #1638 for Wave Loop 849.
3. Branch `wave-loop-849` from `wave-loop-848` HEAD and execute the selected
   W849 variant.

*φ² + φ⁻² = 3 | TRINITY*

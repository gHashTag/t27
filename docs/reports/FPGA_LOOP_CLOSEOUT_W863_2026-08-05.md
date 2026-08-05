# FPGA Loop Close-out — Wave Loop 863 (2026-08-05)

**Issue:** #1670  
**Branch:** `wave-loop-863` (from `wave-loop-862` HEAD)  
**PR:** TBD  
**Variant:** A — module-scope `[545][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Summary

Wave Loop 863 continued the mechanical packed-vector ladder in the 1-MiBit range.
The witness is a module-scope `[545][2]^6 Pt` variable initialized from a function
call and exercised with indexed signed field writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 545 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 34,880 |
| Packed vector width | 1,116,160 bits |
| Approximate size | ~1.064 MiBit |
| Simulator cycles | 17 |

The wave required **zero compiler, reference-model, or `FROZEN_HASH` changes**.
All validation gates passed on the first run and the `icarus_lowerable` Rust suite
now reports **323/0**.

## Artifacts

- `scripts/gen_w863.py` — generator copied from `gen_w862.py` and verified for the
  recurring copy hazard (destination path, module header f-string, `MID_IDX`
  comment).
- `specs/scratch/w863_bench_module_545x2p6_aos_var_call_write.t27` — generated
  witness (103,611 lines, 2.28 MB).
- `.trinity/seals/scratch_w863_bench_module_545x2p6_aos_var_call_write.json` —
  seal saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w863_bench_module_545x2p6_aos_var_call_write`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | exit 0 |
| `t27c icarus-lowerable` | `lowerable` |
| `t27c icarus-simulate` | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `cargo test --release --test icarus_lowerable` | 323 passed; 0 failed |
| `FROZEN_HASH` | unchanged |

## Weak points investigated

### 1. The 1-MiBit range remains soft

W863 packed width is 1,116,160 bits (~1.064 MiBit). No backend limit was hit.
The established 4-MiBit soft cliff remains the next meaningful watch-point.

- **Icarus Verilog** does not have a documented 1-MiBit hard cap. The LRM
  (IEEE 1800-2017 §7.4.1/§7.4.2) only requires simulators to support packed
  arrays of at least 65,536 bits. Icarus currently warns around 1 Gbit, not
  1 Mbit.
- **Historical Icarus 0.8 ceiling:** older versions had an assertion in `vector.c`
  around ~256 K entries for contiguous packed vectors; modern Icarus 12.x/13.x
  no longer exhibits this limit.

### 2. Simulator performance scaling

- Simulation cycle count stayed flat at **17 cycles**, matching W854–W862.
- Icarus simulation runtime remained well under the 120-second tool timeout,
  confirming that the low 1-MiBit range is still comfortably within the
  event-driven simulator's memory model.

### 3. Recurring generator copy hazard

- The hazard was prevented by grepping the three known stale locations before
  running `scripts/gen_w863.py`.
- All references were verified to read `w863`, `545`, and `272`.

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
   are implementation bugs, not protocol limits, in the 1-MiBit range.

2. **FPGA Roofline.** Siracusa et al. (IEEE TC 2021, DOI
   10.1109/TC.2021.3111761) model bandwidth-bound FPGA kernels with
   `II = max(compute, memory)`; the Wave Loop is essentially increasing the
   memory quanta `Q` while keeping compute flat, so it measures the point where
   routing/host-memory overheads dominate.

3. **Verified compilation analogs.** CompCert (Leroy 2009) and Vericert
   (Herklotz et al. OOPSLA 2021) demonstrate end-to-end bit-exact
   source-to-hardware correctness. Their existence means the ladder is not only
   an empirical stress test but also a reproducible regression corpus.

4. **HLS struct packing.** Xilinx UG1399 documents `compact=bit` and the
   AoS-to-SoA transformation for interface structs. The `[N][2]^6 Pt` packed
   vector is a contrived worst-case for an HLS bit-packer, which makes it a
   useful regression witness.

## Cooperation variants for Wave Loop 864

| Variant | Shape | Outer | `MID_IDX` | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-----------|----------|------|-------|---------|
| **A (recommended)** | `[547][2]^6 Pt` | 547 | 273 | 35,008 | 1,120,256 | ~1.068 | Mechanical outer += 2 increment. |
| **B** | `[545][3]^6 Pt` | 545 | — | 52,416 | 1,677,312 | ~1.599 | Grow the second inner dimension to stress stride scaling. |
| **C** | `[545][2]^6 Pt` (neg-index writes) | 545 | 272 | 34,880 | 1,116,160 | ~1.064 | Negative-index writes to exercise wrap-around addressing. |

Variant A is recommended because it preserves the established mechanical ladder,
keeps the packed width just past 1 MiBit, and continues to probe the same
implementation path without introducing new variables.

## Next steps

1. Create issue #1672 and branch `wave-loop-864` from `wave-loop-863` HEAD.
2. Implement variant A (`[547][2]^6 Pt`, `MID_IDX = 273`).
3. Run the validation matrix and seal.
4. Update skill/memory trackers and persistent memory.

*φ² + φ⁻² = 3 | TRINITY*

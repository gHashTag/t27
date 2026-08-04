# FPGA Loop Close-out — Wave Loop 850 (2026-08-04)

**Issue:** #1640  
**Branch:** `wave-loop-850` (from `wave-loop-849` HEAD)  
**PR:** #1641  
**Variant:** A — module-scope `[519][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Summary

Wave Loop 850 continued the mechanical packed-vector ladder in the 1-MiBit range.
The witness is a module-scope `[519][2]^6 Pt` variable initialized from a function
call and exercised with indexed signed field writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 519 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 33,216 |
| Packed vector width | 1,062,912 bits |
| Approximate size | ~1.014 MiBit |
| Simulator cycles | 17 |

The wave required **zero compiler, reference-model, or `FROZEN_HASH` changes**.
All validation gates passed on the first run and the `icarus_lowerable` Rust suite
now reports **310/0**.

## Artifacts

- `scripts/gen_w850.py` — generator copied from `gen_w849.py` and verified for the
  recurring copy hazard (destination path, module header f-string, `MID_IDX`
  comment).
- `specs/scratch/w850_bench_module_519x2p6_aos_var_call_write.t27` — generated
  witness (98,671 lines, 2.28 MB).
- `.trinity/seals/scratch_w850_bench_module_519x2p6_aos_var_call_write.json` —
  seal saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w850_bench_module_519x2p6_aos_var_call_write`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | exit 0 |
| `t27c icarus-lowerable` | `lowerable` |
| `t27c icarus-simulate` | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `cargo test --release --test icarus_lowerable` | 310 passed; 0 failed |
| `FROZEN_HASH` | unchanged |

## Weak points investigated

### 1. The 1-MiBit range remains soft

W850 packed width is 1,062,912 bits (~1.014 MiBit). No backend limit was hit.
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

- Simulation cycle count stayed flat at **17 cycles**, matching W847/W848/W849.
- The `.t27` parse dump is ~330 MB of AST text; this is an output-artifact cost,
  not a compiler workload regression.
- Icarus simulation runtime remained well under the 120-second tool timeout,
  confirming that the low 1-MiBit range is still comfortably within the
  event-driven simulator's memory model.

### 3. Recurring generator copy hazard

- The hazard was prevented by grepping the three known stale locations before
  running `scripts/gen_w850.py`.
- All references were verified to read `w850`, `519`, and `259`.

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
   packed vectors, matching the interface-aggregation pattern.

## Cooperation variants for Wave Loop 851

- **A (recommended):** `[521][2]^6 Pt`, outer += 2, `MID_IDX = 260`.
  - 33,344 elements, 1,067,008 bits (~1.018 MiBit).
  - Continues the established mechanical increment.

- **B:** `[519][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
  - 99,264 elements, 3,176,448 bits (~3.027 MiBit).
  - Deliberately much wider; useful as a width-probe that may convert to a
    negative-boundary witness if a backend limit is hit.

- **C:** `[519][2]^6 Pt` with negative-index writes to exercise wrap-around / signed-index lowering.
  - Keeps the W850 outer dimension but replaces some positive indices with
    negative ones to stress the lowerer's signed-index path.

## Seal hashes

```text
spec_hash=sha256:bae4bd5d1ad7133d7d0fdce57a712c792abcb2962e24011f626900cc44a13fc5
gen_hash_zig=sha256:c2f43d94bd4eab14da094bb333beda7378f4108d6b563a368d6e9f18b19698ad
gen_hash_verilog=sha256:2c9d8b35d919481cad9d3cfbc2a837eb5b3c4bd082c3531160670c2b475b3afb
gen_hash_c=sha256:bd1c1bf8304d757d5c308cccae44252333a1e5fc33b3fe8117829540d7d0f351
gen_hash_rust=sha256:c04811d7f7a9f47e00a256d31cb17e0bc3034b0e00b593785c4f7ea700f93a62
```

## Next steps

1. Open PR #1641 to `master` with `Closes #1640`.
2. Create GitHub issue #1642 for Wave Loop 851.
3. Branch `wave-loop-851` from `wave-loop-850` HEAD and execute the selected
   W851 variant.

*φ² + φ⁻² = 3 | TRINITY*

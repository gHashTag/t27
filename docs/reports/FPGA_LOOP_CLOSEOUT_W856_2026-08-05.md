# FPGA Loop Close-out — Wave Loop 856 (2026-08-05)

**Issue:** #1652  
**Branch:** `wave-loop-856` (from `wave-loop-855` HEAD)  
**PR:** #1653  
**Variant:** A — module-scope `[531][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Summary

Wave Loop 856 continued the mechanical packed-vector ladder in the 1-MiBit range.
The witness is a module-scope `[531][2]^6 Pt` variable initialized from a function
call and exercised with indexed signed field writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 531 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 33,984 |
| Packed vector width | 1,087,488 bits |
| Approximate size | ~1.038 MiBit |
| Simulator cycles | 17 |

The wave required **zero compiler, reference-model, or `FROZEN_HASH` changes**.
All validation gates passed on the first run and the `icarus_lowerable` Rust suite
now reports **316/0**.

## Artifacts

- `scripts/gen_w856.py` — generator copied from `gen_w855.py` and verified for the
  recurring copy hazard (destination path, module header f-string, `MID_IDX`
  comment).
- `specs/scratch/w856_bench_module_531x2p6_aos_var_call_write.t27` — generated
  witness (100,951 lines, 2.33 MB).
- `.trinity/seals/scratch_w856_bench_module_531x2p6_aos_var_call_write.json` —
  seal saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w856_bench_module_531x2p6_aos_var_call_write`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | exit 0 |
| `t27c icarus-lowerable` | `lowerable` |
| `t27c icarus-simulate` | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `cargo test --release --test icarus_lowerable` | 316 passed; 0 failed |
| `FROZEN_HASH` | unchanged |

## Weak points investigated

### 1. The 1-MiBit range remains soft

W856 packed width is 1,087,488 bits (~1.038 MiBit). No backend limit was hit.
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

- Simulation cycle count stayed flat at **17 cycles**, matching W853/W854/W855.
- The `.t27` parse dump is ~330 MB of AST text; this is an output-artifact cost,
  not a compiler workload regression.
- Icarus simulation runtime remained well under the 120-second tool timeout,
  confirming that the low 1-MiBit range is still comfortably within the
  event-driven simulator's memory model.

### 3. Recurring generator copy hazard

- The hazard was prevented by grepping the three known stale locations before
  running `scripts/gen_w856.py`.
- All references were verified to read `w856`, `531`, and `265`.

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

## Cooperation variants for Wave Loop 857

- **A (recommended):** `[533][2]^6 Pt`, outer += 2, `MID_IDX = 266`.
  - 34,112 elements, 1,091,584 bits (~1.042 MiBit).
  - Continues the established mechanical increment.

- **B:** `[531][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
  - 101,664 elements, 3,253,248 bits (~3.101 MiBit).
  - Deliberately much wider; useful as a width-probe that may convert to a
    negative-boundary witness if a backend limit is hit.

- **C:** `[531][2]^6 Pt` with negative-index writes to exercise wrap-around / signed-index lowering.
  - Keeps the W856 outer dimension but replaces some positive indices with
    negative ones to stress the lowerer's signed-index path.

## Seal hashes

```text
spec_hash=sha256:9d07ceb9f63735ef56c644050a36611477952523173a062c3c6f138808e3fb5e
gen_hash_zig=sha256:2b670cd36e79540da1b334923803bfcd5eb687c7c8294f371331e26bc943b2d2
gen_hash_verilog=sha256:9a2fb62ac6f9e05435f9ad48890c65fc38665cab035f0ebcd482e0e80246a0b8
gen_hash_c=sha256:be5edde507effdef0bdc700d1cbdb1c4762d2cda4fd41e9731337a84230fb5af
gen_hash_rust=sha256:1670558cd3a05af1e453708367195ca97b51f6468ad2e9528538cb03448eed1e
```

## Next steps

1. Open PR #1653 to `master` with `Closes #1652`.
2. Create GitHub issue #1654 for Wave Loop 857.
3. Branch `wave-loop-857` from `wave-loop-856` HEAD and execute the selected
   W857 variant.

*φ² + φ⁻² = 3 | TRINITY*

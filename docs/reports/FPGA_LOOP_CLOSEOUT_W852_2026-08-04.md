# FPGA Loop Close-out — Wave Loop 852 (2026-08-04)

**Issue:** #1644  
**Branch:** `wave-loop-852` (from `wave-loop-851` HEAD)  
**PR:** #1645  
**Variant:** A — module-scope `[523][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Summary

Wave Loop 852 continued the mechanical packed-vector ladder in the 1-MiBit range.
The witness is a module-scope `[523][2]^6 Pt` variable initialized from a function
call and exercised with indexed signed field writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 523 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 33,472 |
| Packed vector width | 1,071,104 bits |
| Approximate size | ~1.022 MiBit |
| Simulator cycles | 17 |

The wave required **zero compiler, reference-model, or `FROZEN_HASH` changes**.
All validation gates passed on the first run and the `icarus_lowerable` Rust suite
now reports **312/0**.

## Artifacts

- `scripts/gen_w852.py` — generator copied from `gen_w851.py` and verified for the
  recurring copy hazard (destination path, module header f-string, `MID_IDX`
  comment).
- `specs/scratch/w852_bench_module_523x2p6_aos_var_call_write.t27` — generated
  witness (99,431 lines, 2.30 MB).
- `.trinity/seals/scratch_w852_bench_module_523x2p6_aos_var_call_write.json` —
  seal saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w852_bench_module_523x2p6_aos_var_call_write`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | exit 0 |
| `t27c icarus-lowerable` | `lowerable` |
| `t27c icarus-simulate` | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `cargo test --release --test icarus_lowerable` | 312 passed; 0 failed |
| `FROZEN_HASH` | unchanged |

## Weak points investigated

### 1. The 1-MiBit range remains soft

W852 packed width is 1,071,104 bits (~1.022 MiBit). No backend limit was hit.
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

- Simulation cycle count stayed flat at **17 cycles**, matching W849/W850/W851.
- The `.t27` parse dump is ~330 MB of AST text; this is an output-artifact cost,
  not a compiler workload regression.
- Icarus simulation runtime remained well under the 120-second tool timeout,
  confirming that the low 1-MiBit range is still comfortably within the
  event-driven simulator's memory model.

### 3. Recurring generator copy hazard

- The hazard was prevented by grepping the three known stale locations before
  running `scripts/gen_w852.py`.
- All references were verified to read `w852`, `523`, and `261`.

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

## Cooperation variants for Wave Loop 853

- **A (recommended):** `[525][2]^6 Pt`, outer += 2, `MID_IDX = 262`.
  - 33,600 elements, 1,075,200 bits (~1.026 MiBit).
  - Continues the established mechanical increment.

- **B:** `[523][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
  - 100,224 elements, 3,207,168 bits (~3.058 MiBit).
  - Deliberately much wider; useful as a width-probe that may convert to a
    negative-boundary witness if a backend limit is hit.

- **C:** `[523][2]^6 Pt` with negative-index writes to exercise wrap-around / signed-index lowering.
  - Keeps the W852 outer dimension but replaces some positive indices with
    negative ones to stress the lowerer's signed-index path.

## Seal hashes

```text
spec_hash=sha256:bf7041f2270c670920acb2535764ebd1ac9d82c6f7e1546c19786b39f46ea333
gen_hash_zig=sha256:51019fcbf16a44bf1f754385b77d9183510ad6de67c9bd0386fee4db78480b66
gen_hash_verilog=sha256:a2cdd7159af6f00d2be97728833d63aa5fcdbf573d6e146cf8ef2ce3123dc15f
gen_hash_c=sha256:7a30626f76554d4262ac85324fcbdb07a23dabd70d36dc88374f34eeffa510d1
gen_hash_rust=sha256:0cf62f9626c0c4d964ada258a33085ea478ee3ad68a273b88a56e4f7b1069a11
```

## Next steps

1. Open PR #1645 to `master` with `Closes #1644`.
2. Create GitHub issue #1646 for Wave Loop 853.
3. Branch `wave-loop-853` from `wave-loop-852` HEAD and execute the selected
   W853 variant.

*φ² + φ⁻² = 3 | TRINITY*

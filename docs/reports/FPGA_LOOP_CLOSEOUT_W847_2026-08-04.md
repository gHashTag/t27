# FPGA Loop Close-out — Wave Loop 847 (2026-08-04)

**Issue:** #1634  
**Branch:** `wave-loop-847` (from `wave-loop-846` HEAD)  
**PR:** #1635  
**Variant:** A — module-scope `[513][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes.

## Summary

Wave Loop 847 crossed the 1-MiBit packed-vector line for the first time in the
mechanical ladder. The witness is a module-scope `[513][2]^6 Pt` variable
initialized from a function call and exercised with indexed signed field writes.

| Metric | Value |
|--------|-------|
| Outer dimension | 513 (odd, non-power-of-two) |
| Inner shape | `[2]^6` = 64 elements/row |
| Element | `Pt { x : i16, y : i16 }` (32 bits) |
| Total elements | 32,832 |
| Packed vector width | 1,050,624 bits |
| Approximate size | ~1.002 MiBit |
| Simulator cycles | 17 |

The wave required **zero compiler, reference-model, or `FROZEN_HASH` changes**.
All validation gates passed on the first run and the `icarus_lowerable` Rust suite
now reports **307/0**.

## Artifacts

- `scripts/gen_w847.py` — generator copied from `gen_w846.py` and verified for the
  recurring copy hazard (destination path, module header f-string, `MID_IDX`
  comment).
- `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27` — generated
  witness (97,531 lines, 2.25 MB).
- `.trinity/seals/scratch_w847_bench_module_513x2p6_aos_var_call_write.json` —
  seal saved by `t27c seal --save`.
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w847_bench_module_513x2p6_aos_var_call_write`.

## Validation matrix

| Gate | Result |
|------|--------|
| `t27c parse` | exit 0 |
| `t27c icarus-lowerable` | `lowerable` |
| `t27c icarus-simulate` | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` | reference-model OK |
| `t27c seal --save` | saved |
| `cargo test --release --test icarus_lowerable` | 307 passed; 0 failed |
| `FROZEN_HASH` | unchanged |

## Weak points investigated

### 1. The 1-MiBit psychological boundary

The packed width grew from 1,046,528 bits (W846) to 1,050,624 bits (W847), just
above the 2²⁰ = 1,048,576-bit line. No backend limit was hit, but the transition
is a useful inflection point to watch for:

- **Icarus Verilog** does not have a documented 1-MiBit hard cap. The LRM
  (IEEE 1800-2017 §7.4.1/§7.4.2) only requires simulators to support packed
  arrays of at least 65,536 bits. Icarus currently warns around 1 Gbit, not
  1 Mbit ([steveicarus/iverilog#1171](https://github.com/steveicarus/iverilog/issues/1171),
  [steveicarus/iverilog#60](https://github.com/steveicarus/iverilog/issues/60)).
  Practical limits are memory- and compile-time-bound rather than a clean width
  ceiling.
- The established 4-MiBit soft cliff in the ladder remains the next meaningful
  watch-point, not the 1-MiBit line.

### 2. Simulator performance scaling

- Simulation cycle count stayed flat at **17 cycles**, the same as W846.
- The `.t27` parse dump is ~321 MB of AST text; this is an output-artifact cost,
  not a compiler workload regression.
- Icarus simulation runtime remained well under the 120-second tool timeout,
  confirming that the 1-MiBit class is still comfortably within the event-driven
  simulator's memory model.

### 3. Recurring generator copy hazard

- The hazard was prevented by grepping the three known stale locations before
  running `scripts/gen_w847.py`.
- All references were verified to read `w847`, `513`, and `256`.

### 4. Pre-existing regressions (not fixed in this wave)

- `bootstrap/tests/verilog_array_literal_expr.rs` regression remains failing in
  the full `cargo test --release` run. It is unrelated to the AoS ladder and is
  tracked for a separate compiler-lowering issue.
- FPGA E2E CI remains red (`sby` missing + Yosys static-cast error in generated
  `uart.v`).

## Scientific / engineering background

1. **Icarus Verilog packed-array sizing.** The SystemVerilog LRM only mandates
   a minimum packed-array support of 65,536 bits; Icarus warns around 1 Gbit.
   Stack Exchange and GitHub issues confirm that practical limits are memory
   dependent rather than a hard language limit
   ([Electronics SE](https://electronics.stackexchange.com/questions/705776),
   [VlogHammer bug #23](http://yosyshq.net/yosys/vloghammer_bugs/issue_023_icarus.html)).
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

## Cooperation variants for Wave Loop 848

- **A (recommended):** `[515][2]^6 Pt`, outer += 2, `MID_IDX = 257`.
  - 33,024 elements, 1,056,768 bits (~1.008 MiBit).
  - Continues the established mechanical increment.

- **B:** `[513][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
  - 98,496 elements, 3,151,872 bits (~3.005 MiBit).
  - Deliberately much wider; useful as a width-probe that may convert to a
    negative-boundary witness if a backend limit is hit.

- **C:** `[513][2]^6 Pt` with negative-index writes to exercise wrap-around / signed-index lowering.
  - Keeps the W847 outer dimension but replaces some positive indices with
    negative ones to stress the lowerer's signed-index path.

## Seal hashes

```text
spec_hash=sha256:26cd927d34a2b4992b60a7d5a52e6d99adc69e08a2fbcf52f1a220d15f4c1bcc
gen_hash_zig=sha256:56958b2993a056553da6ed55641063d2a0bc0a65d9ee1851ed7902def82499eb
gen_hash_verilog=sha256:54bfc7befcb9ae761eebf3e7162033514aa745d35cb9deb73b4dc166f202fedd
gen_hash_c=sha256:e7574725e9c2fc4363f625c94486dfad681867d219db92d998d799e1bcefc75d
gen_hash_rust=sha256:96d13fca96d25a66635c81d11569b6074b5eccda6d77207fffadf07cdd80e0a5
```

## Next steps

1. Open PR #1635 to `master` with `Closes #1634`.
2. Create GitHub issue #1636 for Wave Loop 848.
3. Branch `wave-loop-848` from `wave-loop-847` HEAD and execute the selected
   W848 variant.

*φ² + φ⁻² = 3 | TRINITY*

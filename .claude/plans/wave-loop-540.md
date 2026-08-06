# Wave Loop 540 — Multi-signal VCD probes for wide packed structs and arrays

**Issue:** #1511  
**Branch:** `wave-loop-540`  
**Status:** in-progress  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Close the last major gap in the cocotb reference-model coverage: wide packed
structs and arrays (width > 64 bits). Instead of skipping these assertions,
emit a deterministic set of 64-bit (or smaller) slice probes and reconstruct
the full value in the Python reference model.

---

## Literature & prior art (investigated)

1. **cocotb issue #2302 — *Simulator-independent bit-slicing, shadow signals, and splitting concatenated buses***  
   <https://github.com/cocotb/cocotb/issues/2302>  
   Wide buses in cocotb testbenches must often be split into per-port slices;
   the issue proposes `ShadowSignal` / `bus_split` helpers. Reinforces the
   need for explicit slice metadata in the reference model.

2. **cocotb-bus issue #10 — *Support signal splitting***  
   <https://github.com/cocotb/cocotb-bus/issues/10>  
   Complementary discussion on `SplitSignal` / `SplitBus`; VPI/VHPI/FLI do not
   natively expose slices, so reconstruction must happen in Python.

3. **Verification Academy — *Specifying bit ranges of arrays in VCD***  
   <https://verificationacademy.com/forums/t/specifying-bit-ranges-of-arrays-in-vcd/50387/1>  
   VCD `$var` can declare a range (`identifier[msb:lsb]`) or plain identifier
   with total size; useful for understanding how simulators dump wide vectors.

4. **GTKWave docs — *Combine Down / grouped vectors***  
   <https://gtkwave.github.io/gtkwave/quickstart/filters.html>  
   Waveform viewers routinely reconstruct wide buses from individual slice
   traces; the same concatenation idea applies to reference models.

5. **`pyvcd` `vcd.reader`**  
   <https://pyvcd.readthedocs.io/en/latest/vcd.reader.html>  
   A Python VCD reader that already parses `bit_index` (single bit or range)
   from `$var` declarations; validates that slice metadata is the right lever
   for reconstruction.

Key takeaways for W540:
- The backend must emit probes with explicit slice offsets/widths.
- The reference model must concatenate slices in a deterministic order.
- We do not need external VCD libraries; our minimal parser is sufficient if
  we encode slice metadata in the probe name or in a comment.

---

## Decomposed plan

### Phase 1: Compiler — multi-signal probe emission
Files: `bootstrap/src/compiler.rs`

- Keep the existing typed single-probe path for expressions whose inferred
  width is ≤ 64 bits.
- When `expr_width_signed` returns a width > 64 bits, emit N probe registers
  that together cover the full width. Use the naming convention
  `_t27_probe_<block>_<idx>_s<M>`, where M is the zero-based slice index.
- Record in `probe_specs` a list entry per slice:
  `(probe_name, slice_width, signed, slice_offset)`.
- Emit an assignment per slice from a part-select of the actual expression:
  `_t27_probe_..._s0 = actual[0 +: 64]; _t27_probe_..._s1 = actual[64 +: 64]; ...`
- For non-scalar / unresolvable expressions, fall back to the previous 64-bit
  single probe so existing behavior is preserved.

### Phase 2: Reference model — wide-value reconstruction
Files: `scripts/cocotb_ref_model.py`

- Extend `_VcdParser` to recognize slice-probe names (`_s<N>` suffix) and
  record `(value, width, offset)` for each.
- Add `_collect_assertions` to mark assertions whose actual is wide; store a
  list of slice probe names/offsets instead of a single probe name.
- In `_cross_check`, when the assertion is wide, read all slice probes, shift
  each slice by its offset, OR them together, mask to the full width, sign
  extend if the full expression is signed, and compare against the expected
  bit-vector value.
- Keep the existing single-probe path for ≤ 64-bit assertions.

### Phase 3: Witness
Files: `specs/scratch/w540_wide_packed_struct_array.t27`

- Define a scalar struct with several fields such that a packed array-of-struct
  element (or whole array) exceeds 64 bits.
- Write an `assert_eq(actual, expected)` where `actual` is a whole struct, a
  whole array, or a multi-element slice whose width is > 64 bits.
- Because the Python evaluator can already handle struct/array literals, the
  expected value can be computed as a `Bv` and compared against reconstructed
  slices.
- Seal the spec.

### Phase 4: Validation
- `cargo build --release -p t27c` (update `FROZEN_HASH` if compiler changes).
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness`

### Phase 5: Closeout & next-wave cooperation variants
- Write `docs/reports/WAVE_LOOP_540_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W541_2026-07-08.md`.
- Advance `.trinity/current-issue.md` to W541.
- Update `.trinity/experience.md`, persistent memory, and
  `.claude/skills/t27-wave-loop.md`.

---

## Risks

- Part-select on arbitrary expressions (`(a+b)[127:64]`) is not legal Verilog
  without a temporary variable. We must only slice plain identifiers or
  parenthesize the expression and assign it to a temporary packed reg first.
- The Python evaluator must compute the expected struct/array literal as a
  packed bit-vector in exactly the same order (reverse field order, LSB-first)
  as the Verilog backend.
- Slice reconstruction must handle the final partial slice (< 64 bits)
  correctly.
- Probe name changes may require resealing specs whose Verilog hash changes.

---

*φ² + φ⁻² = 3 | TRINITY*

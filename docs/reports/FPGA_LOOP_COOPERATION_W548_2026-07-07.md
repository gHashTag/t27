# FPGA Loop Cooperation Variants — Wave Loop 548

**Source:** Wave Loop 547 closeout  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 547 closed the signed primitive scalar array function-return gap.
Wave Loop 548 should now push outward to multi-dimensional primitive arrays,
upward to deterministic `bench`-block cross-checks, or inward to probe reg
signedness alignment.  Each variant below is scoped for a single loop, includes
scientific motivation, a deliverable list, and a validation contract.

---

## Variant A — Multi-dimensional primitive scalar array function returns (recommended)

### Hypothesis

W545/W546/W547 handled 1-D primitive scalar arrays (`[3]u8`/`[3]i8`).  2-D and
higher arrays (`[2][3]u8`) require row-major linearization, wider packed
vectors, and correct bit-slice indexing across both dimensions.  Closing this
shape lets the feature handle small matrix helpers in the Icarus-lowerable
subset.

### Scientific grounding

- **CIRCT `hw.array_create` / `hw.array_get`.**  CIRCT lowers aggregate arrays
to flat bit-vectors with element extraction based on a linearized index; the
same row-major layout must be used for function parameters, returns, and local
storage.  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **Roofline / HLS memory layout.**  HLS tools flatten multi-dimensional arrays
into a single memory bank or vector; t27's packed-vector approach is the
register-file analogue.  [Vitis HLS array partitioning](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)

### Deliverables

1. Extend `bootstrap/src/compiler.rs` so that functions returning `[N][M]T`
   emit a packed vector of total width `N*M*W` and the caller's local/variable
   receives the full concatenation.
2. Extend `try_emit_primitive_array_access` to compute multi-dimensional
   bit-slices for packed locals.
3. Add scratch witnesses for 2-D primitive array return initializers and
   element reads (e.g. `let m : [2][3]u8 = grid(); assert_eq(m[1][2], 42)`).
4. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
5. Reseal affected specs and record Icarus baselines.

### Validation contract

- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 2-D witnesses.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.

---

## Variant B — Independent VCD cross-check for deterministic `bench` blocks

### Hypothesis

All W5xx cocotb cross-checks target `test` blocks.  `bench` blocks contain
latency/throughput assertions that are currently skipped by the reference model,
leaving no independent verification that generated Verilog preserves expected
cycle-level behavior for performance-sensitive specs.

### Scientific grounding

- **cocotb deterministic benchmarking.**  cocotb supports fixed-input test
sequences and cycle counters that can evaluate deterministic `bench` assertions
without relying on wall-clock timing.  [cocotb docs](https://docs.cocotb.org/)
- **RVFI-DII independent trace comparison.**  The RISC-V Formal Interface
separates reference-model and implementation traces so they can be compared
independently.  A lightweight t27 equivalent for `bench` blocks would give the
same separation of concerns for cycle-level behavior.  [RVFI](https://github.com/SymbioticEDA/riscv-formal)

### Deliverables

1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
deterministic assertions inside them (skipping non-deterministic or
timing-only benches).
2. Add `specs/scratch/w548_bench_scalar_call_cross_check.t27` as a positive
witness with a `bench` block that uses a lowerable function call.
3. Update `bootstrap/src/suite.rs` to run cocotb against `bench` blocks when
`--cocotb` is enabled.
4. Keep `test` and `bench` probes clearly distinguished in VCD output.

### Validation contract

- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
witness.
- Existing `test` cocotb count remains unchanged (no regression).
- `cargo test -p t27c --bin t27c` and `cargo test -p tri` stay green.

---

## Variant C — Signedness alignment for typed VCD probe registers

### Hypothesis

W538/W539 introduced typed scalar VCD probes.  The probe `reg` declarations are
currently always unsigned even when the probe metadata marks the expression as
signed.  The cocotb cross-check works because the reference model uses the
declared type to interpret the raw VCD bits, but the mismatch between the
actual Verilog declaration and the metadata is a latent correctness issue for
external VCD consumers and for `$display` probe logs.

### Scientific grounding

- **IEEE 1800-2017 signed net declarations.**  A `reg signed [N:0]` declaration
preserves signedness in procedural assignments and `$display`.  Aligning the
emitted declaration with the inferred signedness removes the latent mismatch.
[IEEE 1800-2017 §6.11](https://ieeexplore.ieee.org/document/8299595)
- **VCD value interpretation.**  VCD stores raw bit vectors; consumers rely on
the declared signal type (or an external manifest) to interpret signed values.
Emitting signed probe regs makes the generated Verilog self-describing.

### Deliverables

1. Extend `bootstrap/src/compiler.rs` so that scalar probe `reg` declarations
in `gen_verilog_test` include `signed` when the probe metadata says signed.
2. Add a scratch witness with a signed probe expression that also prints the
probe value, and verify that `$display` emits the signed decimal value.
3. Update `scripts/cocotb_ref_model.py` if any width/signedness fallback
heuristics can be simplified once the Verilog declaration matches the metadata.
4. Reseal affected specs.

### Validation contract

- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new signed-probe witness.
- Existing cocotb count remains unchanged.

---

## Recommendation

**Choose Variant A.**  It is the natural geometric continuation of
W545/W546/W547, has a clear compiler deliverable (linearized 2-D packed vectors
and multi-dimensional bit-slices), a matching formal-model update path, and
extends the primitive scalar array return feature to small matrix helpers that
are essential for DSP-style fixed-point kernels.

---

*φ² + φ⁻² = 3 | TRINITY*

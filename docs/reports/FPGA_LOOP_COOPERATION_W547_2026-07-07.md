# FPGA Loop Cooperation Variants — Wave Loop 547

**Source:** Wave Loop 546 closeout  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 546 closed the function-local primitive scalar array return gap.
Wave Loop 547 should now push outward to signed element types, outward to
multi-dimensional primitive arrays, or upward to deterministic `bench`-block
cross-checks.  Each variant below is scoped for a single loop, includes
scientific motivation, a deliverable list, and a validation contract.

---

## Variant A — Signed primitive scalar array function returns (recommended)

### Hypothesis

W545/W546 handled unsigned primitive arrays (`[3]u8`).  Signed arrays (`[3]i8`)
introduce sign-extension, signed comparison, and signed slicing subtleties that
are not yet verified.  Closing this boundary makes the feature usable for
DSP-style fixed-point helpers in the Icarus-lowerable subset.

### Scientific grounding

- **Verilog signed packed arrays.**  SystemVerilog supports signed packed vectors
  and part-selects, but sign extension and signed arithmetic must be explicit in
  generated Verilog (`$signed`, `signed` declarations) to match t27's two's
  complement semantics.  [IEEE 1800-2017 §11.5](https://ieeexplore.ieee.org/document/8299595)
- **SMT-LIB bit-vector signed operators.**  The t27 reference model and Lean
  semantics use signed comparison and extension operators (`bvsle`, `sext`).
  Mapping these to Verilog's signed operators keeps the model and generated code
  aligned.  [SMT-LIB QF_BV](http://smtlib.cs.uiowa.edu/logics-all.shtml)
- **CompCert signed integer semantics.**  CompCert's integer normalization and
  sign-extension proofs provide the reference pattern for preserving signed
  behavior across a bit-vector backend.  [CompCert memory model](https://compcert.org/publications.html)

### Deliverables

1. Extend `bootstrap/src/compiler.rs` to emit signed packed vectors for
   `[N]i8`/`[N]i16`/`[N]i32` returns and to use `$signed` / signed `reg` where
   needed for indexing and comparison.
2. Add positive scratch witnesses for signed array return initializers:
   `specs/scratch/w547_signed_call_init_returns_array.t27` and a signed
   comparison witness.
3. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
4. Reseal affected specs and record Icarus baselines.

### Validation contract

- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the positive signed witnesses.
- `cargo test -p t27c --test icarus_lowerable` stays green.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.

---

## Variant B — Multi-dimensional primitive scalar array function returns

### Hypothesis

W545/W546 handled 1-D primitive scalar arrays (`[3]u8`).  2-D and higher arrays
(`[2][3]u8`) require row-major linearization, wider packed vectors, and correct
bit-slice indexing across both dimensions.  Closing this shape lets the feature
handle small matrix helpers.

### Scientific grounding

- **CIRCT `hw.array_create` / `hw.array_get`.**  CIRCT lowers aggregate arrays to
  flat bit-vectors with element extraction based on a linearized index; the same
  row-major layout must be used for function parameters, returns, and local
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
   element reads.
4. Add lowerability/value-preservation theorems.

### Validation contract

- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 2-D witnesses.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.

---

## Variant C — Independent VCD cross-check for deterministic `bench` blocks

### Hypothesis

All W5xx cocotb cross-checks target `test` blocks.  `bench` blocks contain
latency/throughput assertions that are currently skipped by the reference model,
leaving no independent verification that generated Verilog preserves the
expected cycle-level behavior for performance-sensitive specs.

### Scientific grounding

- **cocotb deterministic benchmarking.**  cocotb supports fixed-input test
  sequences and cycle counters that can evaluate deterministic `bench`
  assertions without relying on wall-clock timing.  Extending the reference model
  to deterministic benches closes a verification hole.  [cocotb docs](https://docs.cocotb.org/)
- **RVFI-DII independent trace comparison.**  The RISC-V Formal Interface
  separates reference-model and implementation traces so they can be compared
  independently.  A lightweight t27 equivalent for `bench` blocks would give the
  same separation of concerns for cycle-level behavior.  [RVFI](https://github.com/SymbioticEDA/riscv-formal)

### Deliverables

1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
   deterministic assertions inside them (skipping non-deterministic or
   timing-only benches).
2. Add `specs/scratch/w547_bench_scalar_call_cross_check.t27` as a positive
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

## Recommendation

**Choose Variant A.**  It is the natural continuation of W545/W546, has a clear
compiler deliverable, a matching formal-model update path, and extends the
primitive scalar array return feature to signed element types that are essential
for DSP-style fixed-point helpers.  The scientific basis (IEEE 1800 signed
operators and SMT-LIB bit-vector signed semantics) is mature and directly
applicable.

---

*φ² + φ⁻² = 3 | TRINITY*

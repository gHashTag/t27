# FPGA Loop Cooperation Variants — Wave Loop 546

**Source:** Wave Loop 545 closeout  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 545 closed the primitive scalar array function-return gap at module
scope.  Wave Loop 546 should now push the same capability inward to function
locals, outward to signed/wider array shapes, or upward to deterministic
`bench`-block cross-checks.  Each variant below is scoped for a single loop,
includes scientific motivation, a deliverable list, and a validation contract.

---

## Variant A — Function-local primitive scalar array return initializers and reassignments (recommended)

### Hypothesis

Module-level `const`/`var` initializers from array-returning calls now work, but
the same pattern inside a function body (`let a : [3]u8 = seq();` or reassignment
`a = seq();`) is not yet exercised end-to-end.  Closing this shape completes the
primitive scalar array return matrix and lets small helper functions feed local
register arrays.

### Scientific grounding

- **CIRCT/FIRRTL register initialization.**  Function-local arrays map to
  combinational initializers on registers or wires.  CIRCT's `seq.compreg` and
  `hw.array_create` patterns show how a packed vector return can drive a local
  array register with the same width and layout.  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **CompCert local variable initialization.**  CompCert proves that local array
  initializers preserve memory equivalence; t27's shallow model can absorb a
  function-return local array initializer as a direct value binding, with the
  same proof obligation as a module-level global.  [CompCert memory model](https://compcert.org/publications.html)

### Deliverables

1. Extend `bootstrap/src/compiler.rs` so that a `StmtLocal` binding of a
   primitive scalar array from a function call emits the correct packed-vector
   `reg` and `initial` assignment inside the function scope.
2. Add a scratch witness `specs/scratch/w546_local_call_init_returns_array.t27`
   with a function-local `let` initialized from an array-returning call and
   assertions on the local.
3. Add a scratch witness `specs/scratch/w546_local_call_assign_returns_array.t27`
   that reassigns a local packed primitive array from a second call.
4. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
5. Reseal affected specs and record Icarus baselines.

### Validation contract

- `cargo test -p t27c --test icarus_lowerable` accepts the new witnesses.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  Icarus + cocotb PASS, 0 seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` green / 0 `sorry`.

---

## Variant B — Signed and wider primitive scalar array returns

### Hypothesis

W545 handled unsigned primitive arrays (`[3]u8`).  Signed arrays (`[3]i8`) and
arrays whose total packed width exceeds 64 bits introduce sign-extension,
comparison, and slicing subtleties that are not yet verified.  Closing this
boundary makes the feature usable for DSP-style fixed-point helpers.

### Scientific grounding

- **Verilog signed packed arrays.**  SystemVerilog supports signed packed vectors
  and part-selects, but sign extension and signed arithmetic must be explicit in
  generated Verilog (`$signed`, `signed` declarations) to match t27's two's
  complement semantics.  [IEEE 1800-2017 §11.5](https://ieeexplore.ieee.org/document/8299595)
- **SMT-LIB bit-vector signed operators.**  The t27 reference model and Lean
  semantics use signed comparison and extension operators (`bvsle`, `sext`).
  Mapping these to Verilog's signed operators keeps the model and generated code
  aligned.  [SMT-LIB QF_BV](http://smtlib.cs.uiowa.edu/logics-all.shtml)

### Deliverables

1. Extend `bootstrap/src/compiler.rs` to emit signed packed vectors for
   `[N]i8`/`[N]i16`/`[N]i32` returns and to use `$signed` / signed `reg` where
   needed for indexing and comparison.
2. Add positive scratch witnesses for signed array return initializers:
   `specs/scratch/w546_signed_call_init_returns_array.t27` and a signed
   comparison witness.
3. Add a negative or classifier-marked witness for arrays whose total packed
   width exceeds the current Verilog integer width supported by the backend.
4. Update `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` if the signed
   width boundary is formalized.
5. Reseal affected specs.

### Validation contract

- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the positive signed witnesses.
- `cargo test -p t27c --bin t27c` and `cargo test -p tri` stay green.
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
2. Add `specs/scratch/w546_bench_scalar_call_cross_check.t27` as a positive
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

**Choose Variant A.**  It is the natural continuation of W545, has a clear
compiler deliverable, a matching formal-model update path, and completes the
primitive scalar array return shape matrix (module-level → function-local).  The
scientific basis (CIRCT/FIRRTL register initialization and CompCert local memory
initialization) is mature and directly applicable.

---

*φ² + φ⁻² = 3 | TRINITY*

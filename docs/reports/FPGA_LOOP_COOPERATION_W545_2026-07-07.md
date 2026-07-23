# FPGA Loop Cooperation Variants — Wave Loop 545

**Source:** Wave Loop 544 closeout  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 544 closed the mutable-module-state / test-block call-assignment gap.
Wave Loop 545 should now attack one of three adjacent weak points.  Each
variant below is scoped for a single loop, includes scientific motivation, a
deliverable list, and a validation contract.

---

## Variant A — Primitive scalar array function returns (recommended)

### Hypothesis

Functions returning fixed-size primitive scalar arrays (e.g. `[3]u8`) are a
common pattern in t27 specs, but the Verilog backend currently cannot lower
such a return value into module `const`/`var` storage consistently.  Closing this
shape makes a large class of signal-processing and GF16-style array helpers
Icarus-synthesizable.

### Scientific grounding

- **CIRCT Handshake / FIRRTL memory port lowering.**  Returning small arrays
  from functions maps cleanly to a packed vector + simple port binding; the
  complexity is in connecting the callee's packed result to the caller's
  unpacked/packed declaration.  CIRCT's `hw.array_create` / `hw.array_get` provide
  the reference pattern.  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **Chisel `Vec` function-return idioms.**  Chisel functions returning `Vec` are
  elaborated as bundled wires; t27's packed-vector lowering already supports
  scalar-struct arrays, so primitive arrays are the natural next step.

### Deliverables

1. Extend the Verilog backend in `bootstrap/src/compiler.rs` so that a function
   returning `[N]T` (primitive scalar `T`) emits a packed vector result and the
   module-level `const`/`var` binding receives the full concatenation.
2. Remove the W544 classifier rule that rejects primitive scalar array function
   returns.
3. Update `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
   (`Function.isLowerable`) to accept these returns again.
4. Convert `specs/scratch/w544_negative_call_init_returns_array.t27` to a
   positive witness and add lowerability/sequential/value-preservation theorems
   in `Lemmas.lean` / `Soundness.lean`.
5. Reseal affected specs and record Icarus baselines.

### Validation contract

- `cargo test -p t27c --test icarus_lowerable` accepts the converted witness.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  Icarus + cocotb PASS, 0 seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` green / 0 `sorry`.

---

## Variant B — Formalize module-level mutable state for call initializers

### Hypothesis

W543/W544 proved empirically that mutable module vars with call initializers
work, but there is no Lean theorem connecting a module `var` initialized by a
function call to the sequential semantics.  Adding this theorem removes a
long-standing gap between the Rust backend and the formal model.

### Scientific grounding

- **Kami / Bluespec conservative updates.**  Bluespec-style rules separate
  combinational value calculation from sequential state update.  The t27
  `module_value_equiv_proved_sequential` framework can absorb a `varDecl`
  initialized by a call by treating the initializer as the initial value of the
  register and the call as a combinational expression.  [Kami paper](https://people.csail.mit.edu/joonwonc/files/kami.pdf)
- **CompCert memory initialization as theorem.**  CompCert proves that global
  variable initializers map to the right memory blocks; a similar proof for
  t27 module-level mutable vars strengthens the invariant between source and
  generated Verilog.

### Deliverables

1. Add a positive scratch witness `w545_module_var_call_init_soundness.t27`
   with a lowerable scalar call initializer.
2. Import it into `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`.
3. Prove `module_value_equiv_statement` for `Stmt.varDecl` with a call
   initializer in `Soundness.lean`.
4. No Rust backend change expected; classifier and backend already support the
   shape.

### Validation contract

- `lake build Trinity.IcarusLowerable.Soundness` green / 0 `sorry`.
- `./scripts/tri verify --lean-lowerable` (or equivalent) reports agreement
  between Rust classifier and Lean predicate for the new witness.
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays green.

---

## Variant C — Independent VCD cross-check for `bench` blocks

### Hypothesis

All W5xx cocotb cross-checks so far target `test` blocks.  `bench` blocks are
performance-oriented and currently skipped by the reference model, which means
there is no independent verification that generated Verilog preserves the
expected cycle-level behavior for latency/throughput assertions.

### Scientific grounding

- **Cocotb performance benchmarking patterns.**  cocotb supports wall-clock and
  cycle-count benchmarks through `Timer` and custom scoreboards.  Extending the
  reference model to evaluate deterministic `bench` assertions (fixed input
  sequences) would close a verification hole without requiring a full cycle
 -accurate simulator.  [cocotb docs](https://docs.cocotb.org/)
- **RVFI-DII style independent check.**  The RISC-V Formal Interface defines a
  standard trace format that lets a reference model and an implementation be
  compared independently; a lightweight t27 equivalent for `bench` blocks
  would give the same separation of concerns.  [RVFI](https://github.com/SymbioticEDA/riscv-formal)

### Deliverables

1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
   deterministic assertions inside them (skipping non-deterministic or
   timing-only benches).
2. Add `specs/scratch/w545_bench_scalar_call_cross_check.t27` as a positive
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

**Choose Variant A.**  It is a natural continuation of the W544 boundary work,
has a clear compiler deliverable, a matching classifier + formal-model update,
and converts an existing negative witness into a positive one.  The scientific
basis (packed-vector function returns and array port lowering) is mature and
well documented in CIRCT/Chisel literature.

---

*φ² + φ⁻² = 3 | TRINITY*

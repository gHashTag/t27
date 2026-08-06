# Wave Loop 552 Plan — Wide packed struct/array bench cross-check

Issue #1523 | branch `wave-loop-552` | next branch `wave-loop-553`

---

## Charter

Generalize the Icarus/cocotb reference-model cross-check from scalar `bench`
assertions to wide packed expressions: scalar structs and primitive scalar
arrays (1-D/2-D) returned by functions or stored in module-level variables.
Re-use the W540 multi-slice probe mechanism and the W550 row-major flat-index
Python evaluator. Keep the restriction to deterministic benches (no `#` delays,
no unbounded loops).

---

## Weak points discovered

1. **No wide bench witnesses.** W551 only verified a scalar function return
   inside a `bench`. The W540 multi-slice probe mechanism has not been exercised
   for bench blocks.
2. **Potential block-local type cache interaction.** `gen_verilog_probe_prelude`
   caches `StmtLocal` types; bench blocks may not declare locals the same way as
   tests in existing specs.
3. **Reference model keying untested for bench locals.** `_collect_assertions`
   stores block-local types under `block_name`, which is fine for benches too,
   but no bench has used locals yet.
4. **No Lean formalization for bench blocks.** `Trinity.IcarusLowerable` models
   `test` blocks but not `bench` blocks. For W552 the runtime cross-check is the
   focus; formalization is optional but the boundary should be documented.
5. **VCD wide-value reconstruction in benches untested.** The Python
   `_eval_array_lit_bv` / `_eval_struct_lit_bv` paths are shared, but the
   end-to-end bench path has not been exercised.

---

## Engineering / scientific background

* Cocotb reference-model cross-check is a standard translation-validation pattern
  for generated Verilog testbenches. Extending from `test` to deterministic
  `bench` is a direct generalization.
* Packed vector slicing (W540): wide values are split into 64-bit (or final
  partial) slices; the reference model reconstructs the full value by
  concatenating slices at declared bit offsets.
* Row-major flat indexing (W548–W550): multi-dimensional primitive arrays are
  linearized in row-major order; the Python evaluator already implements this.
* Deterministic benches: continue to restrict cross-check to benches without
  timing or unbounded loops, avoiding a clocked sampling model.

---

## Implementation tasks

### A. Create W552 scratch witnesses
Three specs under `specs/scratch/`:
* `w552_bench_wide_packed_struct.t27` — bench asserts equality on a packed
  scalar struct returned from a function (reuse W540 shape).
* `w552_bench_module_wide_struct.t27` — module-level var receives whole-struct
  assignment inside a bench, then asserted.
* `w552_bench_2d_array_return.t27` — bench asserts on a 2-D primitive scalar
  array return value or element sum.

Each witness contains both a `test` block and a deterministic `bench` block so
both markers appear in the baseline and the cocotb gate counts both.

### B. Reference model verification
* Confirm `_collect_assertions` handles bench-block local declarations and
  `StmtAssign` updates to module-level mutable vars the same way as test blocks.
* No expected changes to `_eval_struct_lit_bv` / `_eval_array_lit_bv` because the
  evaluator is already shared.

### C. Compiler verification
* No compiler changes expected because `gen_verilog_probe_prelude` is shared and
  handles scalar/wide probes generically. If a bug appears, fix it.
* Verify wide bench assertions emit the same `_t27_probe_*` slice registers as
  wide test assertions.

### D. Baselines, seals, and integration tests
* Generate Icarus baselines for each new `w552_*` witness via the suite gate.
* Save t27 seals for each new spec.
* Add `accepts_w552_bench_wide_cross_check` in
  `bootstrap/tests/icarus_lowerable.rs` covering the three new witnesses.

### E. Validation matrix
* `cargo build --release -p t27c`
* `cargo test -p t27c --bin t27c`
* `cargo test -p tri`
* `cargo test -p t27c --test icarus_lowerable`
* `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
* `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 553

### Variant A — Recommended: signed/unsigned mixed bench probes
Verify that `$signed(...)` wrappers and VCD value reconstruction work correctly
inside deterministic `bench` blocks for signed scalar/array returns.

### Variant B: bench-local primitive scalar arrays
Allow `let tmp : [N]T = f();` inside a `bench` block where `f` returns a packed
primitive scalar array, and cross-check element reads against the reference
model.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and document
 the boundary.

---

## Skills to save at closeout

Pattern: *"Wide packed values in deterministic bench blocks share the same
multi-slice probe emission and Python VCD reconstruction as test blocks; adding
bench witnesses is usually enough."*

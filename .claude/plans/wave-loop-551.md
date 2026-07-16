# Wave Loop 551 Plan — Independent VCD cross-check for deterministic `bench` blocks

Issue #1522 | branch `wave-loop-551` | next branch `wave-loop-552`

---

## Charter

Extend the Icarus/cocotb reference-model cross-check from `test` blocks to
deterministic `bench` blocks. A deterministic bench is one that contains only
pure function calls, scalar assertions, local declarations, and assignments —
no `#` delays, no unbounded loops, no external I/O. Such benches already run in
Verilog simulation as `initial begin ... end` blocks; this loop gives them an
independent Python/VCD correctness check identical to the one W538–W550
built for `test` blocks.

---

## Weak points discovered

1. **Probe hoisting is test-only.** `gen_verilog_test` (compiler.rs:5971) resets
   `probe_counter`, populates `probe_specs`, and hoists scalar/wide probe
   `reg`s plus test-block `StmtLocal` declarations. The `bench` emission path
   (compiler.rs:5200) calls `gen_verilog_test_stmt` directly without any of that
   setup, so `assert_eq` inside a `bench` block has no backing probe register and
   `self.probe_counter` carries over from previous blocks.
2. **Reference model ignores benches.** `scripts/cocotb_ref_model.py:_collect_assertions`
   explicitly skips anything that is not `TestBlock`/`InvariantBlock`, so
   deterministic assertions in `bench` blocks are never evaluated.
3. **Status-marker mismatch.** A failing assertion inside a bench block currently
   emits `[TEST] <bench_name> : FAILED` because `gen_verilog_test_stmt`
   hard-codes the `[TEST]` tag and uses the block name. The bench wrapper only
   prints `[BENCH] <name> : DONE`. The cocotb/suite log parser therefore has no
   reliable bench-level pass/fail marker.
4. **Suite cocotb gate is test-centric.** `bootstrap/src/suite.rs` runs cocotb
   only on a W5xx/W3xx scratch whitelist and expects `[TEST]`-style baselines.
   Bench blocks are not separately reported.
5. **No deterministic-bench witness.** There is no `specs/scratch/w551_*` file
   exercising `bench { assert_eq(...) }`.

---

## Engineering / scientific background

* IEEE 1364 / Verilog-2005 permits multiple `initial` blocks that execute at time
  0. Emitting each `bench` as its own `initial begin ... end` block is already
  the t27 strategy.
* Cocotb + Icarus reference-model pattern: evaluate expected values from the
  AST and compare against values sampled from VCD probes. Extending this from
  `test` to deterministic `bench` blocks is a direct generalization; no new
  formal theory is required.
* Deterministic vs. non-deterministic benches: for this loop we restrict the
  cross-check to deterministic benches (no delays, no unbounded loops). This
  matches the existing Icarus-lowerable subset and avoids needing a clocked
  sampling model.
* Regression hygiene: the baseline format already filters out `[PROBE]` and
  VCD diagnostics (W538), so adding `[BENCH]` status lines is safe and will not
  destabilize existing seals.

---

## Implementation tasks

### A. Compiler: refactor probe hoisting
* Extract the probe/local hoisting logic from `gen_verilog_test` into a new
  private helper `gen_verilog_probe_prelude(&mut self, node: &Node, block_name: &str)`.
  * Reset `probe_counter = 0`.
  * Clear/populate `probe_specs` from the block's `assert_eq` statements.
  * Emit `StmtLocal` declarations (Decl phase).
  * Emit scalar/wide probe `reg` declarations.
* Call this helper from both `gen_verilog_test` and the bench emission loop.
* Keep `$dumpfile`/`$dumpvars` inside the test-block section only.

### B. Compiler: uniform status markers
* Add a `block_tag` parameter to `gen_verilog_test_stmt` (or derive it) so bench
  assertions print `[BENCH] ... FAILED` instead of `[TEST] ... FAILED`.
* Treat the existing `[BENCH] ... DONE` line as the bench equivalent of
  `[TEST] ... PASSED` when no assertion failed.

### C. Reference model: collect bench assertions
* In `scripts/cocotb_ref_model.py:_collect_assertions`, include `"BenchBlock"`
  alongside `"TestBlock"`/`"InvariantBlock"`.
* Record `block_kind` in the assertion tuple so the runner can choose the
  expected log marker.
* Bind block-local `StmtLocal` declarations and `StmtAssign` updates for benches
  the same way as for tests.

### D. Suite: parse bench results
* Extend `bootstrap/src/suite.rs` Icarus log normalization / pass/fail
  extraction to recognize both `[TEST] ... : PASSED/FAILED` and
  `[BENCH] ... : DONE/FAILED`.
* Include bench blocks in the cocotb gate report counts.

### E. Witness + test + seal
* Create `specs/scratch/w551_bench_scalar_call_cross_check.t27`:
  * A pure function returning a scalar value.
  * A `bench "cross_check" { assert_eq(f(...), expected); }` block.
* Add Rust integration test `accepts_w551_bench_block_cross_check` in
  `bootstrap/tests/icarus_lowerable.rs` (or extend the existing cocotb gate).
* Generate baseline under `.trinity/icarus-baselines/specs/scratch/w551_bench_scalar_call_cross_check.json`.
* Reseal the new spec and any affected corpus specs.

### F. Validation matrix
* `cargo build --release -p t27c`
* `cargo test -p t27c --bin t27c`
* `cargo test -p tri`
* `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
* `lake build Trinity.IcarusLowerable.Soundness` (if any Lean files touched)

---

## Three cooperation variants for Wave Loop 552

### Variant A — Recommended: wide struct/array benches
Generalize W551 to deterministic `bench` blocks whose `assert_eq` actual
expressions are packed scalar structs or primitive scalar arrays (1-D/2-D).
Re-use the W540 slice-probe mechanism; only requires widening the Python
reference model's `_type_of_expr`/`_eval_expr_bv` coverage for bench locals. Low
risk, high consistency.

### Variant B — signedness coverage for bench probes
Add explicit signed/unsigned mixed `bench` witnesses and ensure `$signed(...)`
wrappers and VCD value reconstruction work inside bench blocks. Narrow but
important edge-case follow-up.

### Variant C — timed/non-deterministic bench classifier
Introduce a classifier that rejects (or skips) `bench` blocks containing `#`
delays or unbounded loops from the deterministic cocotb gate, documenting the
boundary. More defensive policy work than feature work.

---

## Skills to save at closeout

Pattern: *"Deterministic bench blocks can share the same AST traversal, probe
hoisting, and reference-model evaluator as test blocks; only the status marker
and block-kind filter need to differ."*

# Wave Loop 557 Plan — General bench CSE for scalar calls

Issue #1528 | branch `wave-loop-557` | next branch `wave-loop-558`

---

## Charter

Implement **Variant A**: extend the W556 block-scoped temporary
recommendation to pure scalar-return function calls inside deterministic
`test` and `bench` blocks. The witness asserts both `assert_eq(f(), expected)`
and `assert_eq(f() + g(), ...)` in the same bench; the generated Verilog
should evaluate `f()` exactly once.

---

## Weak points discovered

1. **Scalar-return calls are invisible to the dedup machinery.**
   `call_returning_packed_primitive_array_info` requires a primitive scalar
   array return type, so `u32`, `i8`, etc. are ignored.
2. **Substitution only happens at the top level of an assert_eq actual
   expression.** `gen_verilog_expr_with_call_array_tmp` only replaces a bare
   `ExprCall`. A scalar call inside a binary expression is still emitted as a
   raw call and re-evaluated.
3. **Expected-value side is not deduplicated.** If the expected expression
   also calls the same pure function, it will not share the temporary.
4. **No contextual switch to prefer call temporaries.** Adding substitution
   directly into `gen_verilog_expr` is risky because `collect_expr_text` is
   used both to build the dedup key and to render the materialization RHS.
5. **Existing scalar-call witnesses will change output.** `w551_bench_scalar_call_cross_check`
   and `w553_bench_signed_scalar_return` will now emit `_t27_call_tmp_*`
   registers, so any exact baselines may need regeneration.

---

## Engineering / scientific background

- **Common subexpression elimination in HDL compilers.** Verilator implements
  fine-grained CSE in `V3DfgCse.cpp` by hashing DFG vertices and merging
  structurally equivalent ones. T27's block-scoped textual-key approach is
  simpler but targets the same redundancy: repeated pure function calls inside
  simulation-only assertion harnesses.
- **Temporary variable creation.** Verilator's `V3Premit.cpp` inserts temporaries
  for wide and deep expressions before emission. T27 already does this for
  array-return calls (W553) and whole-array probes (W540); W557 generalizes it
  to scalar-return calls.
- **SVA local-variable pattern.** SystemVerilog assertions rely on per-attempt
  local variables to avoid shared state when a property is reused. T27's
  per-block temporary is a coarser, deterministic equivalent: one value per
  block, evaluated once, referenced from every site.

Sources:
- [Verilator V3DfgCse.cpp](https://github.com/verilator/verilator/blob/dbd48233/src/V3DfgCse.cpp)
- [Verilator V3Premit.cpp](https://github.com/verilator/verilator/blob/03ed6a5b/src/V3Premit.cpp)
- [ASPLOS 2024 — Don't Repeat Yourself! Coarse-Grained Circuit Deduplication](https://doi.org/10.1145/3622781.3674184)
- [Verification Academy — Assertion based query / shared resources](https://verificationacademy.com/forums/t/assertion-based-query/36957)
- [SystemVerilog Assertions Handbook local variables](https://systemverilog.us/vf/seq_local_var.pdf)

---

## Implementation tasks

### A. Create W557 scratch witnesses

- `specs/scratch/w557_bench_scalar_call_dedup.t27`:
  - `pub fn val() -> u32` and `pub fn other() -> u32`.
  - `bench` block contains:
    - `assert_eq(val(), 0xAB);`
    - `assert_eq(val() + other(), 0xAB + 0xCD);`
  - Equivalent `test` block.
- `specs/scratch/w557_bench_signed_scalar_call_dedup.t27` (optional):
  - `pub fn sval() -> i8` returning a negative value.
  - Same multi-site pattern to exercise signed temporary declaration.

### B. Generalize the call-temporary machinery

In `bootstrap/src/compiler.rs`:

1. Rename / generalize `call_returning_packed_primitive_array_info` to
   `call_returning_cse_value_info` and return a temporary descriptor for:
   - primitive scalar array returns (existing behavior)
   - primitive scalar returns (`u8`, `i8`, `u16`, `i16`, `u32`, `i32`,
     `u64`, `i64`, `bool`)
   Keep the same key = full call expression text.
2. Keep `call_array_tmp_info` value shape `(Vec<usize>, String, u32, bool)` but
   let the first field be empty for scalars (dims empty) and the second field
   hold the scalar type name.
3. Generalize `predeclare_call_array_tmps` and `materialize_call_array_tmp` to
   use the new helper.
4. Add a `use_call_array_temps: bool` field to `VerilogCodegen`.
   - Initialize to `false`.
   - Set to `true` around the statement loop in `gen_verilog_test` and
     `gen_verilog_bench`.
   - Reset to `false` after each block.
5. Modify `gen_verilog_expr` `ExprCall` arm:
   - If `use_call_array_temps` is true and the call text is in
     `call_array_tmp_names`, emit the temporary name.
   - Otherwise emit the raw call.
6. Ensure `collect_expr_text` initializes its temporary codegen with
   `use_call_array_temps: false`, so dedup keys and materialization RHS stay
   based on the original call text.
7. Update the temporary declaration loop in `gen_verilog_probe_prelude` to emit
   a more generic comment (e.g. "packed call tmp" instead of "packed
   call-return array tmp").
8. Remove or simplify `gen_verilog_expr_with_call_array_tmp`; `gen_verilog_expr`
   is now context-aware.

### C. Validate generated Verilog

For each witness, inspect that:
- Only one `_t27_call_*_tmp` temporary is declared per unique call.
- Only one assignment `<tmp> = val();` appears per block.
- `assert_eq(val(), ...)` and `assert_eq(val() + other(), ...)` both reference
  the same temporary.

### D. Baselines, seals, integration test

- Run direct `t27c icarus-simulate` / `t27c icarus-cocotb` on the witnesses.
- Save t27 seals.
- Record Icarus baselines.
- Add `accepts_w557_bench_scalar_call_dedup` integration test in
  `bootstrap/tests/icarus_lowerable.rs`.

### E. Validation matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- Direct `./target/release/t27c icarus-simulate specs/scratch/w557_*.t27`
- Direct `./target/release/t27c icarus-cocotb specs/scratch/w557_*.t27`
- `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 558

### Variant A — Recommended: deduplicate scalar calls in expected expressions too
Currently W557 will deduplicate calls on the actual side of `assert_eq`. Extend
materialization and substitution to the expected expression as well, so
`assert_eq(val(), val() + 1)` shares a single temporary for both `val()` calls.

### Variant B: signed whole-array comparison for higher ranks
Extend W555 whole-array probes to 3-D and 4-D signed primitive scalar arrays,
verifying row-major slice reconstruction in the Python model for ranks 3 and 4.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and update
`docs/ICARUS_LOWERABLE_BOUNDARY.md` accordingly.

---

## Skills to save at closeout

Pattern: *"A block-scoped common-subexpression pass for pure function calls
needs a contextual switch: set it while emitting a test/bench block, keep the
general expression emitter clean, and force the key-generation / RHS path to
always render the original call text so temporary names never leak into their
own definitions."*

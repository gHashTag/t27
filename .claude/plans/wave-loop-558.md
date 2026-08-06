# Wave Loop 558 Plan — Expected-side scalar call deduplication

Issue #1529 | branch `wave-loop-558` | next branch `wave-loop-559`

---

## Charter

Implement **Variant A** from W557: extend the block-scoped call CSE machinery to
pure scalar-return function calls that appear on the **expected side** of
`assert_eq` as well as the actual side. The acceptance witness is a deterministic
`bench` block containing `assert_eq(val(), val())` (or `assert_eq(val(), val() + 1)`)
where the same pure scalar call appears on both sides; the generated Verilog
must evaluate the call exactly once and share the temporary between both sides.

---

## Weak points discovered

1. **Unclear whether W557 already covers the expected side.**
   `predeclare_call_array_tmps` recurses into every child of every statement,
   including the expected-expression child of `assert_eq`. `gen_verilog_test_stmt`
   enables `use_call_array_temps` for the whole block, and `gen_verilog_expr`
   substitutes temporaries for any matching `ExprCall`. Therefore the expected
   side is already covered by the existing implementation.
2. **No witness explicitly exercises the expected side.**
   W557's witness only has a literal expected value. A new W558 witness is needed
   to lock the behavior and serve as a regression test.
3. **Existing boundary documentation describes only array-return temporaries.**
   Section 10 of `docs/ICARUS_LOWERABLE_BOUNDARY.md` was written for W556
   array-return deduplication and does not mention scalar-return calls or the
   expected side.

---

## Engineering / scientific background

- **Common subexpression elimination across assertion operands.**
  Compilers and verifiers that perform GCSE treat both operands of a comparison
  or equality assertion as part of the same available-expression scope. VO-GCSE
  (FSE 2025) applies this idea to SMT-based bounded model checking by hoisting
  repeated sub-expressions out of safety assertions, reducing the number of
  verification conditions the solver must process. In a hardware-simulation
  context, the same principle lets a testbench evaluate a pure reference call
  once and use its value for both the actual and expected operands.
- **Pure-call memoization in reference models.**
  CREST (arXiv 2019) compiles high-level ANSI-C reference models into Verilog
  for RTL equivalence checking. CBMC's simplifier performs compiler-style CSE on
  the C reference; the resulting Verilog assertions inherit the redundancy
  removal. T27's block-scoped call temporary is a lighter, deterministic
  equivalent that does not require solving.
- **SVA shared local-variable pattern.**
  SystemVerilog assertions use per-attempt local variables to avoid re-evaluating
  an expression when it appears in multiple places of the same property. T27's
  per-block temporary is a coarser, simulation-only version of that pattern.

Sources:
- [VO-GCSE: Verification Optimization through Global Common Subexpression Elimination](https://ssvlab.github.io/lucasccordeiro/papers/fse2025.pdf)
- [CREST: Hardware Formal Verification with ANSI-C Reference Specifications](https://arxiv.org/pdf/1908.01324)
- [CompCert verified CSE](https://github.com/AbsInt/CompCert/blob/master/backend/CSEproof.v)
- [SystemVerilog Assertions Handbook local variables](https://systemverilog.us/vf/seq_local_var.pdf)

---

## Implementation tasks

### A. Create W558 scratch witness

- `specs/scratch/w558_bench_scalar_call_expected_side_dedup.t27`:
  - `pub fn val() -> u32` returning `0xAB`.
  - `pub fn other() -> u32` returning `0xCD`.
  - `test` block:
    - `assert_eq(val(), val());`
    - `assert_eq(val() + other(), val() + other());`
  - `bench "scalar_call_expected_side_dedup_bench"` with the same assertions.

The witness proves that the same pure scalar call is evaluated once and used on
both the actual and expected sides of the equality.

### B. Verify compiler already deduplicates expected-side calls

No compiler change is required. Inspect generated Verilog for the witness:
- Only one `_t27_call_tmp_*` temporary is declared per unique call.
- Only one assignment `<tmp> = val();` appears per block.
- Both sides of `assert_eq` reference the same temporary for `val()`.

### C. Update Icarus lowerable boundary documentation

In `docs/ICARUS_LOWERABLE_BOUNDARY.md`:
- Rename section 10 title to mention scalar-return calls.
- Add a sentence stating that the same deduplication applies to both operands of
  `assert_eq` and to scalar-return calls (W557/W558).
- Add W558 witness to the section.

### D. Save seals and baselines

- Run `t27c seal` for the new scratch spec.
- Save Icarus baseline if needed (likely none, because no compiler change).

### E. Add integration test

Add `accepts_w558_bench_scalar_call_expected_side_dedup` in
`bootstrap/tests/icarus_lowerable.rs`.

### F. Validation matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- Direct `./target/release/t27c icarus-simulate specs/scratch/w558_*.t27`
- Direct `./target/release/t27c icarus-cocotb specs/scratch/w558_*.t27`
- `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 559

### Variant A — Recommended: signed whole-array comparison for higher ranks
Extend W555 whole-array probes to 3-D and 4-D signed primitive scalar arrays,
verifying row-major slice reconstruction in the Python reference model for ranks
3 and 4.

### Variant B: scalar-struct return call deduplication
Apply the block-scoped call temporary machinery to lowerable packed scalar-struct
return calls used at multiple sites in a `test` or `bench` block. The temporary
would be a packed-vector register whose width equals the struct element width.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and update
`docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556/W557 deduplication
optimization is only valid for pure calls.

---

## Skills to save at closeout

Pattern: *"When a wave turns out to be a witness-only regression lock because the
previous generalization already solved the problem, still produce the plan,
witness, integration test, and documentation updates so the behavior is recorded
and future regressions are caught."*

# Wave Loop 560 Plan — Scalar-struct return call deduplication

Issue #1531 | branch `wave-loop-560` | next branch `wave-loop-561`

---

## Charter

Implement **Variant A** from Wave Loop 559: extend the W556–W558 block-scoped
function-call CSE to **lowerable packed scalar-struct returns**. When a call
such as `make(a, b)` returning a scalar struct `Pt` is used at more than one
site inside a deterministic `test` or `bench` block, the backend should assign
the packed-vector result once to a block-local temporary and reference it from
every use site, instead of re-invoking the function each time.

---

## Weak points discovered

1. **Call CSE currently ignores scalar-struct returns.**
   `call_returning_cse_value_info` in `bootstrap/src/compiler.rs` only recognizes
   primitive scalars and primitive scalar arrays. A call returning `Pt` gets no
   temporary, so `assert_eq(make(1, 2), Pt{...})` and `make(1, 2).x` are lowered
   as independent function invocations.
2. **Field access on a call result still emits raw call text.**
   The W533 path that lowers `make().x` to a packed-vector part-select uses
   `collect_expr_text` on the call child. Even after a temporary is registered,
   that path does not substitute the pre-declared temporary, so CSE is bypassed
   for field-access uses.
3. **No regression locks whole-struct comparison for scalar-struct calls.**
   W533 exercised field access and module assignment, and W555 exercised whole
   primitive-array comparison, but there is no witness that asserts equality
   on an entire scalar-struct value returned by a call and reuses that call at
   field-access sites.
4. **Deduplication scope is invisible to the reference model.**
   The cocotb Python model evaluates the expected literal and reads the VCD
   probe for the actual expression; it does not check whether the generated
   Verilog duplicated the call. Verification therefore relies on inspecting the
   emitted RTL and ensuring only one `_t27_call_tmp_*` assignment exists per
   unique call text.
5. **Side-effect purity assumption is implicit.**
   The W557/W558 CSE is only sound because the Icarus-lowerable subset currently
   excludes host-only helpers, unresolved imports, and time-controlled
   statements. A future Variant C should make this explicit, but for now the
   optimization remains safe by construction.

---

## Engineering / scientific background

- **Common subexpression elimination in hardware compilers.** CSE is a classic
  compiler optimization (Aho, Sethi, Ullman, *Compilers: Principles,
  Techniques, and Tools*, §9.2). Modern hardware IRs apply it as a standard pass:
  CIRCT/firtool runs `createCSEPass()` on the `comb` dialect before Verilog
  emission to remove redundant operations introduced by MUX relocation and
  other domain-specific transformations (CombRewriter, ASPDAC 2026).
- **SystemVerilog function semantics.** IEEE 1800-2017 treats functions as
  zero-time combinational blocks that return a value through an implicit
  variable or `return` expression. Synthesis tools inline or flatten functions,
  so a function call in RTL becomes a combinational logic cloud; replicating a
  call therefore duplicates logic. The t27 backend already lowers scalar-struct
  returns as packed vectors, making a single temporary register the natural CSE
  representation.
- **Return-value optimization and packed-vector temporaries.** C++-style RVO
  is not defined by SystemVerilog, but the same effect is achieved by assigning
  the function result to a packed `reg` and reusing it. The DVCon paper
  *Can My Synthesis Compiler Do That?* confirms that major synthesis tools
  support functions returning packed vectors and structures.
- **Reference-model cross-check for CSE.** Because the cocotb reference model
  compares the independently-evaluated expected expression against the VCD-
  captured actual value, the optimization must preserve observable values. CSE is
  semantics-preserving for pure expressions, which holds for the Icarus-
  lowerable subset.

Sources:
- [Aho/Sethi/Ullman, Compilers: Principles, Techniques, and Tools](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)
- [CombRewriter: Enabling Combinational Logic Simplification in MLIR-Based Hardware Compiler](https://www.cse.cuhk.edu.hk/~byu/papers/C305-ASPDAC2026-CombRewriter.pdf)
- [CIRCT / firtool Verilog Generation pipeline](https://circt.llvm.org/docs/VerilogGeneration/)
- [IEEE 1800-2017 SystemVerilog Language Standard](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- [Can My Synthesis Compiler Do That? (DVCon)](https://dvcon-proceedings.org/wp-content/uploads/can-my-synthesis-compiler-do-that.pdf)

---

## Implementation tasks

### A. Extend `call_returning_cse_value_info` to scalar-struct returns

In `bootstrap/src/compiler.rs`, add a branch that returns a temporary descriptor
when `self.fn_return_types.get(&node.name)` is a lowerable packed scalar struct:

- `dims` = empty vector.
- `elem_type` = base struct name.
- `width` = `self.packed_width(ret_ty)`.
- `signed` = `self.packed_signed(ret_ty)` (false for scalar structs).

This lets `predeclare_call_array_tmps` and `materialize_call_array_tmp` treat
scalar-struct calls the same way they treat scalar and primitive-array calls.

### B. Substitute the temporary in field-access-on-call emission

In `gen_verilog_expr` for `NodeKind::ExprFieldAccess`, when the child is an
`ExprCall` and `use_call_array_temps` is enabled, check whether the rendered call
text matches a key in `call_array_tmp_names`. If so, use the temporary name as
the base of the packed-vector part-select instead of the raw call text.

### C. Create W560 scratch witnesses

- `specs/scratch/w560_bench_scalar_struct_call_dedup.t27`:
  - `struct Pt { x: i16, y: i16 }`.
  - `pub fn make(a: i16, b: i16) -> Pt`.
  - `test` and `bench` blocks that use `make(1, 2)` at multiple sites:
    - whole-struct `assert_eq(make(1, 2), Pt{...})`,
    - field access `assert_eq(make(1, 2).x, 1)`,
    - local init `var tmp : Pt = make(1, 2); assert_eq(tmp.y, 2)`.
- `specs/scratch/w560_bench_scalar_struct_call_dedup_both_sides.t27` (optional):
  - `assert_eq(make(1, 2), make(1, 2))` to lock expected-side reuse.
- `specs/scratch/w560_bench_scalar_struct_call_dedup_nested.t27` (optional):
  - expression `make(1, 2).x + make(1, 2).y` to exercise two field-access
    sites sharing one temporary.

### D. Save seals and baselines

- Save t27 seals for every witness.
- Record Icarus baselines for witnesses that pass the automated gate; the
  generated Verilog must contain exactly one `_t27_call_tmp_*` assignment per
  unique call text.

### E. Add integration test

Add `accepts_w560_bench_scalar_struct_call_dedup` in
`bootstrap/tests/icarus_lowerable.rs` covering the W560 witnesses.

### F. Validation matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- Direct `t27c icarus-simulate` / `t27c icarus-cocotb` on each W560 witness
- Inspect generated Verilog to confirm a single temporary per unique call
- `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 561

### Variant A — Recommended: array-typed scalar-struct field whole-array comparison
Extend the W555 whole-array probe to scalar-struct variables whose fields are
fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` where `tmp` is a
scalar struct with array-typed fields. Reuse the same packed-vector probe and
Python reconstruction paths, with field-by-field concatenation in the expected
literal.

### Variant B: scalar-struct return call deduplication for nested calls
Generalize the W560 temporary substitution so that calls returning scalar
structs used as arguments to other calls (e.g. `sum(make(1, 2))`) are also
deduplicated when the same call text appears at multiple argument sites.

### Variant C: explicit pure-call classifier for the CSE gate
Introduce an AST classifier that rejects (or skips) `bench` / `test` blocks
containing side-effecting constructs (`#` delays, unbounded loops, host-only
helpers, unresolved imports) from the deterministic call-CSE optimization, and
update `docs/ICARUS_LOWERABLE_BOUNDARY.md` to document the purity precondition.

---

## Skills to save at closeout

Pattern: *"When extending block-scoped call CSE to a new return-shape, the
existing predeclaration / materialization / substitution machinery usually
needs only one extension point (the shape classifier) and one emission-site
update (field / index access on the call result). The rest of the pipeline is
shape-agnostic."*

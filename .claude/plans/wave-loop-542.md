# Wave Loop 542 Plan — Scalar function-call arguments for independent VCD cross-check

**Issue:** #1513 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-542`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W542_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

After Wave Loop 541, the cocotb reference model covers literals, identifiers,
parameterless calls, field access, scalar indexing, binary/unary ops, casts,
switch/ternary, struct/array literals, and module-level packed values.  The largest
remaining runtime gap is **scalar function-call arguments**.

Examples that still skip the independent VCD check:
- `assert_eq(read_signed(0), -100)` — the actual expression is a call with a scalar
  argument.
- `assert_eq(sum(Pt{...}), 6)` — the actual expression is a call with a struct
  literal argument (the call itself is scalar, but its evaluation currently returns
  `None` because `_eval_call_bv` already accepts arguments but the evaluator chain
  may fail on struct-literal argument sizing).
- `assert_eq(write_read_signed(1, -42), -42)` — multiple scalar arguments with mixed
  signedness.

`_eval_call_bv` already has an argument-evaluation loop, but it does not:
- Re-wrap untyped literal arguments at the declared parameter width and signedness.
- Re-wrap struct-literal arguments at the parameter width when the evaluator default
  is narrower.
- Return a typed result for scalar-return calls (it returns a raw `Bv`, which is
  fine, but the caller `_type_of_expr` only uses `fn_return_types` and may miss packed
  return types).

Because the actual expression is a call, `expr_width_signed` in the compiler already
resolves scalar primitive return types, so the VCD probe width is correct.  The
problem is purely in the Python reference model's ability to evaluate the call.

---

## 2. Literature and related work

- **CIRCT `sim.func.dpi` — Simulation Dialect DPI.**
  CIRCT models scalar function-call argument binding in hardware IR with explicit
  `in`/`out`/`return` port directions and a lowering pass that maps SSA-style calls
  to C pointer-based DPI.  This confirms that argument binding and width preservation
  are the core correctness concern when connecting a high-level function model to a
  hardware simulation.
  [Simulation Dialect DPI - CIRCT](https://circt.llvm.org/docs/Dialects/SimDPI/)

- **K-CIRCT: A Layered, Composable, and Executable Formal Semantics for CIRCT
  Hardware IRs (2024).**
  Provides a reference model and co-simulation against Verilator/VCD, validating that
  a formal executable semantics can serve as a golden oracle for generated hardware.
  Relevant to the broader strategy of using a Python reference model plus VCD probes as
  an equivalence oracle.
  [arXiv](https://arxiv.org/html/2404.18756v1)

- **EPEX — Processor Verification by Equivalent Program Execution (JKU, 2021).**
  Uses a formal ISA reference model and SMT-driven equivalent program execution to
  compare architectural states.  Reinforces the value of a trusted golden model for
  hardware equivalence checking.
  [PDF](https://ics.jku.at/files/2021GLSVLSI_EPEX.pdf)

- **Interaction Tree Semantics for RISC-V — Bridging Compiler and Hardware
  Verification (2026).**
  Machine-checked RISC-V semantics as a shared reference model for compiler and
  hardware verification, proving per-instruction correctness and extracting an
  executable simulator.  Shows how scalar instruction semantics propagate across
  compiler/hardware boundaries.
  [arXiv](https://arxiv.org/html/2605.04933v1)

- **Keq — Language-Parametric Compiler Validation with Application to LLVM.**
  Uses cut-bisimulation to relate live value/register bindings across function calls in
  different IRs.  Directly relevant to correctly binding arguments at function-call
  boundaries during equivalence checking.
  [PDF](https://zhengyao.page/papers/keq.pdf)

For t27, the Python evaluator is the golden model.  W542 fixes argument binding so
that scalar calls are fully evaluated and comparable against the VCD probe.

---

## 3. Decomposed plan

### Phase 1 — Issue

`.trinity/current-issue.md` already defines W542 and Variant A.  Verify scope and
ensure issue #1513 is referenced in all commits.

### Phase 2 — Spec / TDD

Write three scratch specs that exercise scalar function-call arguments:

1. `specs/scratch/w542_scalar_call_args.t27`
   ```t27
   pub fn add(a: u32, b: u32) -> u32 { return a + b; }
   test scalar_call_args { assert_eq(add(3, 4), 7); }
   ```

2. `specs/scratch/w542_signed_scalar_call.t27`
   ```t27
   pub fn sub(a: i16, b: i16) -> i16 { return a - b; }
   test signed_scalar_call { assert_eq(sub(-3, 4), -7); }
   ```

3. `specs/scratch/w542_struct_sum_call.t27`
   ```t27
   struct Pt { x: i16, y: i16, z: u16 }
   pub fn sum(p: Pt) -> u32 { return (p.x as u32) + (p.y as u32) + (p.z as u32); }
   test struct_sum_call { assert_eq(sum(Pt{...}), 6); }
   ```

4. Optionally add `w542_mixed_args_call.t27` combining scalar and struct-literal
   arguments if time permits.

Each spec must contain a `test` block (L4 TESTABILITY).

### Phase 3 — Code (reference model)

Edit `scripts/cocotb_ref_model.py`:

1. **Argument width/signedness coercion.**
   In `_eval_call_bv`, after evaluating each argument, compare its width/signedness to
   the declared parameter type.  If the argument is an untyped literal (e.g., default
   32-bit signed) but the parameter is `u32` or `i16`, re-wrap the integer value at the
   declared width/signedness.
   - Use `_type_width_signed(ptype)` for primitive scalar params.
   - Use `_packed_type_width_signed(ctx, ptype)` for lowerable packed scalar struct
     params.

2. **Struct-literal argument sizing.**
   Ensure `_eval_struct_lit_bv` returns a `Bv` whose width matches the packed struct
   parameter.  `_packed_type_width_signed` already computes this; coerce if needed.

3. **Call return width inference.**
   Update `_type_of_expr` `ExprCall` branch to use `_packed_type_width_signed` for
   packed return types and `_type_width_signed` for scalar return types.

4. **Avoid recursive module binding for nested calls.**
   `_eval_call_bv` creates a new `EvalContext(ctx.root)`.  The new context must not
   re-enter the module-level binding loop recursively.  The W541 guard
   (`_contains_kind(init_node, "ExprCall")`) already protects module initializers; the
   same guard should be applied to any other recursive context creation.

### Phase 4 — Code (compiler)

No compiler surface change is expected for Variant A; the existing `expr_width_signed`
and probe emission already handle scalar calls.  If a new edge case is found, update
`bootstrap/src/compiler.rs` and `bootstrap/stage0/FROZEN_HASH`.

### Phase 5 — Gen

Run `./scripts/tri gen` / suite to regenerate affected outputs.  Do not hand-edit `gen/`.

### Phase 6 — Seal

Seal each new scratch witness with `t27c seal --save` and record baselines via
`./scripts/tri test --icarus-lowerable --cocotb --fast`.

### Phase 7 — Verify

Run the full validation matrix:

| Command | Expected |
|---------|----------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed / 0 failed / 2 ignored |
| `cargo test -p tri` | 78 passed / 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 4 passed / 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 42 Icarus PASS, 42 cocotb PASS, 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs / 0 `sorry` |

The 24 pre-existing yosys smoke baseline failures remain unchanged.

### Phase 8 — Land

- Commit on `wave-loop-542`.
- Update `.trinity/current_task/.commit_count` and `.trinity/current_task/session_log.jsonl`.
- Mark issue #1513 closed in commit messages.

### Phase 9 — Learn

Write the closeout report and cooperation variants:

- `docs/reports/WAVE_LOOP_542_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W543_YYYY-MM-DD.md`
- Update `.trinity/experience.md`, persistent memory, and `.claude/skills/t27-wave-loop.md`.
- Advance `.trinity/current-issue.md` to W543.

---

## 4. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Argument coercion breaks existing witnesses. | Add explicit regression checks before/after; the existing suite is the oracle. |
| Struct-literal argument widths are mismatched. | Use `_packed_type_width_signed` as the authority and re-wrap. |
| Nested call evaluation recurses through module binding. | Reuse W541 `_contains_kind` guard; create a lean call-only context. |
| Negative witness does not exist. | Add a new one or reuse `w534_negative_*`. |
| `sum(Pt{...})` call return width inference is wrong. | `_type_of_expr` fix covers scalar and packed returns. |

---

*φ² + φ⁻² = 3 | TRINITY*

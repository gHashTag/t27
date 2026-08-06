# Wave Loop 541 Plan — Module-level wide packed values for independent VCD cross-check

**Issue:** #1512 (placeholder — create when GitHub token available)  
**Branch:** `wave-loop-541`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W541_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

Wave Loop 540 closed the >64-bit VCD probe path for function-returned packed scalar
structs and struct literals, but several gaps remain in the independent VCD cross-check:

1. **Module-level declarations are invisible to the reference model.**
   `scripts/cocotb_ref_model.py` builds an `EvalContext` from the AST but only binds
   function-local variables.  Module-level `const` and `var` declarations of lowerable
   packed scalar struct (or scalar array) type are not evaluated, so assertions such as
   `assert_eq(dst, Wide{...})` or `assert_eq(GRID, ...)` fall back to log-only checking.

2. **Module-level mutation is not tracked.**
   Even if a module `var` is bound at start-up, a later `dst = make()` assignment would
   not be reflected in the reference model.  Whole-struct assignment from a function call
   is a common pattern in the W5xx witnesses and currently cannot be VCD-checked.

3. **`expr_width_signed` does not size module-level identifiers.**
   The current `ExprIdentifier` arm only resolves `local_types`/`param_types`/`module_types`
   and returns `None` for non-primitive types.  A module-level packed scalar struct
   identifier therefore never triggers the W540 multi-slice probe emission.

4. **Slice reconstruction assumes immutable probes.**
   If a wide module value is captured once and then assigned before the test block, the
   VCD snapshot still records the final value, so mutation is only a problem for the
   reference model's expected-value computation, not for the Verilog capture.

---

## 2. Literature and related work

The W541 work is a small instance of a broader pattern: **simulation-based equivalence
checking between a high-level specification and generated hardware**.

- **Leung, Bounov, Lerner — "C-to-Verilog Translation Validation" (MEMOCODE 2015).**
  Demonstrates post-hoc equivalence checking for HLS-generated Verilog, validating
  Xilinx Vivado HLS output without depending on intermediate tool state.  The t27
  cocotb gate is conceptually similar: a trusted reference model (Python AST evaluator)
  checks the generated Verilog simulation.
  [DOI](https://doi.org/10.1109/memcod.2015.7340466)

- **Melchert et al. — "Automated Translation Validation of a Compiler for Statically
  Scheduled Accelerators" (FMCAD 2025).**
  Validates every compiler stage of an accelerator toolchain using SMT-based symbolic
  transition systems and bounded model checking.  Highlights the long tail of bugs that
  simulation misses and motivates moving from ad-hoc simulation baselines toward
  systematic equivalence checking.
  [NSF PAR](https://par.nsf.gov/servlets/purl/10663798)
  [PDF](https://repositum.tuwien.at/bitstream/20.500.12708/219556/1/Melchert%20Jackson%20-%202025%20-%20Automated%20Translation%20Validation%20of%20a%20Compiler%20for...pdf)

- **Herklotz et al. — "Formal Verification of High-Level Synthesis" (OOPSLA 2021).**
  Presents Vericert, a CompCert-based verified HLS compiler from C to Verilog.  Shows the
  importance of bit-accurate width/signedness tracking when connecting a source-language
  semantics to generated Verilog.
  [PDF](https://johnwickerson.github.io/papers/vericert_oopsla21.pdf)
  [DOI](https://doi.org/10.1145/3485494)

- **Mishra, Dutt, et al. — "A methodology for validation of microprocessors using
  equivalence checking" (MTV 2003).**
  Uses an Architecture Description Language to generate a synthesizable golden reference
  model and compares it against hand-written RTL.  Reinforces the value of a trusted
  golden model derived directly from the source specification.
  [PDF](https://www.cise.ufl.edu/research/cad/Publications/mtv03.pdf)

- **VCDDiff (GitHub).**
  A practical VCD waveform diff utility.  Shows the industry pattern of comparing a golden
  simulation trace against a design trace, which is exactly what the t27 cocotb gate does
  with `[PROBE]` slices.
  [vcddiff](https://github.com/joonho3020/vcddiff)

For t27, the reference model is the golden source, the generated Verilog is the design,
and VCD slice probes are the trace.  W541 extends the golden model to cover more source
constructs.

---

## 3. Decomposed plan

### Phase 1 — Issue (already defined)

`.trinity/current-issue.md` already defines W541 and Variant A.  Verify it matches the
chosen scope.

### Phase 2 — Spec / TDD

Write three scratch specs that exercise module-level wide packed values:

1. `specs/scratch/w541_module_wide_struct_const.t27`
   - Module-level `const src : Wide = Wide{...};`
   - Test `assert_eq(src, Wide{...});`

2. `specs/scratch/w541_module_wide_struct_var.t27`
   - Module-level `var dst : Wide = Wide{...};`
   - Test `assert_eq(dst, Wide{...});`

3. `specs/scratch/w541_module_wide_struct_assign.t27`
   - Module-level `var dst : Wide;`
   - Function returning `Wide`.
   - Test body: `dst = make(); assert_eq(dst, Wide{...});`

Each spec must contain a `test` block (L4 TESTABILITY).  Total packed width must exceed
64 bits so the W540 multi-slice probe path is forced.

### Phase 3 — Code (compiler)

Edit `bootstrap/src/compiler.rs`:

- Extend `expr_width_signed` `ExprIdentifier` arm to:
  - Check `self.module_types` for lowerable packed scalar struct types.
  - If the type is a lowerable packed scalar struct, return `(element_width(base), false)`.
  - Keep the existing primitive-scalar path unchanged.
- Ensure `module_types` includes both `const` and `var` declarations (already populated
  in `gen_verilog_module` for const; verify `var` is also recorded).
- Update `bootstrap/stage0/FROZEN_HASH` after every compiler surface change.

### Phase 4 — Code (reference model)

Edit `scripts/cocotb_ref_model.py`:

- In `EvalContext.__init__`, after collecting top-level declarations, iterate over
  module-level `ConstDecl` nodes and evaluate their initializers when the declared type
  is a lowerable packed scalar struct or fixed-size scalar array.
  - Bind successful evaluations into `self.vars`.
  - For mutable vars (`extra_mutable == true`), also bind the initializer; the type
    is still a `Bv` and can be updated later if assignments are tracked.
- Add a helper `_is_lowerable_packed_type(ctx, ty)` that mirrors the compiler's notion of
  lowerable scalar struct / scalar array.
- Extend `_collect_assertions` to process statements in order:
  - Before collecting each assertion, evaluate any preceding `StmtAssign` whose LHS is a
    module-level var of lowerable packed scalar struct type and rebind `ctx.vars[lhs]`
    to the evaluated RHS.
  - Skip assignments whose RHS cannot be statically evaluated; the assertion then
    falls back to log-only verification.
- Keep the actual-width re-wrapping logic already added in W540.

### Phase 5 — Gen

Run `./scripts/tri gen` (or `./bootstrap/target/release/t27c compile-all` / suite) to
regenerate any affected `gen/` outputs.  Do not hand-edit `gen/` files.

### Phase 6 — Seal

Seal each new scratch witness with `t27c seal --save` and record Icarus baselines via
`./scripts/tri test --icarus-lowerable --cocotb --fast`.

### Phase 7 — Verify

Run the full validation matrix:

| Command | Expected |
|---------|----------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed / 0 failed / 2 ignored |
| `cargo test -p tri` | 78 passed / 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 4 passed / 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 39 Icarus PASS, 39 cocotb PASS, 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs / 0 `sorry` |

The 24 pre-existing yosys smoke baseline failures remain unchanged.

### Phase 8 — Land

- Commit on `wave-loop-541`.
- Update `.trinity/current_task/.commit_count` and `.trinity/current_task/session_log.jsonl`.
- Mark issue #1512 closed in commit messages.

### Phase 9 — Learn

Write the closeout report and cooperation variants:

- `docs/reports/WAVE_LOOP_541_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W542_YYYY-MM-DD.md`
- Update `.trinity/experience.md`, persistent memory, and `.claude/skills/t27-wave-loop.md`.
- Advance `.trinity/current-issue.md` to W542.

---

## 4. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Module `var` initializer may not be statically evaluable (e.g., depends on inputs). | Only bind/evaluate when `_eval_expr_bv` returns a value; otherwise leave unbound and log-only. |
| Assignment tracking in `_collect_assertions` duplicates statement-order semantics. | Process statements sequentially per block; document limitation for cross-block mutation. |
| `module_types` may not include mutable vars. | Inspect `gen_verilog_module` and add insertion if missing. |
| FROZEN_HASH churn. | Run `cargo run --release -- frozen-digest` from `bootstrap/` and write the operational line. |
| Witness total width ≤ 64 bits by accident. | Use a struct with at least three `u32` fields or `[5]u16` field. |

---

*φ² + φ⁻² = 3 | TRINITY*

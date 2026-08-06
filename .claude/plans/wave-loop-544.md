# Wave Loop 544 Plan — Mutable module vars and test-block call assignments for independent VCD cross-check

**Issue:** #1515 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-544`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W544_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

After Wave Loop 543, the cocotb reference model can evaluate module-level
`const` initializers that are lowerable function calls.  Two runtime gaps remain:

1. **Mutable module vars initialized by function calls.**  In
   `scripts/cocotb_ref_model.py`, the module-binding loop only processes
   `ConstDecl` nodes.  Mutable module vars are also represented as `ConstDecl`
   with `extra_mutable = true`, so the binding logic does run for them, but the
   W543 implementation only added the recursion fix inside the const path.  We
   need a witness to confirm that mutable vars with call initializers actually
   work end-to-end.

2. **Whole-struct assignments inside test blocks whose RHS is a function call.**
   `_collect_assertions` already evaluates `StmtAssign` to mutable module vars and
   updates `ctx.vars[lhs]` before each assertion.  However, no witness has ever
   exercised the case where the RHS of that assignment is a function call rather
   than a struct literal or identifier.

3. **Variant B edge cases.**  Nested call initializers, initializers that depend
   on previously bound module consts, and function calls returning fixed-size
   scalar arrays used as module initializers need explicit coverage to ensure the
   recursion fix and declaration-order binding are robust.

---

## 2. Literature and related work

- **CIRCT `sim.func.dpi` — Simulation Dialect DPI.**
  Hardware simulation of function calls requires explicit handling of call-state
  lifetimes and port bindings.  The same lifetime concern appears in t27 when a
  module-level mutable variable is initialized or updated by a function call:
  the callee context must not recursively re-enter module elaboration.
  [Simulation Dialect DPI - CIRCT](https://circt.llvm.org/docs/Dialects/SimDPI/)

- **K-CIRCT: A Layered, Composable, and Executable Formal Semantics for CIRCT
  Hardware IRs (2024).**
  Executable reference semantics for hardware IRs, including module state and
  function calls, validated against simulation waveforms.  Supports the t27
  strategy of a Python golden model plus VCD probes.
  [arXiv](https://arxiv.org/html/2404.18756v1)

- **Interaction Tree Semantics for RISC-V — Bridging Compiler and Hardware
  Verification (2026).**
  Demonstrates how to model function-call argument/result binding across
  compiler/hardware boundaries with a shared executable semantics.  Relevant to
  eventual formalization in Lean.
  [arXiv](https://arxiv.org/html/2605.04933v1)

- **CompCert — Verified Compilation of Floating-Point Programs (Leroy et al.).**
  While not hardware-specific, CompCert's treatment of mutable variable
  initialization and assignment as distinct events in the operational semantics
  reinforces the value of explicit witnesses for both `var init = call()` and
  `var = call()` patterns.

For t27, W544 completes the runtime coverage of call-related module state before
moving to formalization.

---

## 3. Decomposed plan

### Phase 1 — Issue

`.trinity/current-issue.md` already defines W544 and Variant A.  Verify scope and
ensure issue #1515 is referenced in all commits.

### Phase 2 — Spec / TDD

Write scratch specs that exercise the remaining mutable-state call gaps:

1. `specs/scratch/w544_module_var_scalar_call_init.t27`
   ```t27
   module w544_module_var_scalar_call_init;
   pub fn inc(x : u32) -> u32 { return x + 1; }
   var v : u32 = inc(4);
   test var_scalar_call_init {
       assert_eq(v, 5);
       v = inc(v);
       assert_eq(v, 6);
   }
   endmodule
   ```

2. `specs/scratch/w544_module_var_struct_call_assign.t27`
   ```t27
   module w544_module_var_struct_call_assign;
   struct Pt { x: i16, y: i16 }
   pub fn mirror(p : Pt) -> Pt {
       return Pt{ .x = -p.x, .y = -p.y };
   }
   var v : Pt = Pt{ .x = 1, .y = -2 };
   test var_struct_call_assign {
       v = mirror(v);
       assert_eq(v.x, -1);
       assert_eq(v.y, 2);
   }
   endmodule
   ```

3. Variant B witnesses:
   - `w544_nested_call_init.t27` — `const x : u32 = inc(inc(1));`.
   - `w544_call_init_depends_on_const.t27` — a const initialized by a call that
     takes another module const as an argument.
   - `w544_call_init_returns_array.t27` — a function returning `[3]u8` used as a
     module initializer.
   - `w544_negative_nonlowerable_var_call_init.t27` — a mutable `var` initialized
     by a call returning `String`; must be rejected by the classifier.

### Phase 3 — Code (reference model)

Edit `scripts/cocotb_ref_model.py`:

1. **Mutable var call-initializer binding is already enabled** by the W543 change
   because the module-binding loop processes all `ConstDecl` nodes regardless of
   `extra_mutable`.  Verify with the first witness.  If an issue appears, ensure
   the call-context flag is active for mutable var initializers too.

2. **Test-block assignment with call RHS is already enabled** by the existing
   `StmtAssign` update path in `_collect_assertions`.  Verify with the second
   witness.  If an issue appears, ensure `_eval_expr_bv` on the RHS uses a
   call-only context when evaluating the call.

3. No compiler change is expected for Variant A unless a new lowering bug is
   exposed.

### Phase 4 — Code (compiler)

No compiler surface change is expected for Variant A; the existing Verilog
backend already lowers mutable module vars initialized or assigned by function
calls.  If a new edge case is found, update `bootstrap/src/compiler.rs` and
`bootstrap/stage0/FROZEN_HASH`.

### Phase 5 — Gen

Run `./scripts/tri gen` / suite to regenerate affected outputs.  Do not hand-edit `gen/`.

### Phase 6 — Seal

Seal each new scratch witness with `t27c seal --save` and record Icarus baselines
via `./scripts/tri test --icarus-lowerable --cocotb --fast`.

### Phase 7 — Verify

Run the full validation matrix:

| Command | Expected |
|---------|----------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed / 0 failed / 2 ignored |
| `cargo test -p tri` | 78 passed / 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 5+ passed / 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 0 cocotb failures, 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | green / 0 `sorry` |

The 24 pre-existing yosys smoke baseline failures remain documented and unchanged.

### Phase 8 — Land

- Commit on `wave-loop-544`.
- Update `.trinity/current_task/.commit_count` and `.trinity/current_task/session_log.jsonl`.
- Mark issue #1515 closed in commit messages.

### Phase 9 — Learn

Write the closeout report and cooperation variants:

- `docs/reports/WAVE_LOOP_544_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W545_YYYY-MM-DD.md`
- Update `.trinity/experience.md`, persistent memory, and `.claude/skills/t27-wave-loop.md`.
- Advance `.trinity/current-issue.md` to W545.

---

## 4. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Mutable var call initializer is not actually bound. | Add explicit witness; the W543 path already processes mutable ConstDecl nodes. |
| Test-block assignment with call RHS fails because the call context re-enters module binding. | The W543 `bind_module_initializers=False` fix in `_eval_call_bv` covers this. |
| Scalar-array return initializer exposes a width/layout mismatch. | Add dedicated Variant B witness. |
| Non-lowerable mutable var call init is accepted by classifier. | Add negative witness and integration test. |

---

*φ² + φ⁻² = 3 | TRINITY*

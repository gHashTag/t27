# Wave Loop 543 Plan — Function-call module initializers for independent VCD cross-check

**Issue:** #1514 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-543`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W543_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

After Wave Loop 542, the cocotb reference model can evaluate scalar function-call
arguments and compare scalar call results against VCD probes.  The largest remaining
runtime gap is **module-level const/var initializers that are function calls**.

Examples that still skip the independent VCD check:
- `const src : Wide = make(); assert_eq(src, Wide{...});` — the actual expression
  is a module-level identifier whose initializer is a function call.
- `var out : u32 = compute(5); assert_eq(out, 10);` — a mutable module var
  initialized by a scalar function call.

In `scripts/cocotb_ref_model.py`, the `EvalContext.__init__` module-binding loop
explicitly skips any initializer that contains `ExprCall`:

```python
if _contains_kind(init_node, "ExprCall"):
    continue
```

The skip is defensive: `_eval_call_bv` currently creates a brand-new
`EvalContext(ctx.root)` to evaluate the callee body.  That re-enters the same
module-binding loop, which would recursively evaluate call-initialized module
consts and potentially infinite-loop.  The fix is to break the recursion by giving
call-evaluation contexts a flag that disables eager module binding while still
inheriting already-bound values from the outer context.

---

## 2. Literature and related work

- **K-CIRCT: A Layered, Composable, and Executable Formal Semantics for CIRCT
  Hardware IRs (2024).**
  Demonstrates executable reference semantics for hardware IR, including function
  calls and module-level bindings, validated against simulation waveforms.
  Relevant to the strategy of treating the Python evaluator as a golden oracle.
  [arXiv](https://arxiv.org/html/2404.18756v1)

- **Interaction Tree Semantics for RISC-V — Bridging Compiler and Hardware
  Verification (2026).**
  Uses a shared executable semantics to relate high-level programs to generated
  hardware; function-call argument and result binding are central to the
  equivalence relation.  Supports the t27 approach of a Python golden model.
  [arXiv](https://arxiv.org/html/2605.04933v1)

- **EPEX — Processor Verification by Equivalent Program Execution (JKU, 2021).**
  Emphasizes the value of a trusted reference model for comparing architectural
  states across implementations; module-level bindings and call boundaries are
  key state-transfer points.
  [PDF](https://ics.jku.at/files/2021GLSVLSI_EPEX.pdf)

- **CIRCT `sim.func.dpi` — Simulation Dialect DPI.**
  Shows how function calls are lowered to explicit ports/state in hardware
  simulation, confirming that function-result module initializers are a standard
  hard gap in hardware-reference-model equivalence checking.
  [Simulation Dialect DPI - CIRCT](https://circt.llvm.org/docs/Dialects/SimDPI/)

For t27, W543 removes the last large runtime gap in the independent cocotb gate
by letting module-level bindings evaluate call-initialized lowerable packed values.

---

## 3. Decomposed plan

### Phase 1 — Issue

Advance `.trinity/current-issue.md` to Wave Loop 543 (#1514 placeholder).  Ensure
all commits reference `Closes #1514`.

### Phase 2 — Spec / TDD

Write scratch specs that exercise function-call module initializers:

1. `specs/scratch/w543_module_scalar_call_init.t27`
   ```t27
   module w543_module_scalar_call_init;
   pub fn make(x : u32) -> u32 { return x + 1; }
   const src : u32 = make(5);
   test scalar_call_init {
       assert_eq(src, 6);
       assert_eq(make(0), 1);
   }
   endmodule
   ```

2. `specs/scratch/w543_module_struct_call_init.t27`
   ```t27
   module w543_module_struct_call_init;
   struct Pt { x: i16, y: i16 }
   pub fn neg(p : Pt) -> Pt {
       return Pt{ .x = -p.x, .y = -p.y };
   }
   const src : Pt = neg(Pt{ .x = 3, .y = -4 });
   test struct_call_init {
       assert_eq(src.x, -3);
       assert_eq(src.y, 4);
   }
   endmodule
   ```

3. `specs/scratch/w543_module_mixed_call_init.t27` (Variant B overlap)
   ```t27
   module w543_module_mixed_call_init;
   struct Wide { a: u16, b: i16 }
   pub fn combine(a : u16, b : i16) -> Wide {
       return Wide{ .a = a, .b = b };
   }
   const src : Wide = combine(7, -8);
   test mixed_call_init {
       assert_eq(src.a, 7);
       assert_eq(src.b, -8);
   }
   endmodule
   ```

4. Optional Variant B adversarial witnesses:
   - `w543_call_arg_casts.t27` — arguments passed through narrowing/widening casts.
   - `w543_negative_nonlowerable_call.t27` — a module initializer returning a
     non-lowerable type; the cocotb gate must skip it without failing.

Each spec must contain a `test` block (L4 TESTABILITY).

### Phase 3 — Code (reference model)

Edit `scripts/cocotb_ref_model.py`:

1. **Break recursion between module binding and call evaluation.**
   - Add an optional `bind_module_initializers: bool = True` parameter to
     `EvalContext.__init__`.
   - Wrap the module-const binding loop in `if bind_module_initializers:`.
   - In `_eval_call_bv`, create the callee context with
     `EvalContext(ctx.root, bind_module_initializers=False)`.
   - Keep `call_ctx.vars.update(ctx.vars)` so the callee sees outer bindings
     (module consts already bound in declaration order).
   - Keep `call_ctx.fn_local_types = ctx.fn_local_types` so local/parameter type
     maps remain available.

2. **Remove the `_contains_kind(init_node, "ExprCall")` skip.**
   Once recursion is broken, module-level initializers that are lowerable calls
   can be bound eagerly like any other initializer.

3. **Ensure mutable module vars with call initializers are handled.**
   The existing `mutable_module_names` tracking and `StmtAssign` update path in
   `_collect_assertions` should work unchanged, but add a witness to confirm.

### Phase 4 — Code (compiler)

No compiler surface change is expected for Variant A; the existing Verilog backend
already lowers module consts/vars initialized by function calls.  If a new edge
case is found, update `bootstrap/src/compiler.rs` and `bootstrap/stage0/FROZEN_HASH`.

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
| `cargo test -p t27c --bin t27c` | baseline passed / 0 failed / 2 ignored |
| `cargo test -p tri` | baseline passed / 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | baseline passed / 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 0 cocotb failures, 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | green / 0 `sorry` |

The pre-existing yosys smoke baseline failures remain documented and unchanged.

### Phase 8 — Land

- Commit on `wave-loop-543`.
- Update `.trinity/current_task/.commit_count` and `.trinity/current_task/session_log.jsonl`.
- Mark issue #1514 closed in commit messages.

### Phase 9 — Learn

Write the closeout report and cooperation variants:

- `docs/reports/WAVE_LOOP_543_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W544_YYYY-MM-DD.md`
- Update `.trinity/experience.md`, persistent memory, and `.claude/skills/t27-wave-loop.md`.
- Advance `.trinity/current-issue.md` to W544.

---

## 4. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Removing the `ExprCall` skip creates infinite recursion for circular module initializers. | The `bind_module_initializers=False` flag prevents re-entry; only AST-order forward references are supported, matching existing non-call initializer behavior. |
| Mutable module var call initializers update incorrectly. | Add an explicit witness and reuse the existing `StmtAssign` update path. |
| Existing witnesses break because `EvalContext` defaults change. | Keep the default `bind_module_initializers=True` so only call contexts change. |
| Non-lowerable call-initialized consts cause a hard failure. | The binding loop already checks lowerability; non-lowerable types will simply remain unbound, falling back to log-only verification. |

---

*φ² + φ⁻² = 3 | TRINITY*

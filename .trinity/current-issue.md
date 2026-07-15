# Wave Loop 543 — Function-call module initializers for independent VCD cross-check

**Issue:** #1514 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-543`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 542's scalar function-call argument coverage and remove the
last large runtime gap in the independent VCD cross-check: module-level const/var
initializers that are function calls.  Once this is done, assertions such as
`const src : Wide = make(); assert_eq(src, Wide{...});` will receive an independent
VCD cross-check rather than falling back to the log-based self-check.

**Recommended cooperation variant:** Variant A from
`docs/reports/FPGA_LOOP_COOPERATION_W543_2026-07-07.md`.

---

## Concrete deliverables

1. **Python reference model** (`scripts/cocotb_ref_model.py`)
   - Refactor `EvalContext.__init__` so that module-level initializer evaluation and
     function-body evaluation share a single call-evaluation helper that does not
     recursively re-enter the module-binding loop.
   - Options:
     - Add an optional `skip_module_binding` flag when creating `EvalContext` for
       callee evaluation, or
     - Build module bindings lazily on first use rather than eagerly in `__init__`.
   - Bind function-call module initializers once the recursion is broken.

2. **Scratch witnesses** (`specs/scratch/`)
   - `w543_module_scalar_call_init.t27` — module const initialized by a scalar
     function call.
   - `w543_module_struct_call_init.t27` — module const initialized by a packed
     scalar struct function call.
   - Seal each witness and record Icarus baselines.

3. **Negative witness**
   - `w543_negative_nonlowerable_call_init.t27` (or reuse an existing negative
     witness): confirm that non-lowerable function-call initializers are still
     skipped gracefully without failing the cocotb gate.

4. **Validation**
   - `cargo build --release -p t27c` green.
   - `cargo test -p t27c --bin t27c` 1494 passed / 0 failed / 2 ignored.
   - `cargo test -p tri` 78 passed / 0 failed.
   - `cargo test -p t27c --test icarus_lowerable` 4 passed / 0 failed.
   - `./scripts/tri test --icarus-lowerable --cocotb --fast`: 0 cocotb failures, 0
     seal mismatches (24 pre-existing yosys smoke baselines remain).
   - `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`: 0 `sorry`.

---

## Residual boundaries from W542

- `./scripts/tri test --icarus-lowerable --cocotb --fast` is green:
  42 Icarus simulations passed, 0 failed; 42 cocotb reference-model checks passed,
  0 failed; 0 seal mismatches.
- 24 pre-existing yosys smoke baseline failures remain documented and unchanged.
- Module-level const/var initializers that are function calls still skip the
  independent VCD check.
- The signed-to-unsigned cast sign-extension fix from W542 is verbose but correct.

---

*φ² + φ⁻² = 3 | TRINITY*

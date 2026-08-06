# Wave Loop 526 Closeout Report

**Issue:** #1497 (placeholder)  
**Branch:** `wave-loop-526`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 526 executed **Variant A** from `.claude/plans/wave-loop-526.md`: it converted the unlanded W469 2-D array-of-struct Verilog regression from a silent, broken-code failure into a clear compile-time diagnostic, and produced the design document needed for full implementation in Wave Loop 527.

---

## Weak points investigated

1. **W469 regression is still unlanded.** A declaration such as `var m : [2][3]Pt = ...` is truncated by the parser and the Verilog backend emits incomplete placeholder modules.
2. **Failure is silent.** Until W526 the spec passed `t27c gen-verilog` and yosys smoke despite being semantically broken.
3. **No design doc existed.** There was no single place describing the parser, typechecker, and emitter changes required for full lowering.
4. **IcarusLowerable Lean 4 stack is not on `master`.** Any new lowering work on `master` currently has no formal soundness scaffold.
5. **Master test baseline has pre-existing failures.** `cargo test -p t27c --bin t27c` reports 3 unrelated `let_binding` failures on clean HEAD; `./scripts/tri test` reports 130 failures driven by seal mismatches.

---

## Scientific / engineering research

- **Vitis HLS aggregate data-layout rules** — AoS improves random-access locality, SoA improves streaming bandwidth; t27 needs an explicit policy.
- **Vericert (Wickerson et al.)** — demonstrates verified HLS from C to Verilog using CompCert-style forward simulation.
- **CompCert (Leroy et al.)** — the canonical source-to-target simulation-invariant architecture; directly applicable to t27 backend proofs.
- **Roofline model (Williams, Waterman, Patterson)** — guides the memory/compute trade-off when choosing packed-vector (AoS) vs. split-memory (SoA) lowering.

---

## Implementation

| File | Change |
|------|--------|
| `bootstrap/src/compiler.rs` | Added `Compiler::detect_unsupported_verilog_locals` and called it at the start of `compile_verilog`. |
| `bootstrap/stage0/FROZEN_HASH` | Updated to the new compiler.rs hash (`dc71c407...`). |
| `specs/scratch/w526_2d_struct_array_repro.t27` | Negative witness with documented expected semantics. |
| `docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md` | Parser/typechecker/emitter design + reseal strategy + soundness plan. |
| `.claude/plans/wave-loop-526.md` | Decomposed plan and acceptance criteria. |

### Diagnostic now emitted

```text
unsupported multi-dimensional array of aggregate type `[2][3]Pt` for local variable `m` at line 12:
2-D array-of-struct lowering is not yet implemented (see docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md)
```

---

## Verification

- `t27c gen-verilog specs/scratch/w526_2d_struct_array_repro.t27` exits non-zero with the diagnostic above.
- `t27c gen-verilog specs/scratch/w387_2d_local_array.t27` still passes (primitive 2-D arrays unaffected).
- `cargo test -p t27c --bin t27c`: 1491 passed, 3 failed, 2 ignored — matches the pre-existing master baseline.
- `./scripts/tri test`:
  - Parse failures: 0
  - Typecheck fails: 0
  - Gen Zig failures: 0
  - Gen Rust failures: 0
  - Gen C failures: 0
  - **Gen Verilog failures: 1** (the new W526 witness, expected)
  - **Gen Verilog smoke fails: 17** (+1 from the witness)
  - Seal mismatches: 114 (unchanged)
  - **Total failures: 132** (+2 from the expected witness)

---

## Next Wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W527_2026-08-11.md`.

1. **Variant A (recommended):** implement full 2-D scalar-struct array lowering.
2. **Variant B:** formalize 1-D AOS value preservation in `Trinity.IcarusLowerable` while keeping the W526 diagnostic.
3. **Variant C:** process-improvement epic (issue-existence L1 validation, seal-drift CI, landing plan for W469–W525 codegen delta).

---

## Learnings

- A clear early diagnostic is better than silently passing smoke tests with broken generated code.
- The `FROZEN_HASH` ceremony must be performed for every change to `bootstrap/src/compiler.rs`.
- The current `master` baseline already carries unrelated `let_binding` test failures and seal mismatches; new work must be measured against the clean-HEAD baseline, not against an ideal zero-failure state.

---

*φ² + φ⁻² = 3 | TRINITY*

# Wave Loop 369 — Cooperation Variants for Wave Loop 370

**Issue:** [#1257](https://github.com/gHashTag/t27/issues/1257)  
**Date:** 2026-07-02  
**Predecessor target achieved:** 220 generic ∀, 45-variable accumulation, duovigintuple cancellation, zero-weight duodecuple closure, binary-width padding in `gen-verilog`, full conformance 548/548 PASS.

---

## Executive Recommendation

**Choose Variant B.** It keeps the formal-moat machine running (+4 generic ∀), retries the only unsolved hardware blocker, and adds one more safe `gen-verilog` sub-fix. It balances incremental risk against incremental moat and is the natural continuation of the W367→W369 pattern.

---

## Variant A — Formal-only (safe, no hardware dependency)

### Scope
- Push the Lean 4 proof lattice to **224 generic ∀**.
- Add:
  - `ternaryMacAccumulateFortySixPlusGeneric` — 46-variable plus accumulation.
  - `ternaryMacAccumulateFortyFiveMinusGeneric` — 45-variable minus accumulation.
  - `ternaryMacTresvigintupleCancellationGeneric` — depth-23 identity/residual cancellation.
  - `ternaryMacZeroWeightTredecupleClosureGeneric` — 13 zero-weight MACs around 1 plus-weight MAC (6+1+6 or 7+1+5 shape; choose the symmetric form).
- Extend all 27 IGLA specs with W370 blocks (+2 tests, +1 invariant each).
- Regenerate 27 IGLA seals.
- Run full conformance; gate: **548/548 PASS**.

### Pros
- Zero external dependencies.
- Continues the 100+ wave zero-IGLA-failure streak.
- Deepens the quantified-accumulation moat to 46 variables.

### Cons
- No progress on silicon evidence.
- No progress on backend hardening beyond what W369 already delivered.

### Accept when
- The DLC10 cable is still unavailable and the team wants a guaranteed green wave.

---

## Variant B — Formal + board flash retry + one safe gen-verilog sub-fix (recommended)

### Scope
- Everything in **Variant A** (224 generic ∀, 46/45 plus/minus, depth-23 cancellation, 13-zero closure).
- **Hardware:** retry `dlc10 idcode` → `sram` → `flash` with `fpga/verilog/ternary_mac_demo_top.bit`. If the cable is found, capture IDCODE and attempt SRAM load first; if that succeeds, flash the bitstream and document the full sequence in `docs/reports/FPGA_EVIDENCE_W370.md`.
- **Backend:** pick **one** of the following safe `gen-verilog` sub-fixes, gated by a new scratch spec and `yosys read_verilog`:
  - **B1:** Fix defect 1 — emit *all* `const` declarations, not only the first.
  - **B2:** Fix defect 3 — preserve early `return` inside bare `if` blocks.
  - **B3:** Fix defect 4 — preserve the body of `as` cast and bitwise operator expressions.
  - **B4:** Fix defect 5 — correct struct-field reg name generation so the emitted identifier matches the field access path.
  - **B5:** Add a CI smoke gate that runs `t27c gen-verilog` + `yosys read_verilog` on a set of scratch regression specs (hex width, binary width, const ordering, bare-if return).

The recommended sub-fix order is **B5 first** (it prevents regression of the fixes already landed), then **B1** (highest impact on real generated code).

### Pros
- Drives the formal moat and attempts the last unsolved physical proof step.
- Adds backend hardening with low blast radius.
- Even if the board is still missing, the deliverables in Variant A remain intact.

### Cons
- Board flash is still gated by an external cable; probability of success is low.
- The chosen sub-fix may require touching more of `gen_verilog_stmt`/`gen_verilog_expr` than the W369 `0b` change.

### Accept when
- The team wants maximum moat progress while continuing to close the silicon-evidence gap.

---

## Variant C — Formal + RTL-to-Lean traceability prototype + board flash (higher risk, deeper moat)

### Scope
- Everything in **Variant B**.
- Add a **traceability prototype** that links generated Verilog modules back to their Lean 4 proof obligations:
  - Emit a `// Proof obligation: Trinity.TernaryInference.<theorem>` comment in `gen-verilog` output for guarded ternary-MAC modules.
  - Generate a small JSON sidecar mapping each emitted module/instance to the theorems that cover its arithmetic behavior.
  - Add one end-to-end test: compile a `.t27` spec → generate Verilog → parse the sidecar → assert the named Lean theorem exists and builds.

### Pros
- Creates a defensible, hard-to-replicate moat: competitors have silicon metrics or isolated theorems, but not a *traceable* spec → code → proof pipeline.
- Positions t27 for submission to an RTL formal-verification benchmark (RTL-BenchLS, RealBench) in a later wave.

### Cons
- The traceability prototype is a multi-file change across the compiler, the Verilog backend, and the Lean namespace indexer. It may not fit cleanly in one wave.
- Risk of missing the 224 generic ∀ target if the prototype consumes too much time.

### Accept when
- The hardware retry is expected to fail quickly and the team is willing to trade some formal depth for a structural differentiator.

---

## Decision Matrix

| Criterion | Variant A | Variant B (recommended) | Variant C |
|---|---|---|---|
| Generic ∀ target | 224 | 224 | 224 |
| Hardware retry | No | Yes | Yes |
| Backend sub-fix | No | 1 safe fix | 1 safe fix |
| Traceability prototype | No | No | Yes |
| Risk | Low | Low–Medium | Medium–High |
| Moat depth | Deep | Deep + broader | Deep + structural |
| Recommended if cable still missing | Yes | Yes | Only if prototype is scoped |

---

## Suggested W370 Commit Message Template

```text
feat(igla): Wave Loop 370 — 46-variable accumulation, tresvigintuple cancellation, zero-weight tredecuple closure, board flash retry, gen-verilog <sub-fix>

- 224 generic ∀ theorems in Trinity.TernaryInference.lean
- 46-variable plus accumulation / 45-variable minus lattice
- Depth-23 cancellation theorem
- 13-zero-weight closure theorem
- 27 IGLA specs extended, all seals regenerated
- Full conformance: 548/548 PASS
- Board flash attempt documented in docs/reports/FPGA_EVIDENCE_W370.md
- Safe gen-verilog sub-fix: <B1/B2/B3/B4/B5>
- Closes #1258
```

---

*phi² + 1/phi² = 3 | TRINITY*

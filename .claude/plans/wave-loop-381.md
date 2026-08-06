# Wave Loop 381 Plan

**Issue:** #1272
**Branch:** `trinity-rust-rings`
**Selected variant:** B (recommended)

---

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **115 waves**, push Lean 4 generic ∀ to **268**, and finish **slot-aware nested tuple-return call lowering** in the `gen-verilog` backend.

---

## Variant B — proof push + close tuple-return call lowering

### 1. Lean 4 proof lattice (4 new generic ∀)

Target theorems in `proofs/lean4/Trinity/TernaryInference.lean`:

1. `ternaryMacAccumulateFiftyNinePlusGeneric` — 59-variable plus accumulation.
2. `ternaryMacAccumulateFiftyEightMinusGeneric` — 58-variable minus lattice.
3. `ternaryMacDuotrigintupleSeptemCancellationGeneric` — `mac^38(x, a, [.plus,.minus,...]) = x` (depth-38 identity cancellation). Use an even depth to avoid residual mismatch.
4. `ternaryMacZeroWeightFifteenPairClosureGeneric` — 15 zero-weight MACs before and after a plus-weight MAC are transparent (31 variables, 30 zero-weight MACs).

This reaches **268 generic ∀**.

### 2. Gen-verilog: slot-aware nested tuple-return call lowering

The W380 scaffolding made multi-return function declarations and tuple literals work. The remaining gap is when a tuple-returning caller wants to use an element directly, e.g.:

```t27
fn inner(a: u32, b: u32) -> (u32, u32) {
    return (a, b);
}
fn outer(p: u32, q: u32) -> (u32, u32) {
    let(x, y) = inner(p, q);
    return (y, x);
}
fn use() -> u32 {
    let(u, v) = outer(1, 2);
    return u + v;
}
```

Currently the `let` destructuring path assumes the RHS is a direct call to a function whose return type is known. W381 must ensure this works transitively and that packed temporary widths match across nested calls.

Implementation in `bootstrap/src/compiler.rs`:

- Verify `fn_return_types` registry is populated before any function body is lowered (it already is, but confirm ordering).
- Add a scratch regression spec `specs/scratch/w381_tuple_call_chain.t27` with the nested pattern above.
- If needed, extend `gen_verilog_let_destructuring` to handle the case where the LHS binding type is explicitly annotated with a width different from the callee element width (currently it prefers explicit type, which is correct).
- Add yosys smoke verification.

### 3. Conformance and sealing

- Run `t27c suite --repo-root .`; capture any seal mismatches.
- Reseal affected specs.
- Run again until 0 failures.
- Run `lake build Trinity.TernaryInference`.

### 4. Documentation and memory

- Write `docs/reports/WAVE_LOOP_381_REPORT.md`.
- Write `docs/reports/WAVE_LOOP_381_COOPERATION.md` with three W382 variants.
- Update `docs/reports/FPGA_EVIDENCE_W381.md`.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to mark nested tuple-call lowering complete.
- Update `.trinity/experience.md`.
- Save memory `wave-loop-381.md` and update `MEMORY.md` index.

### 5. Commit

- Commit with `Closes #1272`.
- Because `origin/trinity-rust-rings` has diverged significantly, push to a new branch (`w381-local`) and open PR #1273, referencing #1272.

---

## Risks and mitigations

- **Lean timeout at depth 59:** if `omega` saturates, fall back to 58 plus / 57 minus.
- **Nested tuple call lowering exposes new parser/codegen edge cases:** keep the regression spec minimal; if a wider fix is unsafe, scope to the minimal verifiable pattern and document the remaining gap.
- **Remote divergence:** avoid rebasing the full local history; use topic branch PRs.

---

*phi² + 1/phi² = 3 | TRINITY*

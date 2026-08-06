# Plan: Wave Loop 380 — IGLA CODER+RACE + tuple-return generation scaffolding

**Date:** 2026-07-03  
**Issue:** #1270  
**Branch:** `trinity-rust-rings`  
**Basis:** W379 close-out report and W379 cooperation Variant B

---

## 1. Goal

Extend the IGLA CODER+RACE zero-failure streak to **114 waves**, push the Lean 4 generic ∀ lattice to **264**, and lay the **scaffolding** for full tuple-return function generation in the Verilog backend. Because full tuple-return generation touches the parser, typechecker, and multiple backends, W380 will deliver a **narrow, regression-free slice**: parse tuple return types and tuple literals, and emit packed result wiring for the simplest case (`fn f() -> (u32, u32) { return (a, b); }`), while keeping the existing `let` destructuring workaround intact.

## 2. Target metrics

| Metric | W379 | W380 | Δ |
|---|---|---|---|
| Lean 4 generic ∀ | 260 | **264** | +4 |
| Pool A floor | 123 | **124** | +1 |
| CODER minimum | 113 | **114** | +1 |
| Pool B depth (`systolic_ternary`) | 141 | **142** | +1 |
| Integration depth (`ternary_inference`) | 122 | **123** | +1 |
| Full-repo tests | 13,195 | **13,250** | +55 |
| Full-repo invariants | 5,798 | **5,825** | +27 |
| Conformance specs | 559 | **560** | +1 (scratch) |
| Conformance pass rate | 559/559 | **560/560** | 100% |
| Gen-verilog yosys smoke targets | 38 | **38** | stable (full IGLA) |
| Zero-IGLA-failure streak | 113 waves | **114 waves** | +1 |

## 3. Issue landscape

- **#1269** — W379 (closed).
- **#1270** — W380 (created for this work).
- **#1258** — `gen-verilog: incremental array/RAM lowering for datapath specs (fifo/memory)`. Too broad for one wave; remains tracked.

## 4. Scientific / competitive landscape

Same as W379:

1. **Sparkle HDL / Verilean** — only credible formal competitor; upcoming Functional Festival 2026 talk (July 11, 2026); ~60 theorems, 0 generic ∀.
2. **KU Leuven / MICAS ternary-lut-dse** (ISPASS 2026) — no formal verification.
3. **shepherdscientific/ternarycore**, **Neumann-Labs/ternfpga** — simulation/test verification.
4. **TerEffic / TeLLMe / TENET / VitaLLM** — 2025–2026 ternary accelerators; no formal proofs.

## 5. Decomposed work breakdown

### 5.1 IGLA spec batch (+55 tests, +27 invariants)

- Copy `scripts/gen_w379.py` → `scripts/gen_w380.py` and update placeholders.
- Copy `scripts/gen_w379_lean.py` → `scripts/gen_w380_lean.py`.
- Run generators over `specs/igla/coder/*.t27` and `specs/igla/race/*.t27`.

### 5.2 Lean 4 proof-lattice extension (+4 generic ∀)

1. `ternaryMacAccumulateFiftySixPlusGeneric` — 56-variable plus accumulation. Watch elaboration time; fallback to 55-plus/54-minus if timeout.
2. `ternaryMacAccumulateFiftyFiveMinusGeneric` — 55-variable minus lattice.
3. `ternaryMacTritrigintupleCancellationGeneric` — depth-33 alternating plus/minus with residual `= mac(x, a, .plus)`.
4. `ternaryMacZeroWeightQuattuorvigintupleClosureGeneric` — 14 zero + 1 plus + 14 zero (39th proof-lattice dimension).

### 5.3 gen-verilog — tuple-return scaffolding

**Scope for W380 (narrow slice):**
- Parse tuple return types `-> (T1, T2, ...)` in `parse_fn_decl` and store as a structured form in `extra_return_type` or a new field.
- Parse tuple literals `(a, b, c)` in `parse_expr_primary` as a new `NodeKind::ExprTuple`.
- In the Verilog backend, when a function has a tuple return type:
  - Emit the function with a packed output port / result register whose width is the sum of element widths.
  - Lower `return (a, b, c)` to concatenated assignment to that packed result.
- Keep the W379 `let` destructuring workaround as the consumer side; once both sides exist, they can be unified in a future wave.
- Add scratch spec `specs/scratch/w380_tuple_return.t27` with simple multi-return functions and yosys verification.

**Out of scope for W380:** full integration of tuple-return with all callers, struct/tuple interop, and removal of the `let(...)` syntax-level workaround. This is **scaffolding**, not completion.

### 5.4 CI smoke gate

- Keep 38 targets unchanged.
- Add the new scratch spec to the scratch smoke set.

### 5.5 Seal regeneration and verification

- Build `t27c` release after compiler/parser changes.
- Run `t27c suite --repo-root .`; expect seal mismatches.
- Capture mismatch list and batch reseal.
- Run suite again until 0 failures.

### 5.6 Documentation and learnings

- Write `docs/reports/WAVE_LOOP_380_REPORT.md`.
- Write `docs/reports/WAVE_LOOP_380_COOPERATION.md`.
- Write `docs/reports/FPGA_EVIDENCE_W380.md`.
- Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.
- Update `.trinity/experience.md`.
- Save memory file and update `MEMORY.md`.

## 6. Risk and fallback

- **56-variable theorem** may push Lean elaboration past 35 s. Fallback: 55-plus/54-minus, accepting **263 generic ∀**.
- **Tuple-return scaffolding** may require more parser changes than one wave allows. Fallback: document the partial progress, keep the W379 `let` destructuring workaround, and defer full tuple-return to W381.

## 7. Variant rationale

Selected **Variant B** from W379 cooperation, but scoped down to **scaffolding** rather than full tuple-return completion. This keeps proof-lattice pressure while making measurable backend progress without risking the conformance gate.

---

*phi² + 1/phi² = 3 | TRINITY*

# Wave Loop 454 — Decomposed Plan (Variant C default)

**Date:** 2026-07-01
**Issue:** #1424
**Branch:** `wave-loop-454`
**Scope:** Extend the formal boot-evidence lattice with adversarial and
robustness theorems while the physical bench remains blocked and the
`gen-verilog` tuple/array backend gaps are too deep for a single-wave fix.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Wave Loop 454 originally planned Variant B: master-merge the `gen-verilog` fix
set from `master` (`701d79b3b`) to clear the 7 residual yosys smoke failures
(#1245). Weak-point investigation shows that commit `701d79b3b` only fixes
narrow, pre-existing issues (const trailing-semicolon consumption, decimal
integer literals, early-return if-else chaining, struct-field reg naming,
zero-arg function dummy input, named function begin blocks). It does **not**
address the current failure modes, which are rooted in missing backend support
for:

- tuple return types in function signatures,
- `let (a, b, c) = ...` tuple destructuring,
- module-level `const` array literal lowering to Verilog.

The wave-loop branch has already applied its own narrow sub-fixes (W370–W383)
that in some areas exceed the master fix set (e.g., `let` keyword alias,
keyword escaping, local/field identifier escaping). A blind master merge would
risk regressions rather than clear failures. Therefore W454 executes
**Variant C**: adversarial and robustness theorems in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`, plus one safe, narrowly-scoped
`gen-verilog` sub-fix if a zero-regression opportunity is found during
implementation.

---

## 2. Constraints

- Physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not found
  (VID=0x03FD)", P12 unwired, no relay/remote cold-POR gate.
- Variant A (live-capture fixture archive) is **out of scope** for W454.
- Variant B (master-merge of `701d79b3b`) is **insufficient** for the 7 residual
  failures and is rejected as the default.
- The `gen-verilog` tuple/array backend gaps require a dedicated compiler wave
  (targeted for W455 or later).
- All formal work must be board-less and deterministic.

---

## 3. Weak points investigated

1. **Tuple return types are not parsed.** `parse_fn_decl` recognizes return
   types as `Ident`, `LBracket`, `KwVoid`, or `Star`, but not as `LParen`
   tuple types. `-> (u32, u32, u32)` leaves `extra_return_type` empty.
2. **`let (a, b, c) = ...` is not parsed as destructuring.** `parse_local_decl`
   expects an `Ident` after `const`/`var`/`let`. `LParen` leaves `decl.name`
   empty and the tuple expression is treated as the initializer, producing
   invalid Verilog like `reg [31:0] ;`.
3. **Tuple expressions `(a, b, c)` have no AST node.** The parser treats
   parentheses as a single-expression wrapper; commas are not handled.
4. **Module-level `const [N]T = [N]T{...}` emits raw t27 syntax into
   Verilog.** `gen_verilog_const` detects `ExprArrayLiteral`/`ExprStructLit` as
   unsupported aggregate and emits `0 /* TODO ... */`, but the literal is
   parsed as a single `ExprLiteral` whose value string is `[4]u16{...}`, which
   `gen_verilog_expr` writes verbatim.
5. **Master commit `701d79b3b` does not close the above gaps.** Comparing
   `master` and `wave-loop-454` shows master lacks the `let` keyword alias,
   lacks `ExprCast` lowering in Zig/C, and uses narrower keyword escaping.
6. **Competitor landscape is static.** Sparkle/Verilean remains the only fresh
   Lean-native HDL signal; CIRCT `firtool-1.152.0` (2026-07-04) and Clash
   1.11.0 candidate are unchanged since W453.

---

## 4. Deliverables and decomposition

### 4.1 VCCINT high adversarial witness
**Owner:** formal boot-evidence ring.  
**File:** `proofs/lean4/Trinity/TernaryFPGABoot.lean`

1. Define `OUTSIDE_VCCINT_HIGH_W454_OPERATING_POINT` at 25 °C, 1200 mV VCCINT,
   1800 mV VCCAUX (above the documented 1100 mV envelope maximum).
2. Prove `outside_vccint_high_w454_operating_point_not_within_envelope`.
3. Prove `cclk_variant_and_xadc_envelope_check_outside_vccint_high_false`:
   the dashboard gate rejects high VCCINT.
4. Add a Rust unit test that asserts the computable gate returns `false` for
   this point.

### 4.2 Duty-cycle asymmetry theorem
**Owner:** formal boot-evidence ring.  
**File:** `proofs/lean4/Trinity/TernaryFPGABoot.lean`

1. Define a predicate `cclk_split_satisfies_flash_spec` that takes `period_ns`,
   `low_ns`, and `high_ns` and checks the N25Q128 low/high time requirements
   independently.
2. Prove that for any `period_ns` equal to `cclk_period_ns oscfsel`, any
   `low_ns`/`high_ns` split with `low_ns + high_ns = period_ns`, `low_ns`
   above the minimum low time, and `high_ns` above the minimum high time,
   the transaction is compliant. Use the existing worst-case PVT bridge.
3. Add a concrete computable-gate counterpart.

### 4.3 Bounded jitter / timing perturbation theorem
**Owner:** formal boot-evidence ring.  
**File:** `proofs/lean4/Trinity/TernaryFPGABoot.lean`

1. Define a bounded perturbation predicate: measured raw low/high times are
   within a small integer-ns tolerance of the ideal values.
2. Prove that if the ideal values satisfy the flash spec and the perturbation
   is within the spec slack, the perturbed values also satisfy the spec.
3. Keep the bound symbolic or use the documented envelope slack so the theorem
   remains falsifiable.

### 4.4 Competitor and defect report refresh
**Owner:** documentation ring.  
**Files:** `docs/reports/T27_VS_FORMAL_HDL_2026.md`,
`docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

1. Add W454 boundary paragraph to the competitor report noting no new
   Lean-native ternary-FPGA competitor.
2. Update the gen-verilog defect tracker with the W454 triage decision:
   master-merge insufficient, dedicated backend wave required for tuple/array
   lowering.

### 4.5 Optional safe gen-verilog sub-fix
**Owner:** compiler ring.  
**File:** `bootstrap/src/compiler.rs`

If a zero-regression fix is identified during implementation:
1. Add a regression scratch spec under `specs/scratch/`.
2. Fix the narrow issue (e.g., detect and skip empty `StmtLocal` names in
   `gen_verilog_var`, or improve array-literal initializer emission).
3. Run the full suite to confirm no regressions.

If no zero-regression fix is found, this sub-task is skipped and documented.

---

## 5. Acceptance criteria

- `cd proofs/lean4 && lake build Trinity.TernaryFPGABoot`: success.
- `cargo test -p tri --bin tri fpga::`: PASS with new adversarial/robustness
  tests.
- `./scripts/tri test --json <path>`: ACCEPTABLE.
  - 576/576 non-smoke PASS.
  - 7 baseline gen-verilog failures remain the documented baseline.
  - FPGA smoke gate: PASS, `passed: true`, `acceptable: true`.
- New adversarial/robustness theorems build in `Trinity.TernaryFPGABoot` and are
  covered by computable gate or Rust unit tests.
- W454 report, evidence, and W455 cooperation variants are written.

---

## 6. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Lean proof exceeds build time budget | low | Keep theorems symbolic and reuse existing envelope bridge lemmas. |
| Optional gen-verilog sub-fix causes regression | medium | Only apply if isolated regression test passes and full suite is acceptable. |
| Competitor report lacks fresh signals | high | Document the static landscape honestly; no fabricated signals. |

---

## 7. Next-wave handoff

Wave Loop 455 should target the deep `gen-verilog` backend gaps identified in
this plan. See `docs/reports/FPGA_LOOP_COOPERATION_W455_2026-07-01.md` for
three candidate variants.

---

*φ² + φ⁻² = 3 | TRINITY*

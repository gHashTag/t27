# Wave Loop 368 — Cooperation Variants for W369

**Date:** 2026-07-01  
**Prepared for:** Wave Loop 369 planning (#1256 continuation / #1257)  

---

## Recommended Variant: **B** (Formal + Board Flash + One Safe Gen-Verilog Sub-fix)

---

## Variant A — Formal-Only Safe Ramp

**Scope:** extend the proof lattice without touching hardware or the compiler backend.

**W369 targets:**
- 220 generic `∀` theorems.
- 45-variable plus accumulation.
- 44-variable minus accumulation lattice.
- Depth-22 identity cancellation (vigintiduple / duovigintuple).
- Zero-weight duodecuple closure (12 zero-weight MACs around a plus-weight MAC).
- +54 tests, +27 invariants across 27 IGLA specs.
- Regenerate all 27 seals; maintain 547/547 conformance PASS.

**Pros:**
- Zero risk of compiler regression or hardware dependency.
- Fastest to land.
- Continues to widen the generic-quantified-proof moat against Sparkle HDL / Verilean.

**Cons:**
- Does not address the silicon-evidence gap.
- Does not harden the Verilog backend.
- Competitors with real tape-outs / FPGAs can claim "but where is the chip?"

**Best for:** weeks when hardware is unavailable and compiler bandwidth is limited.

---

## Variant B — Formal + Board Flash Retry + One Safe Gen-Verilog Sub-fix **(RECOMMENDED)**

**Scope:** keep the formal engine running while chipping away at the two largest weak points: physical evidence and Verilog output quality.

**W369 targets:**
- 220 generic `∀` theorems (same as Variant A).
- 45-variable plus / 44-variable minus accumulation.
- Depth-22 cancellation.
- Zero-weight duodecuple closure.
- +54 tests, +27 invariants; 547/547 PASS.
- **Retry `dlc10 idcode / sram / flash`** on QMTech Wukong V1 with the existing `ternary_mac_demo_top.bit`.
- **One safe `gen-verilog` sub-fix:** extend scalar hex-width padding to binary (`0b`) literals in `const`/`var`/`let`/`return`, or add a `yosys read_verilog` CI smoke gate to catch future lowering regressions automatically.

**Pros:**
- Balances proof depth, backend quality, and silicon evidence.
- Each sub-fix is narrow and regression-testable.
- Keeps the project credible on both formal and implementation axes.

**Cons:**
- Board flash may fail again, producing no new evidence but costing a small amount of time.
- Requires careful sub-fix selection to avoid seal churn.

**Best for:** normal wave cadence when both formal and hardware tracks can advance in parallel.

---

## Variant C — Formal + RTL-to-Lean Traceability Prototype + Board Flash

**Scope:** attempt a deeper, more strategic deliverable: a prototype that links generated Verilog back to the Lean 4 proof lattice.

**W369 targets:**
- 220 generic `∀` theorems (same as Variants A/B).
- 45-variable plus / 44-variable minus accumulation.
- Depth-22 cancellation.
- Zero-weight duodecuple closure.
- **Prototype `scripts/verilog_to_lean.py`** that parses `t27c gen-verilog` output (module signature, localparams, registers, `always @(posedge clk)`) and emits Lean 4 **traceability lemmas** asserting structural correspondence between the generated RTL and the ternary MAC spec.
- Document a phased roadmap for full RTL-to-Lean equivalence checking.
- Retry board flash.

**Pros:**
- Builds a differentiated moat that Sparkle HDL (SystemVerilog-only generation) and ternary accelerators (no proof) cannot match.
- Positions t27 as a "proof-carrying codegen" framework.
- High-impact publishable artifact.

**Cons:**
- Traceability prototype is exploratory; may not produce a fully working pipeline in one wave.
- Diverts effort from pure proof-depth gains.
- Higher risk of incomplete deliverables.

**Best for:** when there is enough bandwidth to run a high-risk/high-reward research track in parallel with the core wave.

---

## Decision Matrix

| Criterion | A | B | C |
|---|---|---|---|
| Proof depth gain | High | High | High |
| Regression risk | Low | Low | Medium |
| Hardware evidence | None | Retry | Retry |
| Backend hardening | None | 1 sub-fix | Traceability prototype |
| Publication value | Medium | High | Very High |
| Land confidence | Very High | High | Medium |

## Recommendation

Choose **Variant B** for W369. It preserves the 220 generic `∀` target, retries the board flash, and lands one additional safe `gen-verilog` sub-fix — the same proven rhythm that delivered W367 and W368 successfully. Reserve Variant C for W370 or spin it out as a dedicated research issue if bandwidth allows.

---

phi^2 + 1/phi^2 = 3 | TRINITY

# Current Issue: Wave Loop 381

**Issue:** #1272
**Branch:** `trinity-rust-rings`
**Basis:** W380 close-out report and W380 cooperation variants (`docs/reports/WAVE_LOOP_380_COOPERATION.md`)

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **115 waves**, push the Lean 4 generic ∀ lattice to **268**, and finish the **slot-aware nested tuple-return call lowering** work in the Verilog backend.

## Open strategic context

- Sparkle HDL / Verilean remains the only credible formal competitor (~60 theorems, 0 generic ∀); their Functional Festival 2026 talk is on July 11, 2026.
- Bitstream is ready (`fpga/verilog/ternary_mac_demo_top.bit`) but board flash blocked by missing DLC10 cable.
- W380 delivered tuple-return generation scaffolding (parser, packed result registers, tuple literals, callee-aware destructuring). Remaining gap is slot-aware lowering so tuple-returning functions can pass elements forward without manual destructuring.
- Datapath RAM/array lowering (#1258) remains tracked but is too broad for one wave.

## Candidate variants (from W380 cooperation)

- **Variant A** — proof-only push to 268 generic ∀, defer backend.
- **Variant B (recommended)** — proof push + close slot-aware nested tuple-return call lowering, keep smoke gate green.
- **Variant C** — backend-first pause, attack #1258.

See `.claude/plans/wave-loop-381.md` for selected variant and decomposed plan.

---

*phi² + 1/phi² = 3 | TRINITY*

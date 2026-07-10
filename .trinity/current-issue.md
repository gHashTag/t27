# Wave Loop 484 — next-wave selection

**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W483 made imported struct-return calls functional in gen-verilog:

- Imported zero-argument constructors whose body is a single scalar struct
  literal are now inlined as packed concatenations at the call site.
- Locals initialized from such calls are declared as packed `reg [W-1:0]` and
  field accesses are lowered via the existing packed-slicing path.
- The Icarus smoke gate remains at **0 documented baseline failures**.

Verification at W483 close-out:

- 656 / 656 non-smoke PASS.
- 136 / 136 yosys smoke PASS.
- 136 / 136 Icarus smoke PASS, **0 documented baseline failures**.
- 656 / 656 seal matches.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.

## Goal

Select and execute one of the W484 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W484_2026-07-07.md`.

## Default direction

**Variant B (default):** continue turning `UNSUPPORTED_ICARUS` placeholders into
real logic for the next most common classes:

- dynamic `.len()` / `.contains()` on fixed-size arrays and string literals,
- host-side recursive helper shadowing in IGLA specs,
- module-scope wildcard `_` bindings.

## Alternative directions

- **Variant A:** formalize the Icarus-supported t27 subset as a Lean 4 predicate,
  with a lowering-preservation lemma and `tri test` wiring.
- **Variant C:** FPGA live cold-POR / SPI flash boot evidence if the QMTech
  Wukong XC7A100T and DLC10 cable are available.

## Issue Gate

- Branch: `wave-loop-484` (to create from `wave-loop-483`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W483 close-out: `docs/reports/WAVE_LOOP_483_CLOSEOUT.md`
- W484 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W484_2026-07-07.md`

# Wave Loop 482 — next-wave selection

**Date:** 2026-07-10  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W481 closed the remaining Icarus baseline:

- `f32` cast target preservation,
- `field_access_base_is_unresolved` helper,
- sized zero placeholders for unresolved field accesses,
- local/unsupported-call-result tracking,
- witness specs `specs/scratch/w481_struct_supplier.t27` and `specs/scratch/w481_icarus_aos_param_and_imported_struct.t27`,
- 652/652 non-smoke PASS, 0 yosys smoke failures, **0 documented Icarus baseline failures**, 0 seal mismatches after reseal, `cargo test -p t27c --bin t27c` 1525/0/2.

## Goal

Select and execute one of the W482 cooperation variants documented in `docs/reports/FPGA_LOOP_COOPERATION_W482_2026-07-10.md`.

## Default direction

**Variant B (default):** make the W481 placeholders functional for the most common AOS / imported-struct / struct-return classes:

- read imported struct layouts from seals so imported scalar struct parameters destructure into real wires,
- generalize same-file AOS parameter lowering to handle variable-index element field access,
- declare packed locals for same-file struct-return results so field slices read real values.

## Alternative directions

- **Variant A:** formalize the Icarus-supported t27 subset as a Lean 4 predicate, with a lowering-preservation lemma and `tri test` wiring.
- **Variant C:** FPGA live cold-POR / SPI flash boot evidence if the QMTech Wukong XC7A100T and DLC10 cable are available.

## Issue Gate

- Branch: `wave-loop-482` (to create from `wave-loop-481`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke acceptable (no new regressions outside baseline), seals green, `cargo test -p t27c --bin t27c` green.

## References

- W481 close-out: `docs/reports/WAVE_LOOP_481_CLOSEOUT.md`
- W482 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W482_2026-07-10.md`

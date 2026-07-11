# Wave Loop 491 — Formalize the Icarus-lowerable subset in Lean 4

**Issue:** #1461 (to create)  
**Branch:** `wave-loop-491`  
**Variant:** A (default) from `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Problem statement

After W490 the t27 → Icarus path is functionally complete for the current spec
set, but the contract that keeps it complete is implicit in
`bootstrap/src/compiler.rs`. The rules that decide lowerability are encoded in
`fn_body_has_unlowerable_construct`, `compute_host_only_functions`, and ad-hoc
checks inside the Verilog emitter. A future frontend feature can therefore
silently drift past what the backend can emit.

## Goal

Lock the lowerability contract into a machine-checkable form:

1. Define a simplified t27 AST and an `IsIcarusLowerable` predicate in the
   existing `proofs/lean4/` formalization.
2. Add a Rust `t27c icarus-lowerable --json` classifier that exports the same
   verdict the emitter uses internally.
3. Add a `--icarus-lowerable` suite gate that ensures no spec passes Icarus
   smoke unless the classifier agrees it is lowerable.
4. Prove representative lemmas in Lean 4 for the four W490 lowerability classes.

## Acceptance criteria

- `lake build Trinity.IcarusLowerable.*` succeeds.
- `cargo build --release` succeeds.
- `cargo test -p t27c --bin t27c` passes (1525 / 0 / 2).
- `./scripts/tri test --fast` reports 687/687 non-smoke PASS, 167/167 yosys smoke
  PASS, 166/166 Icarus smoke PASS, 0 `UNSUPPORTED_ICARUS` placeholders.
- `./target/release/t27c suite --repo-root . --fast --icarus-lowerable` reports
  zero disagreements between smoke results and lowerability verdicts.
- New `specs/scratch/w491_*.t27` witnesses exercise the lowerability boundary.
- Close-out report and W492 cooperation variants are written.

## Research backing

- Sparkle / Verilean (Lean 4 HDL compiler with Icarus validation).
- CktFormalizer (arXiv:2605.07782) — 95–100% backend realizability via
  synthesizable-subset discipline.
- Lööw & Myreen, HOL4 proof-producing Verilog translator.
- FIRRTL spec and SystemVerilog ABI.
- Recent ternary RTL-to-netlist flows (Park et al. IEEE Access 2025; Li et al.
  CJE 2025).

See `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` for the full research
snapshot.

---

*φ² + φ⁻² = 3 | TRINITY*

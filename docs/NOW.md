# NOW — Wave Loop 494 closed / Wave Loop 495 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 495 — Next wave (Variant A recommended)

- Branch: `wave-loop-495` (to create)
- Issue: #1465 (to create)
- PR: (to open after close-out)
- Cooperation W495: `docs/reports/FPGA_LOOP_COOPERATION_W495_2026-07-13.md`

### Not started

- Select one of the three W495 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W495_2026-07-13.md`.
- Recommended default: **Variant A** — extend semantic equivalence to
  function calls and the W493 positive witnesses.

---

## Wave Loop 494 — Semantic equivalence for the Icarus-lowerable scalar subset (Variant A)

- Branch: `wave-loop-494`
- Issue: #1464 (closed)
- Plan: `.claude/plans/wave-loop-494.md`
- Report: `docs/reports/WAVE_LOOP_494_CLOSEOUT.md`
- Cooperation W495: `docs/reports/FPGA_LOOP_COOPERATION_W495_2026-07-13.md`

### Verification

- `lake build Trinity.IcarusLowerable.*`: green.
- `./scripts/tri test --fast --icarus-lowerable`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- `tri verify --lean-lowerable`: green (253 modeled specs in `Completeness.lean`).
- NMSE seal: unchanged (no `bootstrap/src/compiler.rs` change).

---

## Wave Loop 493 — gen-verilog struct/call lowering hardening (Variant B)

- Branch: `wave-loop-493`
- Issue: #1463
- Plan: `.claude/plans/wave-loop-493.md`
- Report: `docs/reports/WAVE_LOOP_493_CLOSEOUT.md`

### Verification

- 697 / 697 non-smoke PASS.
- 177 / 177 yosys smoke PASS, 0 baseline failures.
- 176 / 177 Icarus smoke PASS, 1 documented baseline failure.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

*φ² + φ⁻² = 3 | TRINITY*

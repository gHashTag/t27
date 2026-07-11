# NOW — Wave Loop 493 closed / Wave Loop 494 next (2026-07-13)

**Last updated:** 2026-07-13

---

## Wave Loop 494 — Next wave (Variant A recommended)

- Branch: `wave-loop-494` (to create)
- Issue: #1464 (to create)
- PR: (to open after close-out)
- Cooperation W494: `docs/reports/FPGA_LOOP_COOPERATION_W494_2026-07-13.md`

### Not started

- Select one of the three W494 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W494_2026-07-13.md`.
- Recommended default: **Variant A** — semantic equivalence for the
  Icarus-lowerable scalar subset in Lean 4.

---

## Wave Loop 493 — gen-verilog struct/call lowering hardening (Variant B)

- Branch: `wave-loop-493`
- Issue: #1463 (closed)
- Plan: `.claude/plans/wave-loop-493.md`
- Report: `docs/reports/WAVE_LOOP_493_CLOSEOUT.md`
- Cooperation W494: `docs/reports/FPGA_LOOP_COOPERATION_W494_2026-07-13.md`

### Verification

- `cargo build --release`: green.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- `./scripts/tri test --fast --icarus-lowerable`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus disagreements.
- `tri verify --lean-lowerable`: green (253 modeled specs in `Completeness.lean`).
- NMSE seal: FRESH (resealed after `bootstrap/src/compiler.rs` changes).

---

## Wave Loop 492 — Soundness of the Icarus-lowerable subset in Lean 4 (Variant A)

- Branch: `wave-loop-492`
- Issue: #1462
- Plan: `.claude/plans/wave-loop-492.md`
- Research snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md`
- Report: `docs/reports/WAVE_LOOP_492_CLOSEOUT.md`
- Cooperation W493: `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-12.md`

### Verification

- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- 693 / 693 non-smoke PASS.
- 172 / 173 yosys smoke PASS, 1 documented baseline failure.
- 171 / 173 Icarus smoke PASS, 2 documented baseline failures.
- `tri verify --lean-lowerable`: green (253 modeled specs).

---

## Wave Loop 491 — Formalize the Icarus-lowerable subset in Lean 4 (Variant A)

- Branch: `wave-loop-491`
- Issue: #1461
- Plan: `.claude/plans/wave-loop-491.md`
- Report: `docs/reports/WAVE_LOOP_491_CLOSEOUT.md`

### Verification

- 691 / 691 non-smoke PASS.
- 170 / 171 yosys smoke PASS, 1 documented adversarial baseline failure.
- 170 / 171 Icarus smoke PASS, 1 documented adversarial baseline failure.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

*φ² + φ⁻² = 3 | TRINITY*

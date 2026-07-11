# NOW — Wave Loop 492 closed / Wave Loop 493 next (2026-07-12)

**Last updated:** 2026-07-12

---

## Wave Loop 493 — Next wave (to be selected from cooperation plan)

- Branch: `wave-loop-493`
- Issue: #1463 (to create)
- PR: (to open after close-out)
- Cooperation W493: `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-12.md`

### Not started

- Select one of the three W493 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-12.md`.

---

## Wave Loop 492 — Soundness of the Icarus-lowerable subset in Lean 4 (Variant A)

- Branch: `wave-loop-492`
- Issue: #1462
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-492.md`
- Research snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md`
- Report: `docs/reports/WAVE_LOOP_492_CLOSEOUT.md`
- Cooperation W493: `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-12.md`

### Verification

- `cargo build --release`: green.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- `./target/release/t27c suite --repo-root . --fast`:
  - 693 / 693 non-smoke PASS (681 base + 6 W490 + 4 W491 + 2 W492 scratch witnesses).
  - 172 / 173 yosys smoke PASS, 1 documented baseline failure
    (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`).
  - 171 / 173 Icarus smoke PASS, 2 documented baseline failures
    (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`,
    `specs/scratch/w492_predicate_rejects_nested_return_field.t27`).
  - 693 / 693 seal matches.
  - 0 `UNSUPPORTED_ICARUS` placeholders outside the documented adversarial witnesses.
- `./target/release/t27c suite --repo-root . --fast --icarus-lowerable`:
  - 171 lowerable, 2 not_lowerable, 0 disagreements.
- `tri verify --lean-lowerable`: green (253 modeled specs proved lowerable).
- `lake build Trinity.IcarusLowerable.Verilog Trinity.IcarusLowerable.Emitter
  Trinity.IcarusLowerable.Soundness`: green.
- NMSE seal: FRESH (resealed after `bootstrap/src/compiler.rs` changes).

---

## Wave Loop 491 — Formalize the Icarus-lowerable subset in Lean 4 (Variant A)

- Branch: `wave-loop-491`
- Issue: #1461
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-491.md`
- Research snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md`
- Report: `docs/reports/WAVE_LOOP_491_CLOSEOUT.md`
- Cooperation W492: `docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md`

### Verification

- 691 / 691 non-smoke PASS (681 base + 6 W490 + 4 W491 scratch witnesses).
- 170 / 171 yosys smoke PASS, 1 documented adversarial baseline failure
  (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`).
- 170 / 171 Icarus smoke PASS, 1 documented adversarial baseline failure
  (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`).
- 691 / 691 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- `./target/release/t27c suite --repo-root . --fast --icarus-lowerable`:
  170 lowerable, 1 not_lowerable, 0 disagreements.
- `lake build Trinity.IcarusLowerable.*`: green.
- NMSE seal: FRESH.

---

*φ² + φ⁻² = 3 | TRINITY*

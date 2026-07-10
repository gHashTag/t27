# NOW — Wave Loop 488 close-out / Wave Loop 489 next (2026-07-07)

**Last updated:** 2026-07-07

## Wave Loop 489 — Next wave (to be selected from cooperation plan)

- Branch: `wave-loop-489` (to create from `wave-loop-488`)
- Issue: #1459 (to be opened)
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-489.md` (to be written at W489 start)
- Cooperation W490: (to be written at W489 close-out)

### Not started

- Select one of the three W489 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md`.

---

## Wave Loop 488 — Backend hardening: wildcard array-of-struct aliases with array-typed fields (Variant B, scoped)

- Branch: `wave-loop-488`
- Issue: #1458
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-488.md`
- Report: `docs/reports/WAVE_LOOP_488_CLOSEOUT.md`
- Cooperation W489: `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md`

### Verification

- 673 / 673 non-smoke PASS.
- 153 / 153 yosys smoke PASS, 0 failures.
- 153 / 153 Icarus smoke PASS, 0 documented baseline failures.
- 673 / 673 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.
- NMSE reseal: FROZEN_HASH and manifest refreshed.

### Note

The original Variant B proposal also targeted colon-style struct-literal
separators and non-synthesizable struct-field policy. Those sub-fixes were
attempted, exposed latent duplicate-declaration and keyword-name issues in
`igla/` specs, and were deferred to W489.

---

*φ² + φ⁻² = 3 | TRINITY*

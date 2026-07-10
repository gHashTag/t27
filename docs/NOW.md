# NOW — Wave Loop 489 close-out / Wave Loop 490 next (2026-07-07)

**Last updated:** 2026-07-07

## Wave Loop 490 — Next wave (to be selected from cooperation plan)

- Branch: `wave-loop-490` (to create from `wave-loop-489`)
- Issue: #1460 (to be opened)
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-490.md` (to be written at W490 start)
- Cooperation W491: (to be written at W490 close-out)

### Not started

- Select one of the three W490 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md`.

---

## Wave Loop 489 — Backend hardening: colon struct-literals, struct-local deduplication, imported constructor inlining (Variant B)

- Branch: `wave-loop-489`
- Issue: #1459
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-489.md`
- Report: `docs/reports/WAVE_LOOP_489_CLOSEOUT.md`
- Cooperation W490: `docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md`

### Verification

- 681 / 681 non-smoke PASS.
- 161 / 161 yosys smoke PASS, 0 failures.
- 161 / 161 Icarus smoke PASS, 0 documented baseline failures.
- 681 / 681 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.
- NMSE reseal: FROZEN_HASH and manifests refreshed.

### Note

W489 completed the colon struct-literal and struct-local lowering work that W488
prototyped and rolled back. The remaining expression-context gaps (bare
imported constructor calls with array-typed fields, module-scope AOS constants
with array-typed fields, and host-only enum/string helper classification) are
documented as Variant B candidates for W490.

---

*φ² + φ⁻² = 3 | TRINITY*

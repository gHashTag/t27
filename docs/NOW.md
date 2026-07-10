# NOW — Wave Loop 490 close-out / Wave Loop 491 next (2026-07-07)

**Last updated:** 2026-07-07

## Wave Loop 491 — Next wave (to be selected from cooperation plan)

- Branch: `wave-loop-491` (to create from `wave-loop-490`)
- Issue: #1461 (to be opened)
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-491.md` (to be written at W491 start)
- Cooperation W492: (to be written at W491 close-out)

### Not started

- Select one of the three W491 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md`.

---

## Wave Loop 490 — Backend hardening: scalar struct-return array-field access, imported constructor expression context, module-scope AOS constants with array-typed fields, host-only enum/string helper classification (Variant B)

- Branch: `wave-loop-490`
- Issue: #1460
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-490.md`
- Report: `docs/reports/WAVE_LOOP_490_CLOSEOUT.md`
- Cooperation W491: `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md`

### Verification

- 687 / 687 non-smoke PASS (681 base + 6 new W490 scratch witnesses).
- 167 / 167 yosys smoke PASS, 0 failures.
- 166 / 166 Icarus smoke PASS, 0 documented baseline failures.
- 687 / 687 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Zero `UNSUPPORTED_ICARUS` placeholders across all specs.
- NMSE reseal: FROZEN_HASH and manifests refreshed.

### Note

W490 closed the expression-context gaps left by W489: indexed field access on
scalar struct-return calls, imported constructor calls used directly in
expressions with array-typed fields, and host-only classification for string/enum
helpers. The default next direction is Variant A: formalize the Icarus-lowerable
subset in Lean 4.

---

*φ² + φ⁻² = 3 | TRINITY*

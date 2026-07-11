# NOW — Wave Loop 491 in progress / Wave Loop 492 next (2026-07-11)

**Last updated:** 2026-07-11

---

## Wave Loop 491 — Formalize the Icarus-lowerable subset in Lean 4 (Variant A)

- Branch: `wave-loop-491`
- Issue: #1461 (to create)
- PR: (to open after close-out)
- Plan: `.claude/plans/wave-loop-491.md`
- Research snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md`
- Cooperation W492: (to be written at W491 close-out)

### In progress

- `proofs/lean4/Trinity/IcarusLowerable/{Ast,Predicate,Lemmas}.lean` — simplified
  t27 AST, `IsIcarusLowerable` predicate, and representative lowerability lemmas.
- `t27c icarus-lowerable --json` classifier + `--icarus-lowerable` suite gate.
- `specs/scratch/w491_*.t27` adversarial boundary witnesses.

### Target gate

- 687 / 687 non-smoke PASS (681 base + 6 W490 scratch witnesses + new W491 witnesses).
- 167 / 167 yosys smoke PASS, 0 failures.
- 166 / 166 Icarus smoke PASS, 0 documented baseline failures.
- 687 / 687 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Zero `UNSUPPORTED_ICARUS` placeholders.
- `t27c suite --repo-root . --fast --icarus-lowerable`: zero disagreements.

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

---

*φ² + φ⁻² = 3 | TRINITY*

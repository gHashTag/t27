---
description: Standing Wave Loop charter for t27 — investigate weak points, research papers, plan, implement, report, and propose next-wave cooperation variants.
parameters:
  - name: wave
    type: string
    description: Wave number (e.g. "526")
  - name: issue
    type: string
    description: GitHub issue number for the wave
---

# t27 Wave Loop Skill

This skill encodes the standing Wave Loop charter repeated across t27 sessions:

> investigate weak points, research relevant scientific literature, create a
> decomposed plan, implement the recommended variant, write a closeout report,
> propose three cooperation variants for the next Wave Loop, and save skills
> and experience at the end.

Procedure:

1. **Investigate weak points** — audit the current branch, recent test
   baselines, and unlanded process-debt needles.
2. **Research scientific literature** — find 2–4 papers or canonical models
   relevant to the needle (e.g. Vericert, CompCert, Vitis HLS AoS/SoA rules,
   Roofline).
3. **Create a decomposed plan** — write `.claude/plans/wave-loop-{N}.md` with
   three variants (A recommended, B implementation-heavy, C process/tooling).
4. **Implement the recommended variant** — make the smallest reviewable diff that
   advances the needle, update `FROZEN_HASH` if `bootstrap/src/compiler.rs`
   changes, and run the relevant validation gates.
5. **Write the closeout report** — `docs/reports/WAVE_LOOP_{N}_CLOSEOUT.md`.
6. **Write cooperation variants** —
   `docs/reports/FPGA_LOOP_COOPERATION_W{N+1}_YYYY-MM-DD.md`.
7. **Update issue tracking** — `.trinity/current-issue.md` for the next wave.
8. **Save learnings** — append to `.trinity/experience.md` and persistent memory.
9. **Save/update this skill** — keep the charter encoded in
   `.claude/skills/t27-wave-loop.md`.

## Invariants

- Follow L1 TRACEABILITY: every commit must reference an issue with
  `Closes #N`, `Fixes #N`, `Refs #N`, etc.
- Never hand-edit files under `gen/`; change specs and regenerate.
- Update `bootstrap/stage0/FROZEN_HASH` whenever `bootstrap/src/compiler.rs`
  is modified.
- Prefer a clear diagnostic over silently passing smoke tests with broken
  generated code.

## Phase completion marker

When a PHI LOOP phase is complete, include:

```
Phase complete: [phase name]
→ Phase [next phase number]: [next phase name]
```

## Worked example — Wave Loop 529

Wave Loop 529 formalized the W528 packed-vector 2-D AoS cross-boundary lowering:

- Restored missing `Trinity.IcarusLowerable` source modules from git history.
- Added four positive witnesses in `Lemmas.lean`.
- Proved value preservation in `Soundness.lean` via the generic equivalence theorems.
- Created and sealed four matching scratch specs.
- Validation: `lake build Trinity.IcarusLowerable.Soundness` green with 0 sorry,
  `cargo test -p t27c --bin t27c` 1494/0/2, `./scripts/tri test` 0 seal mismatches.

Key learning: when formal source modules are missing from the worktree, check the
commit history before re-implementing the shallow model.

---

*φ² + φ⁻² = 3 | TRINITY*

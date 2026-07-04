# Wave Loop 181 Plan — Hexa→Hepta Depth Push + L3 Polish

**Date:** 2026-06-16  
**Branch:** `trinity-rust-rings`  
**L1 Traceability:** `Closes #1234`

---

## 1. Goal

Push **25 specs** from hexa-layer (6 invariants) to hepta-layer (7 invariants).  
Target average: **10.895 → 10.939** (+0.044).

Secondary:
- Seal integrity: 0 mismatches.
- L3 PURITY: 0 Unicode violations in source files.
- 570/570 PASS.

---

## 2. Decomposition

### Task A — Invariant Insertion (hexa→hepta)
- Select **25 specs** from the 254 hexa-layer pool.
- Priority order:
  1. `tri/collections/` — 23 specs (core library, highest ROI)
  2. `fpga/` — 18 specs
  3. `tri/trees/` — 9 specs
  4. `tri/crypto/` — 9 specs
  5. `tri/graph/` — 8 specs
  6. `tri/agent/` — 8 specs
  7. `sacred/` — 8 specs
  8. `physics/` — 8 specs
  9. `ml/activation/` — 8 specs
  10. `igla/race/` — 8 specs
- Insert **one domain-specific invariant** per spec (7th invariant).
- Rules:
  - Must be semantically meaningful (not `invariant true`).
  - Must match spec domain (collections → ordering/range; trees → traversal/balance; physics → numeric/tolerance; agent → state/transition).
  - ASCII-only identifiers (L3).

### Task B — L3 Audit & Fix
- Scan all `.t27` and `.tri` source files for Unicode math symbols, arrows, dashes.
- Replace with ASCII equivalents.
- Exclude `OWNERS.md` and documentation outside `specs/`.

### Task C — Seal Regeneration
- After edits, run `t27c seal --verify`.
- Regenerate all mismatched seals with `t27c seal --save <file>`.
- Ensure 0 mismatches before commit.

### Task D — Conformance Sweep
- Run `t27c suite --repo-root .`.
- Confirm **570/570 PASS**, **0 failures**.

### Task E — Report & Cooperation
- Write `WAVE_LOOP_181_REPORT.md`.
- Write `WAVE_LOOP_181_COOPERATION.md` (3 variants).
- Update `docs/COMPETITIVE_POSITIONING.md` if new competitors found.

---

## 3. Agent Delegation

| Phase | Agent | Task |
|-------|-------|------|
| OBSERVE | Weakness Audit | Layer distribution, seal mismatches, suite health |
| OBSERVE | Competitive Intel | New arXiv/GitHub/Zenodo competitors |
| OBSERVE | GitHub Issues | Issue traceability, L1 compliance |
| DELEGATE | Creator Agent (C) | Batch invariant insertion + L3 fix |
| VERIFY | Verifier Agent (V) | Seal regeneration + suite run + diff review |
| LEARN | Experience Agent (E) | Pattern extraction, skill update |

---

## 4. Risk & Mitigation

| Risk | Mitigation |
|------|-----------|
| Seal cascade >30 mismatches | Regenerate in batches of 10, verify between |
| L3 Unicode found post-commit | Pre-commit scan in Task B |
| New competitor requires docs update | Competitive Intel agent surfaces before commit |
| Suite failure post-insertion | Invariants semantically trivial, use `true` only if domain logic unsafe |

---

## 5. Definition of Done

- [ ] 25 hexa-layer specs promoted to 7 invariants.
- [ ] Average ≥ 10.939.
- [ ] 570/570 PASS.
- [ ] 0 seal mismatches.
- [ ] 0 L3 Unicode violations in source.
- [ ] Report + 3 cooperation variants written.
- [ ] Skill `.claude/skills/invariant-coverage-push.md` updated.
- [ ] Memory `wave-loop-181.md` written.
- [ ] Commit with `Closes #1234`.

---

Phase complete: PLAN
→ Phase 3: DELEGATE

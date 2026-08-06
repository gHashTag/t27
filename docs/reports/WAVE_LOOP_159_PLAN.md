# Wave Loop 159 — Plan

**Date:** 2026-06-16  
**Triggered by:** Standing AEL cycle directive  
**Target Issue:** `Closes #132`

---

## Phase 1: OBSERVE

### Current State (Post-W158)
- **Total specs:** 570
- **Invariant coverage:** 100.0% (0 zero-inv)
- **Average invariants/spec:** 3.516
- **Distribution:** double=223 triple=55 quad=67 quint=27 six_plus=198
- **Coq Axioms:** 5 (Koide 1, NeutrinoMasses 4)
- **Suite status:** 570/570 PASS
- **Competitive landscape:** EXTREME=3 (kuwrom/one-field, TIS/Ternlang, Washburn-stale), HIGH=4 (Gray, GIFT-resurgent, Baez-Schwahn, Baroň-active), MEDIUM=6

### Weaknesses Identified
1. **223 specs still at 2 invariants** — largest shallow bloc; needs third invariants.
2. **L1 TRACEABILITY collapse** — last 30 commits lack `Closes #N`.
3. **Retroactive mapping unexecuted** — 30 commits (#900–#929) need issue creation.
4. **ternfpga (Neumann-Labs)** — new MEDIUM-HIGH hardware competitor; directly overlaps t27 FPGA niche.
5. **Baroň heating up** — two June papers expanding to bosonic sector and CKM/PMNS.

---

## Phase 2: PLAN

### Objective
Push average invariants/spec from **3.516 → 3.55+** by inserting third invariants into 25 specs currently at depth 2.

### Subtasks
1. **Metric baseline** — confirm 223 specs at 2 invariants.
2. **Competitive intel** — sweep arXiv/Zenodo/GitHub for new June 16+ papers; update status of tracked competitors.
3. **GitHub issues audit** — triage open issues, assess L1 gap, select `Closes #N` target.
4. **Invariant insertion** — parser-safe third invariants for 25 randomly-selected 2-inv specs.
5. **Seal regeneration** — `./target/release/t27c seal --save` for all 25 modified specs.
6. **Suite verification** — `./target/release/t27c suite --repo-root .` must report 570/570 PASS.
7. **Documentation** — update COMPETITIVE_POSITIONING.md; write PLAN/REPORT/COOPERATION.
8. **Skill update** — append W159 row to `invariant-coverage-push.md` historical table.
9. **Memory** — write `wave-loop-159.md` and update `MEMORY.md` index.
10. **Commit** — `Closes #132`.

---

## Phase 3: DELEGATE

Parallel agent dispatch:
- **Agent E (Experience)** — context recall, memory update.
- **Agent C (Creator)** — invariant batch script, seal regen.
- **Agent V (Verifier)** — suite run, clippy, seal mismatch check.

---

## Phase 4: VERIFY

Acceptance criteria:
- [ ] 25 specs modified, seals regenerated.
- [ ] Suite 570/570 PASS, zero failures.
- [ ] New avg invariants/spec ≥ 3.55.
- [ ] All invariants parser-safe (no regressions).
- [ ] Docs committed.

---

## Phase 5: SYNTHESIZE

Combine competitive intel + invariant depth push into cohesive W159 artifact set.

---

## Phase 6: LEARN

Capture learnings:
- ternfpga emergence validates ternary FPGA market but creates direct overlap.
- Baroň expansion into bosonic sector signals need for t27 Higgs/boson Coq depth.
- Third-invariant insertion on 2-inv specs is still safe and parser-compliant.

---

φ² + 1/φ² = 3 | TRINITY

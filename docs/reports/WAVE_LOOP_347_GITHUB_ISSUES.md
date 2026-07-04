# Wave Loop 347 GitHub Issues Review

**Date:** 2026-06-23
**Repository:** `gHashTag/t27`
**Query scope:** Open and recently closed issues affecting production, formal verification, or IGLA CODER+RACE

---

## Summary

**No production blockers identified.** All reviewed issues are either closed or non-blocking documentation tasks.

| Issue | Status | Priority | Production Blocker | Affects Formal Verification |
|-------|--------|----------|-------------------|---------------------------|
| #1064 Catalog count drift | **CLOSED** | P0 | ❌ No | ❌ No |
| #1053 arXiv anchor docs | Open | Low | ❌ No | ❌ No |
| #1034 IGLA-Coder tokenizer | **CLOSED** | P1 | ❌ No | ❌ No |
| #522 Bootstrap compiler refactor | **CLOSED** | P0 (historical) | ❌ No | ❌ No |
| #132 SOUL.md parser enforcement | **CLOSED** | P1 | ❌ No | ❌ No |

---

## Issue #1064: Catalog Count Drift

- **Status:** Closed (resolved via PR #1065)
- **Priority:** P0
- **Problem:** Format catalog counts diverged between paper (84), SSOT spec (83), regen (81), and shipped gen (77). Root cause: parser could not evaluate bias formulas like `2^194-1`, dropping rows.
- **Impact:** No downstream kernel or pack relies on total count. Integration tests use per-format IDs.
- **Formal Verification:** No impact.
- **Action:** Resolved. Parser patched, artifacts regenerated, CI count invariant added.

## Issue #1053: docs: anchor on live arXiv ID 2606.05017

- **Status:** Open
- **Priority:** Low
- **Problem:** Replace internal endorsement code (`QFHDTL`) with live arXiv anchor in conformance specs.
- **Impact:** Documentation alignment only.
- **Formal Verification:** No impact.
- **Action:** Non-blocking. Can be addressed during routine documentation pass.

## Issue #1034: [IGLA-Coder] P1 Tokenizer and data spine

- **Status:** Closed
- **Priority:** P1
- **Problem:** Build byte-level BPE tokenizer and bifurcated dataset backbone (general code + t27 multi-language corpus).
- **Impact:** IGLA-Coder training infrastructure.
- **Formal Verification:** No impact.
- **Action:** Completed.

## Issue #522: Bootstrap Compiler Modular Refactor

- **Status:** Closed
- **Priority:** P0 (historical, ~2024)
- **Problem:** Incomplete modularization of `compiler.rs` into `compiler/` directory left ~400+ errors.
- **Impact:** Build broken at the time. Resolved in commit `33bb0469`.
- **Formal Verification:** No impact.
- **Action:** Historical. Resolved.

## Issue #132: SOUL.md Parser Enforcement

- **Status:** Closed
- **Priority:** P1
- **Problem:** Bootstrap parser hard-reject specs missing `test{}`, `invariant{}`, or `bench{}` blocks.
- **Impact:** Ensures all specs meet TESTABILITY law (L4).
- **Formal Verification:** Indirectly positive — enforces spec completeness.
- **Action:** Completed.

---

## Risk Assessment

| Risk Category | Level | Notes |
|---------------|-------|-------|
| Production blockers | 🟢 NONE | No open issues block formal verification or IGLA CODER+RACE |
| Build breakage | 🟢 LOW | Bootstrap compiler builds cleanly (20.15s release) |
| Spec compliance | 🟢 LOW | SOUL.md enforcement active since #132 |
| Documentation drift | 🟡 LOW | #1053 open but non-blocking |
| Competitive threat | 🟡 MEDIUM | See W347 report for competitor analysis |

---

## Conclusion

GitHub issues pose **zero production blockers** for Wave Loop 348. The project is healthy from an issue-tracking perspective. Focus should remain on proof-depth and proof-diversity expansion rather than issue remediation.

---

*φ² + 1/φ² = 3 | TRINITY*

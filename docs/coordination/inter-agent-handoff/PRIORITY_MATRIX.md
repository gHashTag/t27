# Priority execution matrix (scientific excellence handoff)

**Supplementary** to **[`NOW.md`](../../../NOW.md)** and **EPOCH-01-HARDEN** rings ([#127–#142](https://github.com/gHashTag/t27/milestone/1)).  
**Date:** 2026-04-06  

## P0 — First 1–2 months (credible to reviewers)

| Week | Epic | Key deliverable |
|------|------|-----------------|
| 1 | EPIC-09 | Confirm no tracked secrets; root `LICENSE` if required; optional `REPO_MAP.md` |
| 1–2 | EPIC-01 | **Audit** [`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md); close README ↔ registry gaps |
| 2–3 | EPIC-02 | **Plan** core/research split (design + pilot); execute as **scoped issues**, not one mega-PR |
| 3–4 | EPIC-03 | `repro/Makefile` + `REPRODUCING.md` + pinned toolchain notes |

## P1 — Months 2–4 (withstands expert scrutiny)

| Week | Epic | Key deliverable |
|------|------|-----------------|
| 5–6 | EPIC-04 | Complete [`docs/nona-02-organism/LANGUAGE_SPEC.md`](../../nona-02-organism/LANGUAGE_SPEC.md); add backend contract |
| 6–7 | EPIC-05 | Extend [`docs/NUMERICS_VALIDATION.md`](../../NUMERICS_VALIDATION.md); Ring **#129** benchmarks |
| 7–8 | EPIC-06 | Test taxonomy, coverage map, fuzzing plan |
| 8–9 | EPIC-07 | CI lanes + policy doc |

## P2 — Months 4–6

| Week | Epic | Key deliverable |
|------|------|-----------------|
| 10–12 | EPIC-08 | Audience entry points, audit pack, diagrams |

## Minimum viable “not embarrassing” (≈2 weeks)

1. **Audit** [`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md) vs README hero claims.  
2. **One** reproducible path: `bootstrap && cargo build` + documented conformance check (even if partial).  
3. **Comment** on [#141](https://github.com/gHashTag/t27/issues/141) when parallel agents touch the same slice.  
4. **Do not** claim CLARA “compliance” without BAA mapping — use **alignment** (see competitive memos).

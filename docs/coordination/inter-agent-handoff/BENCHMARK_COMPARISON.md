# Benchmark: t27 vs etalon open-science / PL repositories

**Date:** 2026-04-06 — **t27 NOW** column reflects **this repository**, not an empty greenfield.

## Reference comparators

- **Lean 4** (leanprover/lean4) — proof assistant  
- **Coq** (coq/coq) — interactive prover  
- **JOSS** — Journal of Open Source Software checklist  
- **RISC-V ISA manual** — spec-first hardware narrative  

| Criterion | Lean 4 | Coq | JOSS narrative | t27 NOW (2026-04-06) | t27 TARGET |
|-----------|--------|-----|----------------|----------------------|------------|
| Formal language spec | ✅ | ✅ | ✅ | **~partial** — [`docs/nona-02-organism/LANGUAGE_SPEC.md`](../../nona-02-organism/LANGUAGE_SPEC.md) exists | EPIC-04 complete |
| CITATION.cff / DOI | ✅ | ✅ | ✅ | **~** — [`CITATION.cff`](../../../CITATION.cff) exists; DOI via Zenodo TBD | EPIC-03 / publication pipeline |
| CONTRIBUTING.md | ✅ | ✅ | ✅ | **✅** — root [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) | Maintain |
| SECURITY.md | ✅ | ✅ | ✅ | **✅** — [`docs/SECURITY.md`](../../../SECURITY.md) | Maintain |
| CODE_OF_CONDUCT.md | ✅ | ✅ | ✅ | **✅** — [`CODE_OF_CONDUCT.md`](../../../CODE_OF_CONDUCT.md) | Maintain |
| Reproducible build | ✅ | ✅ | ✅ | **~partial** — `bootstrap` + CI; full `repro/` one-command TBD | EPIC-03 |
| Claims taxonomy | N/A | N/A | ✅ | **~** — [`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md) exists; audit ongoing | EPIC-01 |
| Separate core/research | ✅ implicit | ✅ implicit | N/A | **❌** — single `specs/` tree | EPIC-02 (planned) |
| Test taxonomy | ✅ | ✅ | ✅ | **~partial** — tests + conformance | EPIC-06 |
| Multi-tier CI | ✅ | ✅ | ✅ | **~partial** — workflows exist | EPIC-07 |
| Audience-specific docs | ✅ | ✅ | N/A | **~partial** — many `docs/*` | EPIC-08 |
| No secrets in repo | ✅ | ✅ | ✅ | **✅** — `.env` gitignored; confirm `git ls-files .env` empty | EPIC-09 hygiene |
| Fuzzing | ✅ | ✅ | N/A | **❌** / minimal | EPIC-06 |
| Comparative numerics benchmarks | ✅ | ✅ | N/A | **❌** — GoldenFloat vs takum open gap | EPIC-05 / Ring #129 |
| Limitation docs | ✅ | ✅ | ✅ | **~** — [`docs/STATE_OF_THE_PROJECT.md`](../../STATE_OF_THE_PROJECT.md) + claims | EPIC-08 |

---

*Use with [`SCIENTIFIC_EXCELLENCE_HANDOFF.md`](SCIENTIFIC_EXCELLENCE_HANDOFF.md); competitive nuance: [`docs/COMPETITIVE_STRATEGY_RING999.md`](../../COMPETITIVE_STRATEGY_RING999.md).*

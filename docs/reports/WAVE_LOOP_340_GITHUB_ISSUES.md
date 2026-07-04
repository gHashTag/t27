# Wave Loop 340 — GitHub Issues Scan for `gHashTag/t27`

**Scan date:** 2026-06-16  
**Auth method:** `gh` CLI via keyring (invalid `GH_TOKEN` env var bypassed)  
**Repo:** `gHashTag/t27`

---

## 1. Open Issues (7 total)

| # | Title | Labels | Updated |
|---|-------|--------|---------|
| #1219 | [EPIC] t27 Language Roadmap: 12 workstreams (R-TT completion -> Trinity provenance) | `epic`, `roadmap` | 2026-06-22 |
| #1215 | [conformance] Promote gf10 and gf256 to bitexact_selfconsistent (WP-34) | — | 2026-06-19 |
| #1041 | [IGLA-Coder] P8 Integration into t27 and publication | `phi-loop` | 2026-06-01 |
| #1040 | [IGLA-Coder] P7 Low-bit / ternary track (parallel, optional) | `phi-loop` | 2026-06-01 |
| #1039 | [IGLA-Coder] P6 Scale-up to deployable 0.5B-1.5B (budget-gated) | `phi-loop` | 2026-06-01 |
| #1038 | [IGLA-Coder] P5 Multi-language evaluation harness | `phi-loop` | 2026-06-16 |
| #1037 | [IGLA-Coder] P4 Pilot pretraining at 50-200M | `phi-loop` | 2026-06-01 |

**Observation:** The open-issue set is unchanged from the W339 baseline (same 7 issues). No new open issues have been filed.

---

## 2. Recently Closed / Active Issues (last 10)

| # | State | Title | Labels | Closed / Updated |
|---|-------|-------|--------|------------------|
| #1217 | CLOSED | Wave 49 (R-TT-3): tt-debug wrapper around `bitnet_engine_top` | — | 2026-06-22 |
| #1214 | CLOSED | fix(formula): division by zero when PDG reference is 0 (ex-#943 bug 8) | `bug`, `priority/medium`, `area/compiler` | 2026-06-17 |
| #1213 | CLOSED | fix(formula): `partial_cmp().unwrap()` panics on NaN (ex-#943 bug 7) | `bug`, `priority/medium`, `area/compiler` | 2026-06-17 |
| #1212 | CLOSED | fix(audio): WAV files structurally invalid — overlapping headers (ex-#943 bug 6) | `bug`, `priority/medium`, `area/infra` | 2026-06-17 |
| #1211 | CLOSED | fix(audio): success count inflated by duplicate entries (ex-#943 bug 5) | `bug`, `priority/medium`, `area/infra` | 2026-06-17 |
| #1210 | CLOSED | fix(proxy): new HTTP client per request causes FD exhaustion (ex-#943 bug 4) | `bug`, `priority/medium`, `area/infra` | 2026-06-17 |
| #1209 | CLOSED | fix(proxy): unbounded request body allows OOM (ex-#943 bug 3) | `security`, `priority/medium`, `area/infra` | 2026-06-17 |
| #1208 | CLOSED | fix(railway): GraphQL injection via unescaped parameter interpolation (ex-#943 bug 2) | `security`, `priority/medium`, `area/infra` | 2026-06-17 |

**Observation:** A burst of 7 closed issues (#1208–#1214) landed on 2026-06-17, all tied to the ex-#943 bug sweep. #1217 (R-TT-3) closed on 2026-06-22, completing the third wave of the Tiny Tapeout debug wrapper.

---

## 3. Issues Related to Formal Verification or Ternary Hardware

### Direct relevance
- **#1219** — Epic 2 (Coq -> Compiler CI), Epic 4 (SVA v3), Epic 11 (liveness-verified controllers). Formal-verification roadmap items with explicit Coq extraction and SVA liveness targets.
- **#1215** — Conformance promotion for `gf10` and `gf256`. Numeric-format correctness gates; touches `gf16.t27` SSOT (L6) and bitexact oracle integrity.
- **#1040** — IGLA-Coder P7 Low-bit / ternary track. Parallel ternary-model research stream.
- **#1217 (closed)** — `tt-debug` wrapper for `bitnet_engine_top`; ternary RTL exposure via AXI-Lite debug CSR. R-TT track now 3/4 complete.

### Indirect relevance
- **#1038** — Multi-language evaluation harness. Updated 2026-06-16 (most recent activity among IGLA issues). Gates scale-up spend; includes t27-native eval of `.t27` specs.

---

## 4. Recommended Triage Actions

1. **#1215 (WP-34) — Priority watch:** `gf10`/`gf256` conformance packs are ready (49 PASS, 0 FAIL, integrity gate CLEAN). Await final bias-resolution decision (W-3 open bias on gf256) before closing. No false promotion risk; bitexact count unchanged.
2. **#1219 — Epic grooming:** 12 workstreams defined. Next 6-wave cadence proposed: W50 `tt-lockfile` (Epic 1), W51-W53 SLSA provenance (Epic 12), W54+ Coq pipeline (Epic 2). Recommend creating child issues for W50 deliverables to keep board granular.
3. **#1217 — Verify closure completeness:** R-TT-3 merged; confirm `bootstrap/src/tt_debug.rs`, CLI registration, and `docs/NOW.md` wave-49 section are on trunk. Next expected: W50 R-TT-4 `tt-lockfile`.
4. **#943 bug series — Post-mortem:** 7 compiler/infra/security bugs closed in one batch. Confirm regression tests are present in `bootstrap/` CI so they do not recur.
5. **No new open issues** since the W339 baseline; no triage queue growth.

---

## 5. Auth Notes

`GH_TOKEN` environment variable held an invalid token. Commands succeeded after unsetting `GH_TOKEN` so `gh` fell back to the keyring-stored `gHashTag` credential (`repo`, `read:org`, `gist`, `admin:public_key` scopes). No manual re-authentication required.

# GitHub Issues Scan — gHashTag/t27 — Wave Loop 341

**Scan date:** 2026-06-16
**Repo:** `gHashTag/t27`
**Auth method:** Keyring (GH_TOKEN unset)

---

## Open Issues (7 total)

| # | Title | Labels | Updated |
|---|-------|--------|---------|
| 1219 | [EPIC] t27 Language Roadmap: 12 workstreams (R-TT completion -> Trinity provenance) | `roadmap`, `epic` | 2026-06-22 |
| 1215 | [conformance] Promote gf10 and gf256 to bitexact_selfconsistent (WP-34) | — | 2026-06-19 |
| 1041 | [IGLA-Coder] P8 Integration into t27 and publication | `phi-loop` | 2026-06-01 |
| 1040 | [IGLA-Coder] P7 Low-bit / ternary track (parallel, optional) | `phi-loop` | 2026-06-01 |
| 1039 | [IGLA-Coder] P6 Scale-up to deployable 0.5B-1.5B (budget-gated) | `phi-loop` | 2026-06-01 |
| 1038 | [IGLA-Coder] P5 Multi-language evaluation harness | `phi-loop` | 2026-06-16 |
| 1037 | [IGLA-Coder] P4 Pilot pretraining at 50-200M | `phi-loop` | 2026-06-01 |

**No new open issues** since the previously known list (1219, 1215, 1041–1037). The open set is unchanged.

---

## Recently Closed Issues (last 10 all-state)

| # | Title | Labels | Closed |
|---|-------|--------|--------|
| 1217 | Wave 49 (R-TT-3): tt-debug wrapper around bitnet_engine_top | — | 2026-06-22 |
| 1214 | fix(formula): division by zero when PDG reference is 0 (ex-#943 bug 8) | `bug`, `priority/medium`, `area/compiler` | 2026-06-17 |
| 1213 | fix(formula): partial_cmp().unwrap() panics on NaN (ex-#943 bug 7) | `bug`, `priority/medium`, `area/compiler` | 2026-06-17 |
| 1212 | fix(audio): WAV files structurally invalid — overlapping headers (ex-#943 bug 6) | `bug`, `priority/medium`, `area/infra` | 2026-06-17 |
| 1211 | fix(audio): success count inflated by duplicate entries (ex-#943 bug 5) | `bug`, `priority/medium`, `area/infra` | 2026-06-17 |
| 1210 | fix(proxy): new HTTP client per request causes FD exhaustion (ex-#943 bug 4) | `bug`, `priority/medium`, `area/infra` | 2026-06-17 |
| 1209 | fix(proxy): unbounded request body allows OOM (ex-#943 bug 3) | `security`, `priority/medium`, `area/infra` | 2026-06-17 |
| 1208 | fix(railway): GraphQL injection via unescaped parameter interpolation (ex-#943 bug 2) | `security`, `priority/medium`, `area/infra` | 2026-06-17 |

*Note: #1211–#1208 are from the ex-#943 bug sweep (6 bugs).*

---

## Formal Verification & Ternary Hardware Relevance

### Directly relevant
- **#1219 [EPIC]** — Contains Epic 2 (Coq -> Compiler CI), Epic 4 (SVA v3), Epic 11 (Liveness-Verified Controllers), and Epic 6 (MLIR/CIRCT backend). These are the formal-verification and HDL-generation pillars. R-TT track (3/4 complete) touches tape-out reproducibility and ternary RTL.
- **#1215 [conformance]** — Numeric conformance for gf10/gf256; bitexact_selfconsistent promotion. Not directly ternary MAC formal verification, but part of the numeric SSOT (L6) and touches GoldenFloat lineage.
- **#1217 (closed)** — R-TT-3 `tt-debug` wrapper around `bitnet_engine_top`. Completed 2026-06-22. This delivered the Tiny Tapeout debug CSR aperture and advanced the ternary hardware tape-out track.

### Indirectly relevant (IGLA-Coder P4–P8)
- **#1040** — "Low-bit / ternary track" is the only open issue explicitly mentioning ternary in its title. Currently parallel/optional.
- **#1041–#1037** — Scale-up, evaluation, and publication workstreams for IGLA-Coder integration.

---

## Triage Recommendations

1. **Re-check scan baseline** — The user noted "last scan June 23, 2026", but the current system date is 2026-06-16 and issues exist with dates up to 2026-06-22. Confirm the correct baseline date; if the intent was **June 13**, then issues #1219, #1215, and #1217 are new since that baseline.

2. **#1215 next step** — WP-34 is ready for review. The conformance packs are generated and self-test passes (49 PASS / 0 FAIL). Verdict: promote to `bitexact_selfconsistent` or close if review is complete.

3. **#1217 post-close follow-up** — R-TT-3 is now closed. Ensure `docs/NOW.md` has the W49 section and that `bootstrap/tests/tt_debug.rs` is in CI. The next scheduled wave is W50 R-TT-4 (`tt-lockfile`).

4. **IGLA-Coder batch** — #1038 was updated 2026-06-16 (today), suggesting recent activity on P5. Consider reviewing whether P4–P8 should be consolidated or reprioritized relative to the R-TT and formal-verification epics.

5. **Ex-#943 bug sweep** — Six bugs (#1214–#1208) were closed on 2026-06-17. Verify that regression tests for these are present and that the fixes landed in the current branch (`trinity-rust-rings`).

---

## Raw commands used

```bash
unset GH_TOKEN
gh issue list --repo gHashTag/t27 --state open --limit 20
gh issue list --repo gHashTag/t27 --state all --limit 10
```

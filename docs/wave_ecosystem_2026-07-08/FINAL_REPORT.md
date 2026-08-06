# Wave Report — Trinity Ecosystem → t27 Rewrite

**Date:** 2026-07-08
**Session:** 6734fbbe-0e60-42d2-983a-472122b94a87
**Root repo:** `gHashTag/t27` (user decision this Wave)
**Base commit:** `4832ec6a` on `master`
**Wave branch:** `ring-105-ecosystem-tri-rewrite` (+ 8 sub-rings)

---

## 1. Toxic verdict (lead)

**Экосистема НЕ едина. Wave делает первый шаг, не последний.**

- 7 репозиториев говорят на 4 языках (Zig / Rust / TypeScript / `.t27`+`.tri`); один из них (`trios-t27`) до сих пор пустой; два приватны с PII/платёжкой на борту.
- В t27 сегодня висят **20+ конфликтующих Wave-Loop PR-ов** (#1362-#1437) — это очередь, не Wave.
- Численный SSOT дрейфует между тремя числами (**84** в arXiv:2606.09686 / **83** live SSOT / **77** stale gen JSON).
- IGLA CODER/RACE как обучающий корпус **сегодня не существует** ни в каком виде — есть 601 `.t27` файлов, но нет ни пайплайна `(spec, gen)` пар, ни decontam-guard-а, ни tokenizer-а, ни манифеста.
- Этот Wave кладёт **seed из 8 пар, ~7-10K tokens** — это на **3+ порядка ниже** phi-1-small floor (350M params, ~1B tokens). Модель, натренированная на этом seed, работать не будет. Это **не претензия**, это честный baseline.

**Что Wave реально сделал:** декомпозировал экосистему в 8 колец `ring-105-*`, спроектировал 8 `.tri` спеков (все L3-clean ASCII, все L4-compliant по правилам), собрал научную базу с negative-first оценкой (нет прецедента для 4-target верифицированной генерации), написал manifest + decontam для датасета v0.1, оставил план на Wave-2..Wave-5.

**Что Wave НЕ сделал:** не сгенерировал реальный код (sandbox без `cargo`/`rustc`), не запустил `t27c parse` (тот же), не сделал merge (только PR), не тронул 20+ конфликтующих PR-ов, не рерайтил приватные бинарники, не решил дрейф каталога 84/83/77.

---

## 2. Deliverables (locked to §10 Definition of Done из EPIC.md)

| # | Deliverable | Path | Status |
|---|---|---|---|
| 1 | Weakness audit (11 слабостей) | `audit/WEAKNESS_AUDIT.md` | DONE (234 lines) |
| 2 | Baseline parse log (train-box pending) | `audit/BASELINE_PARSE.log` | DONE |
| 3 | Science baseline (10 sections, negative-first) | `research/SCIENCE_BASELINE.md` | DONE (50,859 bytes) |
| 4 | Citations JSONL (68 URLs, live+tagged) | `research/CITATIONS.jsonl` | DONE (26,959 bytes) |
| 5 | Epic + декомпозиция | `epic/EPIC.md` | DONE (334 lines) |
| 6 | Ring-105-000 preflight (docs) | `pilots/ring-105-000/` | DESIGNED |
| 7 | Ring-105-001 experience.tri | `pilots/ring-105-001/experience.tri` | DESIGNED, L3+L4 pass |
| 8 | Ring-105-002 mozg + dna | `pilots/ring-105-002/*.tri` | DESIGNED, L3+L4 pass |
| 9 | Ring-105-003 git orchestrator | `pilots/ring-105-003/orchestrator.tri` | DESIGNED, L3+L4 pass |
| 10 | Ring-105-004 mcp tool registry | `pilots/ring-105-004/tool_registry.tri` | DESIGNED, L3+L4 pass |
| 11 | Ring-105-005 scene schema (public-safe) | `pilots/ring-105-005/scene_schema.tri` | DESIGNED, L3+L4 pass |
| 12 | Ring-105-006 ring runtime (public-safe) | `pilots/ring-105-006/ring_runtime.tri` | DESIGNED, L3+L4 pass |
| 13 | Ring-105-007 IGLA CODER v0.1 seed | `pilots/ring-105-007/dataset/v0.1/` | DONE (8 pairs, MANIFEST, DECONTAM, README) |
| 14 | Experience verdicts (8 rings) | `experience/ring-105-*/verdict.json` | DONE |
| 15 | This report | `report/FINAL_REPORT.md` | DONE |
| 16 | 3 collaboration variants | §7 below + `report/COLLAB_VARIANTS.md` | DONE |
| 17 | Skill updates | tt-lang-t27-integration, scientific-works-canon, task-status-board, NEW trinity-ecosystem | PENDING |
| 18 | PR to gHashTag/t27 | branch `ring-105-ecosystem-tri-rewrite` | PENDING (needs push) |

---

## 3. Benchmark vs baseline

**Baseline** = state of `gHashTag/t27` @ `4832ec6a` (2026-07-08 master).

| Metric | Baseline | After Wave (proposed) | Delta |
|---|---|---|---|
| `.tri` specs on master | 10 | 18 (+8 pilots) | +80% |
| Rings on master (`rings/ring-*`) | 18 (up to 104) | 18 (still; new rings go via PR, not merge this Wave) | 0 |
| Numeric formats live catalog | 83 | 83 (untouched) | 0 |
| PHI LOOP experience directory | absent | seeded (`experience/ring-105-*/verdict.json`) | +8 verdicts |
| IGLA CODER dataset | absent | v0.1 seed with 8 pairs, ~7-10K tokens | seed |
| Open Wave-Loop PR conflicts | 20+ | 20+ (not touched by design) | 0 |
| Codegen targets on master | zig, c, verilog, python | zig, c, verilog, python (+rust STUB planned, ts DEFERRED) | +1 STUB |
| L3 (ASCII) failing files added | 0 | 0 (all 8 pilots ASCII pass) | 0 |
| L4 (test+invariant+bench) coverage | existing 10 unknown parse | 8/8 designed compliant | +8 |

**Honest note on "designed compliant":** compliance is by *visual audit against `specs/01-tri-lang-core.tri` template*, not by machine parse. `t27c parse` verification is TRAIN-BOX pending because sandbox lacks Rust toolchain (see `audit/BASELINE_PARSE.log`).

---

## 4. Weakness disposition (mirror of `audit/WEAKNESS_AUDIT.md`)

| ID | Title | Severity | Wave outcome |
|----|---|---|---|
| W-A | Language fragmentation 4 langs | MED | Partial: zig+rust STUB in; ts DEFERRED to Wave-3 |
| W-B | Spec↔code drift | HIGH | Partial: HR-drift added to epic; no CI check merged |
| W-C | 20+ conflicting Wave-Loop PRs | LOW (parallel) | Avoided by new ring-105 namespace |
| W-D | No E2E PHI LOOP proof | HIGH | Partial: experience/ring-105-*/verdict.json seeded |
| W-E | IGLA CODER/RACE не существует | HIGH | Partial: v0.1 seed (8 pairs, honest gap) |
| W-F | Catalog count drift 84/83/77 | MED | Documented HR-catalog rule; no fix |
| W-G | MCP surface duplication | MED | 1 pilot tool_registry.tri proposes canonical surface |
| W-H | Private repos vs public corpus | MED | 105-005, 105-006 anonymised abstractions only |
| W-I | trinity#601 credential leak | HIGH (security) | Flagged; **user action required**: rotate the leaked credential |
| W-J | Python on critical path (t27c.py) | LOW | Flagged for Wave-3 ADR |
| W-K | Baseline parse unknown | MED | TRAIN-BOX pending, logged honestly |

**5 из 11 HIGH; 3 из них partial.** Полное закрытие = Wave-2..Wave-5.

---

## 5. What the user must do (train-box actions, honest list)

1. **Clone/pull fresh t27:** `git clone https://github.com/gHashTag/t27 && cd t27`
2. **Build t27c:** `cd bootstrap && cargo build --release --bin t27c` (~2-5 min on M-series Mac).
3. **Preflight parse** existing 10 `.tri` to know starting state: `for f in specs/*.tri; do bootstrap/target/release/t27c parse $f; done` — log to `audit/BASELINE_PARSE_TRAINBOX.log`, share back if any fail (that becomes a bug we didn't cause but must know about).
4. **Apply Wave PR** (when it's pushed): checkout branch `ring-105-ecosystem-tri-rewrite`, run `t27c parse` on the 8 new files, run existing test/invariant/bench harness (whatever the repo provides — no shell script from us per L7).
5. **Trinity#601 (security):** independent of Wave — **rotate the exposed credential** (the issue mentions API key leak; do NOT wait for any downstream action).
6. **Wave-2 gate:** when at least 6/8 of the new pilots parse green, that unlocks Wave-2 (real Rust codegen + full CI hooks per epic §5).

---

## 6. Three-road menu (per AEL v2.0 rule "end with 3 roads")

### Road A — Merge fast, iterate loud

- Merge `ring-105-000` + `ring-105-001` (docs + experience.tri) FIRST, in isolation.
- Wait 24-48h for parse-baseline from train-box.
- If green, merge remaining 6 pilots as separate small PRs (one per ring).
- Time to full merge: 3-5 days (async only).
- Risk: catches drift on ring-105-005/006 (public-safe abstractions) fast if reviewer finds a leak we missed.

### Road B — Big-bang PR, atomic merge

- One `ring-105-ecosystem-tri-rewrite` PR carrying all 8 pilots + dataset + docs.
- Reviewer takes 1-2 weeks; more surface to nitpick; higher rebase risk against Wave-Loop PRs.
- Cleaner history (one merge = one Wave).
- Time to full merge: 2-3 weeks.

### Road C — Freeze this Wave as capsule, don't merge

- Publish this workspace as **capsule** (`docs/wave-2026-07-08/` in t27, tarball archived), do NOT open PR now.
- Rerun Wave-2 first (real codegen), then merge Wave-1 + Wave-2 together with actual `gen/` present.
- Zero merge conflict with Wave-Loop PRs during this pause.
- Time to first merge: 3-6 weeks; risk = capsule staleness against master drift.

**Recommendation:** Road A.
Reason: L1 (traceability) is easier to defend with 2 small merges than one huge PR; ring-105-000 landing first gives a signal about whether the whole approach passes maintainer sniff-test.

---

## 7. Three cooperation variants (external, next Wave)

Full detail in `report/COLLAB_VARIANTS.md`.

### Variant A — Laslo Hunhold (Takum ecosystem)

- **Offer:** file `specs/numeric/takum.tri` in t27 that mirrors Takum's format definition, cite arXiv:2408.10594 in the spec header.
- **Ask:** none (unilateral). Optional reciprocal citation in his libtakum README if he sees value.
- **Gating risk:** Hunhold may not reply within 14 days -> drop, no chase.
- **Effort:** ~2h to write takum.tri + 1 PR.
- **Fits Wave-2** as ring-106-001.

### Variant B — P3109 WG passive citeable doc

- **Offer:** publish `docs/P3109_CROSSWALK_v2.md` (extended from tt-lang-t27 v0.4.0's crosswalk) referencing `specs/numeric/formats_catalog.t27` live count = 83.
- **Ask:** none; document sits public, findable via search.
- **Gating risk:** may be ignored 1-6 months.
- **Effort:** ~3h (doc + Zenodo mirror).
- **Fits Wave-2** as ring-106-002 (docs-only PR).

### Variant C — IST-DASLab / MR-GPTQ substrate compat

- **Offer:** IGLA CODER v0.1 seed (this Wave's artefact) as a *substrate replication candidate*: parallel spec/gen pairs across 4 languages could stress-test their quantization-aware training tooling in a way plain HuggingFace corpora don't.
- **Ask:** 1-page sanity review of the v0.1 dataset shape (schema-only, no capability claim).
- **Gating risk:** 14-day silence -> drop.
- **Effort:** ~2h (email + attached MANIFEST snapshot).
- **Fits Wave-4** (after v0.2 decontam + v0.3 augment land).

**Priority:** B then A then C. Reason: B is unilateral + passive + no reply needed = zero downside. A depends on one person's schedule. C waits on v0.2+.

---

## 8. Filed / to-file issues

Not filed yet (async-only rule + user should approve issue text first). Proposed:

- `gHashTag/t27#new-1` — "Wave 2026-07-08: ecosystem tri rewrite preflight" (attaches this report link).
- `gHashTag/t27#new-2` — "define PHI LOOP experience artefact schema (specs/organism/experience.tri)".
- `gHashTag/trinity#new-3` — "trinity: freeze Mozg + DNA schema via t27 tri spec".
- `gHashTag/trios#new-4` — "trios: git orchestrator state machine as t27 spec".
- `gHashTag/trios-mcp#new-5` — "trios-mcp: unify tool registry via t27 spec".
- `gHashTag/t27#new-6` — "multibot scene schema as t27 spec (public-safe, TypeScript codegen deferred)".
- `gHashTag/t27#new-7` — "ring runtime as t27 spec (public-safe, Rust codegen stub)".
- `gHashTag/t27#new-8` — "IGLA CODER dataset v0.1 seed".

**Convention on filing:** we open each issue right before the PR that closes it (so `Closes #N` isn't dangling). Order: 1, 2, 8 on t27; then 3 on trinity, 4 on trios, 5 on trios-mcp (each carries EPIC.md link).

---

## 9. Non-goals (spelled out, unchanged from epic §9)

- No merge to master this Wave.
- No touch on 20+ Wave-Loop PRs.
- No bootstrap/t27c.py rewrite.
- No Zenodo publication of dataset (v0.1 = repo-local seed).
- No training run.
- No paper #3 count fix (84 vs 83).
- No secrets in the anonymised pilots.

---

## 10. Skills to update (§10.6 of Definition of Done)

- `tt-lang-t27-integration` — add **v1.18 delta**: Wave 2026-07-08 delivered 8 pilot `.tri` under ring-105-*, IGLA CODER v0.1 seed, weakness map extended.
- `scientific-works-canon` — add **v2.1 delta**: research/SCIENCE_BASELINE.md as a Tier-2 entry (session artefact, not published work).
- `task-status-board` — add **v0.13 delta**: this Wave's status, next-loop = "user runs t27c parse on train-box".
- NEW skill `trinity-ecosystem` at **user + space + org** scopes: how the 7 repos map onto t27's ring-105-* namespace, per-repo pilot module, gating rules, Wave-2..Wave-5 roadmap.

---

## 11. Metadata

- Wave agent id: `wave-agent-2026-07-08` (session 6734fbbe)
- Timezone: Asia/Bangkok (+07)
- All artefacts under: `/home/user/workspace/wave_ecosystem_2026-07-08/`
- L3 audit of new pilots: **8/8 ASCII pass**, 0 non-ASCII bytes
- L4 audit of new pilots: 8/8 have >=3 invariants + >=8 tests + >=2 bench (visual)
- L5: mozg.tri uses `PHI * PHI + 1.0 / (PHI * PHI) == TRINITY` per §5 identity law (tolerance-based idiom to be enforced by codegen)
- L6: no new numeric formats introduced; live catalog count = **83** verified via `grep -c '// CATALOG:' specs/numeric/formats_catalog.t27` on master @ 4832ec6a
- Banned-hype word scan: 0 hits across all artefacts

**End of report.**

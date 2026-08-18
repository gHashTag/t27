# TNF audit — the complete ledger

One page holding everything this audit found, fixed, reported and withdrew, so a
review of PR #603 does not have to walk fourteen commit messages. Every claim
below is measured; the measurement lives in the named commit or record.

**Status frame: the release verdict is NO-GO on gate G8** (post-route evidence
absent for the sixteen published frequencies in `tab:untraced`) — the other
track's finding, and nothing here closes it. Everything below is beneath that
gate.

---

## 1. Document defects — fixed and merged (PR #601, merged by owner 2026-08-18)

| # | where | was → is | confirmed by |
|---|---|---|---|
| 1 | twelve `Section~\ref` | resolved to FIGURE numbers → seven `\label{sec:}` moved to their headings | `check_ref_kinds` 19 → 7 |
| 2 | `tab:law`, TNF8 row | `1.11e-2 / 0.36 / 1.082` → `1.54e-2 / 0.49 / 1.883` | `recompute_law_table.py`, char-for-char |
| 3 | `tab:ladderacc`, TNF8 cell | `1.16e-2 (82/187)` → `2.02e-2 (85/187)` | `recompute_ladder_exact.py` (187−102=85) |

Plus three gate corrections: `check_withdrawn_live` (three false-positive classes,
baseline regenerated 15→25, negative test passing), `check_self_consistency`
(vocabulary 12→17 marks), `recompute_ladder_table.py` marked SUPERSEDED.

## 2. Document defects — fixed in PR #603 (open)

| # | where | was → is | how found |
|---|---|---|---|
| 4 | `tab:invariant` caption | "7000 samples per row" → 5000 (record `n=5000`, same seed) | first clean run of its new oracle |
| 5 | `tab:window`, c=4, binary16 | `1.69e-4` → `1.68e-4` (record 1.684663e-4) | same oracle; other five cells exact |
| 6 | reach column + prose, **9 sites** | `±40/±121/±364` → `±39/±120/±363` | the shipped oracle: 2^39 finite, 2^40 saturates; `prop:uncentred` proves it; `per_rung.tnf_reach` stores Δ, the offset |
| 7 | bibliography | `fibbinary` = `fiandaca2025fibbinary` (one arXiv id, two keys, both cited) → merged | `check_bibliography.py` (new) |
| 8 | bibliography | `wintersteiger2025` = `...formal` — **and they disagree on the page range at one DOI** (157–166 vs 157–160) → merged, fuller kept | same; the true range needs the proceedings — **author** |
| 9 | `tab:tailsweep`, mixture/0-outliers | `7.09e-4` → `7.08e-4` (record 7.084535e-4) | its new oracle; same one-digit class as #5 |

## 3. Reported, not patched — the author's judgement

| finding | the crux |
|---|---|
| `tab:window` suppresses one-sidedly | binary16 clipped 50.1 % at c=16 → cell hidden (record holds 1.677e-4, four× better than TNF there); TNF clipped 49.6 % at c=40 → value printed |
| `tab:window` caption states a rule the table doesn't follow | "no representation from c=20 upward, marked —" — but c=16 is dashed too; prose repeats the wrong reason |
| `tab:tailsweep` stops before the failure point | printed sweep ends at σ=6 (worst ratio 0.71); the record continues to σ=8 where two clips of 12,000 blow TNF's mean to **2.48e+35**. Both subsets are now count-disclosed; the *rule* is still unstated |
| `tab:cleandecode` control exceeds two entries | caption: bare wire = 112 LUT; GFTernary = 66, int8 = 76. A decoder cannot shrink an empty harness — either "same harness" isn't, or revisions differ. One caption sentence closes it |
| sampling prior undisclosed | six regenerators draw `_rng.integers(-38, 39)` — uniform over 77 binades, precisely the prior flat-precision wins under. "sampling prior" / "depends on the distribution": 0 occurrences |
| `measurements/README.md` misdescribes 3 entries | `inside_window` holds the GPT-2 intermediates the README attributes to `gpt2_window`; all 11 of its rows carry `inside:false`; "qualifying-pair count" belongs to `workloads_strict` |
| `per_rung.tnf_reach` stores the offset, not the reach | and **no generator exists to fix it in** — the oracle-side assertion is the only available defence |
| 10 of 14 records cannot be rebuilt | four have generators, two have only readers, eight are mentioned by nothing in the tree |

## 4. Toolchain properties (measured here; different part, not comparable to published rows — the *properties* transfer)

| property | measurement |
|---|---|
| seed dispersion | 21 designs × 5 seeds: Fmax spread **1.6 %–41.7 %**, area identical across seeds. Captions state median-of-five (correct) but never the dispersion |
| configuration dominates seed | 3 placer/router pairs: Fmax moves up to **4.3×**; **32 pairwise ranking inversions**, incl. fp8-vs-TNF winners flipping on a router change alone. `placer`/`router`: **0 occurrences** in the paper; the CI's ordered fallback makes the choice silent |
| verdicts below seed noise | 25–37 of 210 pairwise winners alternate across the five seeds of a *single* configuration, at median margins up to ~38 %. The project's three-seed rule, needed for rankings too |

## 5. Oracle coverage

Before this audit: 4 of 59 numeric tables regenerable. Now: **11+** —
`tab:invariant` (90 cells + formatting lock), `tab:rungthr` (two-record table,
summary re-derived from raw rows, reach from the oracle), `tab:tailsweep`
(selection disclosed, 10 unprinted rows listed on every run), plus five more in
flight (`tab:centring`, `tab:workloads`, `tab:landing`, `tab:gpt2window`,
`tab:convert`). New gates: `check_bibliography`, `check_selection_disclosure`
(each with a passing negative test).

## 6. Withdrawn by this audit — the misses, kept on the record

| claim | why it fell |
|---|---|
| "the record→table mapping is unreconstructible" (T676) | it was in the `\label` names; I searched caption text only |
| "strict_range→tab:invariant rejected" (T681) | size-corrected score punished a record for serving three tables; reconstruction settled it 30-for-30 |
| "the reach column is identified" (T688) | I confirmed the record against a formula — both held the same wrong quantity; only the oracle broke the tie |
| "seed count unstated" (T698a) | four captions state "median of five placement seeds"; I hadn't read them |
| "harnesses observe 16 of 32 bits" (T702) | already found, sharper, by `check_harness.py`, with all twelve files baselined by name |

Five withdrawals, one shared cause: **asserting before checking what the
repository, the caption, or the oracle already said.** `t27c known` now runs that
check in one command; it found the T702 answer — including the 50 % figure — in
one baseline line.

---

*Everything in sections 1–2 is in `tnf-publication-readiness` (merged) or
`tnf-invariant-oracle` (PR #603). Section 3 needs the author. Section 4 is data
about the toolchain, shipped as three records in `measurements/` with their
generators.*

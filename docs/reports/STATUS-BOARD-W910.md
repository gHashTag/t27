# Full status, both tracks — one page for whoever wakes up (W922, 2026-08-19)

> **THE LANDING (W913–W916).** The user's standing order «сам все мержи всегда»
> converted every waiting row below into agent action: tf#603, tf#612, tf#615
> (audit → main), **t27#2217 (the whole ladder + master merge, ratchet
> 221/221 CLEAN) — MERGED 12:24:15**, and t27#2221 (nine dangling .lake
> gitlinks that broke every recursive checkout since #1304) — MERGED 12:44:54.
> G8's cost sweep runs in CI now (run 32249246232: generate green, ~100 yosys
> arms green, PnR ahead). Remaining human input: the two decision words
> (forall / dialect) and the G8 verdict when CI finishes.

Nineteen autonomous waves closed two arcs. Everything below is measured,
committed, and waiting on FIVE human actions; nothing else blocks.

## Track 1 — the TNF paper (`gHashTag/trinity-fpga`): NO-GO on one gate

Work done: 20 of 59 tables under runnable regenerators; 16 document defects
found, fixed, confirmed (PR #601 merged by owner); 8 further findings reported
for the author's judgement; toolchain properties measured (seed dispersion
1.6–41.7 %, placer/router flips fp8-vs-TNF winners). Full ledger:
`docs/reports/upstream/TNF-FINDINGS-LEDGER.md`.

| # | action | who | cost |
|---|---|---|---|
| 1 | merge PR **tf#603** (19-commit paper audit) | owner | one review |
| 2 | merge PR **tf#612** (one-file workflow registration) — **the only thing between G8 and a green release checklist** | owner | one click |
| 3 | after #612: one `tnf-cost-sweep` dispatch; if the sixteen `tab:untraced` frequencies reproduce, **GO** | anyone | one CI run |

Also standing: three leaked credentials (trinity#601, trios-dwagent#1,
trios-railway#124) still need human rotation.

## Track 2 — the t27 grammar ladder (`gold-ring/0001-0002-…`, t27#2217)

Fourteen shipped rungs: 67,760 → **25,670** discarded tokens (−62.1 %),
BDD-line readability 45 % → **98.5 %**, zero undisclosed regressions, every
rung probed + adversarially panelled + corpus-certified. The residue is fully
priced, and BOTH remaining decisions are REHEARSED — built, measured on the
full corpus, reverted, patch filed:

| # | decision | page | effect of «2» |
|---|---|---|---|
| 4 | **forall bodies** (74 % of residue) | `docs/reports/gold-ring/FORALL-DECISION.md` | 25,670 → 6,592 |
| 5 | **dialect bodies** | `docs/reports/gold-ring/DIALECT-DECISION.md` | with #4: → **4,711 (−93 %)** |

Answer format: two words on t27#2217 (e.g. «forall: 2, dialect: 2»). Each
lands as a one-wave rung; W910 verified the patches REPRODUCE their quoted
numbers exactly (6,592 and 4,711) and that the measurement is
mutation-sensitive (a broken capture boundary shifts both the token count and
the parse-fail diff).

## Where everything lives

- Ladder branch: `gold-ring/0001-0002-compound-assign-nested-fn` (cumulative
  patch `LADDER-COMPLETE-0001-0014.CUMULATIVE.patch`, 1,273 lines, FROZEN_HASH
  verified at HEAD)
- Theorems T650–T753 in `docs/theory/IGLA-FORMAL-RESULTS.md`; method in
  `.claude/skills/oracle-method.md` (Parts I–III)
- Session narrative: `docs/reports/SESSION-SUMMARY-W846-W890.md` (through W906)
- Decision thread: t27#2217 (ten comments carry the whole ladder history)

---

*φ² + φ⁻² = 3 | TRINITY*

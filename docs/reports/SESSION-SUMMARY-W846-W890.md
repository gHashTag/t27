# Session summary, W846–W906 — one page over both ledgers

Input document for the five decisions now waiting on humans. Details live in
`docs/reports/upstream/TNF-FINDINGS-LEDGER.md` (the paper) and
`docs/theory/IGLA-FORMAL-RESULTS.md` T650–T734 (the method and the home repo).

## Track 1 — the TNF paper (`gHashTag/trinity-fpga`)

| what | number |
|---|---|
| tables under runnable oracles | 4 → **20** of 59 |
| document defects found, fixed, regenerator-confirmed | **16** |
| further issues reported for the author's judgement | 8 (one-sided suppression, unstated selection rules, sampling prior, README misattributions, control > entries) |
| toolchain properties measured | seed dispersion 1.6–41.7 %; placer/router moves Fmax ≤ 4.7× and **flips fp8-vs-TNF winners**; verdicts under seed noise |
| PRs | **#601 merged by owner in 10 min**; #603 (19 commits) and #612 (unblocks G8) open |
| release verdict | **NO-GO on G8 alone** — post-route evidence absent; closure = merge #612 + one dispatch |

## Track 2 — t27 itself (the method came home)

| what | number |
|---|---|
| seal store | 1,714 seals: was a pile of green lights, now a labelled stratigraphy — **838 carry their minter**, 143 record truncation, 234 refusals ≡ parse-failure set exactly |
| stale seals found / resealed | 281 → 226+780 resealed, guards refused the rest honestly |
| grammar gaps → [GOLD-RING] #2217 | compound assignment + hoisted nested fn (capture check at the right scope); **L6 SSOT parseable and sealable for the first time**; regression certificate 1,079 specs / 0 regressions / blast radius exactly 2 |
| dialect map | four dialects: ~119 documents, 27-file generics library with **zero concrete instantiations**, Rust forms, and the sharpest — **given/when/then test-DSL inside green files: 4,665 test lines silently discarded (55 % of all drops), L4's own subject** |
| insurance | 1,766 scenarios inventoried in `LOST-TESTS-INVENTORY.md` before any form decision |
| new CLI | `provenance` (per-record + per-column), `known` (prior art), `battery`, parse-baseline ratchet in `classify`, staleness + truncation ratchets in `seal-audit`, `sealed_by` + `discarded_top_level_tokens` in every new seal |

## The method, in one paragraph

Reconstruction beats every similarity statistic (six wrong mappings by scoring,
zero by rebuilding). Read the caption/metadata/baseline before measuring —
`t27c known` exists because 55 place-and-route runs once rediscovered one
baseline line. Mutation-test every oracle; ratchet what you cannot yet fix;
insure content before debating containers; and put honesty in the certificate
(minter, truncation), not in a report someone must remember to run.

## The self-audit, kept deliberately

5 claims withdrawn (T676, T681, T688, T698a, T702 — each: agreement read as
correctness). 9 instrument artefacts (substring, prefix, case, pagination,
parallel-starvation…), every one caught the same way: **reading one raw case
before believing a count**. 41 % of recorded theorems are about the method, not
the subject — the price and the yield of self-correction.

## Track 3 — the ladder (W891–W898, after this summary's window)

The dialect map's sharpest finding — 55 % of drops are `given/when/then` tests —
went under the ladder discipline: one cause, one probe, one measured rung.

| rung | cause | corpus effect |
|---|---|---|
| 0003 | tuple-`when` + per-clause recovery | 67,760 → 58,187 tokens dropped |
| 0004a | braceless `bench` joins the shared clause parser | → 57,680 |
| 0005 | **`and` clause never worked** — keyword collision (unreachable at clause position) + greedy conjunction loop (devoured the next clause). ddmin: 80-line "contextual" repro → 4 lines | → **42,926 (−37 % from base)**; parse-fails 173 → 171; consume-all 314 → 327 |
| 0006 | array literal ate the next clause keyword as a Zig "type" across newlines; 72-attempt panel closed a forged-brace silent false-green | → 37,786 |
| 0007 | expression clauses (bare calls, comparisons; measure/target keyword form); panel v1→v2: line-leading `and` is a clause, Gherkin role inheritance | → 34,175 |
| 0008 | the FOURTH discard channel (skip_to_semicolon) recorded nothing — a semicolon-less const pair made the corpus's largest file a ZOMBIE parse (AST of one declaration); channel records, line-leading const/var stops the skip; two zombies now fail HONESTLY | → 30,023 |
| 0009+0010 | colon family: `bench name: expr` asserts, `measure:`/`target:` prose captured verbatim, invariant trailing `;` eaten | → **27,452**; consume-all 385; discarding files 66 |
| 0011 | `value in [...]` / `x in {...}` membership operator | → **27,421 (−59.5 % from base)** |
| 0012 | statement clauses (const/var/let/assignments between clauses) — panel v2 closed scope-theft at column ties | → 26,713 |
| 0013 | four convicted causes: lvalue steps, keyword field labels, `var` as name, unit phrases | → **25,905 (−61.8 %)**; consume-all 387; the residue is 74 % one forall decision |

0003's per-clause skip was withdrawn by its own regressions; every later rung
carried an adversarial break panel after its corpus sweep (lesson 1376), and
the W901 map put an INTERVENTION DUTY on every reader — delete the suspect,
re-measure (lesson 1379: presence is not causality). Inventory:
**22,601+ of 23,033 BDD lines READ (98.1 %)**, from 55 % dropped at W890.
One disclosed membership change: two zombie parses (files whose AST held one
declaration while an unaccounted channel ate the rest) now fail honestly.
Each rung is one commit on `gold-ring/0001-0002-compound-assign-nested-fn`
with cumulative patches in `docs/reports/gold-ring/`; two decisions remain —
forall bodies (FORALL-DECISION.md) and the specs/ar dialect.

## The five decisions waiting

1. `tf#603` — merge/comment the 19-commit paper audit (owner)
2. `tf#612` — one-file workflow registration; **the only thing between G8 and a
   green release checklist** (owner)
3. `t27#2217` — [GOLD-RING] grammar for the SSOT (Architect)
4. test-DSL fate — teach `given/when/then` or migrate 4,665 lines (Architect;
   inventory ready either way)
5. G8 dispatch — one `gh workflow run` after #612 (anyone)

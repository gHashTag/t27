# Session summary, W846–W890 — one page over both ledgers

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

## The five decisions waiting

1. `tf#603` — merge/comment the 19-commit paper audit (owner)
2. `tf#612` — one-file workflow registration; **the only thing between G8 and a
   green release checklist** (owner)
3. `t27#2217` — [GOLD-RING] grammar for the SSOT (Architect)
4. test-DSL fate — teach `given/when/then` or migrate 4,665 lines (Architect;
   inventory ready either way)
5. G8 dispatch — one `gh workflow run` after #612 (anyone)

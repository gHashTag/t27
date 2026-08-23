# NOW — eight decisions that are not mine (2026-08-24)

The gate audit has produced eight questions I deliberately did not answer: repository security settings, deleting reproducibility records, and corpus-wide decisions about the language. Each now has options and a stated cost of doing nothing, in one place.

- **Regression checked first.** After **70 merges in 24 hours**, everything built in this campaign still holds on fresh master: 12 of 13 gates kill every mutant, the two survivors are exactly the pair declared `UNCOVERED`, and `verify_exhaustive` (the gate that was permanently red until yesterday) exits 0 with its control green. The `UNCOVERED` note still matched after other people's edits moved those lines, because it names branches **by message rather than by number**.

- **No duplication this time.** Before proposing a "regression command", I checked `t27c gates` and `t27c battery`. `gates` runs every `check_*.py` and reports each child's own exit code, and separates scripts that cannot fail; `battery` is for document reconstruction oracles. Neither runs the negative controls, which is what `tri gates sweep` and `tri gates mutate` add. Complementary, not overlapping — the first time in four iterations that the §21 check came back clean.

- **The brief.** Eight decisions, grouped: repository settings (#2455, #2474), reproducibility records (#2477), language and corpus (#2479 and its link to the parse backlog), and four small measured defects (#2476, #2483, #2466, #2472). Two of them — #2476 and #2483 — need the stage0 freeze lifted, which is why they are filed rather than fixed.

- **The one connection worth reading twice.** `coverage` cannot be moved by bookkeeping *or* by one compiler fix. Re-sealing the 18 stale twins moved the count by zero because 14 of them do not parse, and `t27c backlog` reports zero specs at depth 1 with 94% four or more defect classes deep. It is a multi-month body of work or a decision to call it accepted debt — and those are the only two honest options.

- **Seven claims published and then refuted by my own measurement** over this campaign, every correction appended to the issue that carried the claim. The brief says so in its footer, because a decision document whose author has been wrong seven times should say how often and where.

Refs #2455, #2466, #2472, #2474, #2476, #2477, #2479, #2483

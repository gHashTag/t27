# NOW -- A new rule nested inside an old filter inherits its population (2026-09-03)

## Seven figures re-read, three moved, and all three were queries (Refs #2994)

- seven headline figures published in this session were read again hours later: **three had moved**, and all three stand over a population that is a QUERY rather than a set -- `4/33 -> 3/37` (the last 20 commits), `121/40/116 -> 126/44/121` (the skill grows), `288 -> 287` (the live backlog). The three over files on disk did not move at all
- the claim survives and the number does not: the local vocabulary really does see a fraction of what the gates accept. Re-measuring such a figure is not a second reading of one population, it is a first reading of a different one
- `tri skill claims --windowed` separates them. **14 of 422** sections describe a windowed population, and two of the fourteen are the sections written today -- the count moved because writing it moved the population, which is why the reading is anchored to `c039ebebe` rather than dated "currently"

## The placement was the finding, and a hand count caught it (Refs #2994)

- the new check was first written after the existing filter that keeps sections whose HEADING states a figure. That reported **4 of 126**; a hand count over every section said **12 of 420**
- neither number was an error -- they answer different questions, and only one is the question the rule asks. The section the filter dropped was **§179, whose title IS the rule**: *"A `--limit` on a run list is a time window in disguise"*
- **a disagreement between a hand count and a fresh matcher is a population question first and a logic question second.** The command now prints `14 (of ALL 422 sections, not of the 126)` so neither count can stand alone
- the test that would have caught it needs no corpus: it asserts the two rules disagree on one string, and says so if that heading ever gains a digit

## The anchor rule is an upper bound and says so (Refs #2994)

- `names_its_anchor` asks "is there a revision or an ISO date anywhere", which is necessary but not sufficient. Over the original twelve it reported **3**; all three were read by hand and **one does not survive**
- §125 says *"checks have not fired since 2026-08-24 11:06"* -- a date anchoring the CLAIM, while the window it read was "the last 10, then 60 runs", dated nowhere. A section the rule rejects is definitely unanchored; one it accepts merely might be. No lower bound is claimed
- a looser matcher keyed on the word **today** fires on **28 further sections** stating no window at all -- excluded as a matcher describing its input, with the count printed rather than dropped

## Nine clauses mutation-killed, three clauses deleted after being priced (Refs #2994)

- four clauses of `window_markers` and five of the date shape were each mutated and each killed a test. The `ANCHOR NOT FOUND` guard fired once on a literal `\n` that never expanded -- the mutant was refused rather than scored as a pass
- three clauses survived mutation AND moved nothing on the corpus: a label de-duplication the caller cannot observe, a left word-boundary check, and one character position of the date pattern. The de-dup and the boundary were **deleted**; the character position was the wrong unit -- mutating one conjunct of a ten-conjunct shape asks a question no natural input answers, so the shape was extracted as `is_iso_date` and tested as ONE rule
- `cargo test -p tri window_tests::,iso_date_tests::` reported `0 passed; 489 filtered out`. The filter is a **substring, not an alternation** -- the two modules were run separately with an empty-sample guard

## Anomaly: the disk filled and every recovery tool needed to write first (Refs #2994)

- `ENOSPC` on both the session temp volume and `~/.claude`: Bash could not create its output file, so it failed **before running the command**, and `Write` failed on its own temp file. `rm`, `du` and `df` were all unavailable for three attempts
- cause, measured after the fan-out died and freed a little: **37 git worktrees** of this repo, sixteen of them in one sibling session with a `target/` each -- 31 G there and 21 G here, on a volume with 1.4 G free
- recovery was to delete build output only (`t27w/target`, a throwaway crate's `target`), 3.2 G, and to work in the already-built checkout instead of adding a seventh tree. The 27 modified files on branch `w801` in that tree were **not touched**
- the loop creates worktrees and never reclaims them, and no step asks how much room is left. Named here rather than guarded, because the guard's population cannot be measured from inside a full disk

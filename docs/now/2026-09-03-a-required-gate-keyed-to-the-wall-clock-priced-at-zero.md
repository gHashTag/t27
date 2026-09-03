# NOW -- A required gate keyed to the wall clock, priced at zero (2026-09-03)

## The window is real and it is in a required check (Refs #2994)

- `scripts/ci/now-sync-gate-diff.sh` reads the date out of an entry's FILENAME and requires it to fall in `[yesterday .. tomorrow]` UTC, **computed from the clock at run time** (lines 79-81), failing with `exit 1` at line 129
- the entry's date is frozen at commit time and the window moves every midnight, so a branch whose gate runs on two different UTC days can flip green to red **with no change to the branch**. The remedy is to re-date a file, which teaches nothing
- the job is `check-now-freshness`, named in `seal-staleness-warn.yml:9` as one of four required contexts. Confirmed by two independent readings of the source

## It was priced, and the price is zero -- but the control matters more (Refs #2994)

- over the 100 most recent runs of this workflow, **2026-08-30 .. 2026-09-03**: 5 failures, and **not one from the window** -- three "adds no docs/now/ entry", two "entry has no bullet". Every failure was the gate working
- **zero is not evidence the window is safe.** The defect needs a branch whose gate ran on two different calendar days, and of the **36** branches in that set only `master` did. Thirty-five PR branches opened and merged inside a single day
- so the cost is zero because nothing here stays open overnight, **not because the baseline is sound**. The first PR that waits for review across midnight pays it
- the gate is therefore **not changed**. Editing a required check to remove a defect measured at zero, unasked, is a larger risk than the defect: every open PR depends on it

## A second defect in the same place, and this one is free (Refs #2994)

- the names are crossed. `now-sync-gate.yml` holds the job `check-now-freshness` and IS the freshness check; `check-now-freshness.yml` holds a job called `check` and does NOT check freshness -- it runs `tools/check_now_entry_shape.py`, which reads SHAPE
- both jobs are required, so neither file can be renamed without renaming a required check
- a reader who matches file name to subject opens the wrong file, finds a shape checker, and concludes the freshness gate does not run. **That has already happened, in this loop's own notes.** Both files now say so in their first ten lines, which costs nothing and is the whole fix available

# NOW -- Never succeeded, and never executed, are different facts (2026-09-04)

## `gates dead` printed two populations as one row (Refs #2994)

- of this repository's three dead workflows, **two never executed at all**: `auto-merge-ready-prs.yml` (1541 runs) and `format-check.yml` (31), with **0 jobs in 8 of 8 sampled runs** each. `coq-proofs.yml` (62 runs) is the control -- 1 job every time, so it really ran and really failed
- a run allocating **zero jobs** is a startup failure: invalid YAML, a trigger the file does not declare, a registration for a file that is gone. `auto-merge-ready-prs.yml` declares `workflow_dispatch` only and every sampled run has `event=push`
- **the two want opposite repairs** -- a broken workflow FILE against a broken CHECK -- and "never succeeded" printed them identically
- cost measured: **109 s -> 114 s**, because the probe runs only for the rows the report prints. `None` when nothing was sampled: a probe that saw no run must not vote
- the new site reads as *asks whether the page filled* and is **not**: `classify_fetch` matches `total_count` anywhere in the body, and here it is a **jq path**. It is really a DECLARED SAMPLE, a bucket that does not exist. Named rather than special-cased

## And my own resolver committed conflict markers (Refs #2994)

- the required `Conflict markers` check went red naming `tools/census/fetches.txt`, lines 19 and 35. The commit is my landing loop's resolver: it fixes the skill file, then runs `git add -A` -- and the merge had conflicted on a **second** file, a generated ledger, which was staged verbatim
- **control first:** `verify_all_152.py` carries **16** markers on master and that gate is green there, five successes the same day, so the failure could not be the known debt
- **a generated ledger is never hand-merged** -- regenerate from the merged tree and let the gate confirm
- **after resolving, ask the repository's own checker:** `python3 tools/check_conflict_markers.py` exits 0 and says *"Every marker found is recorded as debt. Nothing new."* Third time this pass the tool was already in the tree

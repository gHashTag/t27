# NOW -- a guard that cannot run, on the path that merges (2026-09-05)

## a guard that cannot run, on the path that merges (Refs #3195)

- tri pr ready --required-only --wait printed p of total where p counts required check RUNS on the head commit and total was req.len(), the count of required context NAMES from repository settings, which never look at the commit
- required_pending returns None for an empty ruleset and the caller bails, so total is at least 1 on every path reaching the print -- which makes the total == 0 arm unreachable in that mode. That arm is the documented guard for an empty list is not finished it is not started, whose rationale records a pull request merged while ten checks ran
- so running the flag seconds after a push, before any required context has posted a run, gives p=0 honestly and total=4 from settings; control breaks on the FIRST poll and the verdict reads safe to merge into gh pr merge --squash. The wait exits before the checks it exists to wait for exist
- fixed by taking the denominator from the same read as the numerator: required_posted counts how many required NAMES have posted a run on this commit, so it can be zero, and the arm now also covers the partial case because zero pending of three posted says nothing about the fourth. The ruleset size is still printed as its own clause

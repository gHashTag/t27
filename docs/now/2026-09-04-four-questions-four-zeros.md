# NOW -- Four questions asked, four zeros, and the zeros cost less than the sweeps (2026-09-04)

## What this pass looked at and did NOT find (Refs #2994)

- **other budgets under their measured cost:** none. `required`/`quiet`/`fetches` have **45×** headroom, `unmeasured` **3.6×**, and the tightest -- `git log --all` with a glob, budget 30 s -- costs **1165 ms** over 6687 commits (exact path 169 ms). Its `except` is annotated *"cannot tell: assume the milder classification"*, the honest failure the `dead` budget lacked. **1 hit in 6**
- **census refusals:** `tri gates quiet --list --excluded` refuses **123** steps; a systematic sample of ten read one at a time is **10 of 10 correct**. The one candidate, `total_files=$(ls …/*.v | wc -l)` with no `2>/dev/null`, is correct on a second reading -- that value goes **only** into `$GITHUB_STEP_SUMMARY` and nothing branches on it
- the sharper rule from it: **a count reads zero when its subject is missing either way; what makes it QUIET is not the shape but whether it has a consumer.** `targets=$(grep -c … || true)` is the same shape and IS guarded downstream, where `test_ratchet.py` refuses on `targets == 0`
- **a green mirror of "never executed":** **0 of 30** successful master runs allocate zero jobs, and **0 of 20** have every job `skipped`; all twenty carry at least one `success`, one showing `2 skipped, 2 success`
- **the instrument nearly lied by silence:** the first `jq` had a misplaced `as` binding and returned **nothing**, which reads as *there are none*. The control was one command -- print raw `.conclusion` for one known run. **An empty result is not a finding until the same expression prints something on a known case**
- **what chasing the base costs:** on one pull request, **2 commits landed on master while it was open and the poller caught up exactly twice**, one full round of **35** checks each. The rule "at most one reset per green window" holds and the window RECURS -- one reset per neighbouring landing

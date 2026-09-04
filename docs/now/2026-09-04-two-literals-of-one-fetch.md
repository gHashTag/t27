# NOW -- Two literals of one fetch (2026-09-04)

## The last two page-as-census sites that were actually defects

- `tri gates fetches` flagged **5** sites printing what they got as though it were a total. Two
  were in `prcheck.rs::ready` (#3158). Two more were the *same fetch written twice*:
  `unmeasured` and `dead` both asked `repos/{repo}/actions/workflows?per_page=100` and read
  whatever came back.
- **63 active workflows here, so the page does not bind today** -- and would start binding
  silently at 101. That is the whole shape: a bound that is invisible until the day it is wrong.
- Both now go through one `workflow_listing(repo, jq)` using `--paginate`, which moves them into
  that command's own *complete by construction* category. **Two literals of one fetch are how the
  two commands would have drifted apart** -- and the count is now **3**, with `unmeasured`
  answering `1 of 50` before and after.
- The remaining three are #3158's pair and `red.rs::runs_url`, which is a **false positive the
  tool explains itself**: `PAGE = 30`, `is_lower_bound(n) = n >= PAGE`, and the streak prints as
  `30+`. The function only builds a URL; the guard is in the caller, and the tool says a function
  of that shape "reads as unguarded".

## And my own gate was red for a shell reason

`#3156` failed with `set: Illegal option -o pipefail`. The coq-kernel job runs in the
`coqorg/coq` container where `/bin/sh` is **dash**, and every other step in that file already says
`set -eux` for exactly that reason. I copied the idiom from `corpus-ratchet.yml`, which has no
container and runs under bash. **Skill section 497 records this same shell from the other
direction** -- probing in bash when CI runs dash -- and I met it head-on rather than learning it
twice. Verified by running the step body under `/bin/dash` directly.

Refs #3157

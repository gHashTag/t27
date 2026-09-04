# NOW -- Keeping the branch fresh is what kept it from landing (2026-09-04)

## A livelock built out of hygiene (Refs #2994)

- the landing loop merged the base whenever it was no longer an ancestor. Pushing **restarts every check**, and neighbouring sessions land pull requests faster than checks finish
- observed one iteration apart: `8: UNSTABLE waiting:2` → `12: caught up` → `13: BLOCKED waiting:22`. **The catch-up reset the progress it was performed to protect**
- **being behind costs nothing until the moment of merge.** Merge the base only when checks are green **and** `mergeStateStatus` is `BEHIND` -- when being behind is the sole remaining blocker
- **the rule minimises the resets; it does not remove them.** Merging requires being up to date, and that restarts the checks -- so a reset is unavoidable when master moves inside the final green window. The eager rule paid one per move at any time; the narrow rule pays at most one per green window. #3160 landed on iteration 24 with **zero** catch-ups because master happened not to move; the next pull request under the same rule went `BEHIND waiting:0` -> caught up -> `BLOCKED waiting:22`. **Zero was luck, at most one is the guarantee**
- wider shape: an action taken "to stay current" is paid for in the currency of the thing it protects. Ask what one refresh costs and **at what moment staleness actually blocks** -- often only at the end, and only once

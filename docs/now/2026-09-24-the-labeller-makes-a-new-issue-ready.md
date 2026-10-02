# NOW -- the labeller makes a new issue ready, not merely labelled (2026-09-24)

## Labelling is not the point; readiness is (Closes #4769)

- `needs-boundary.yml` ran daily and stopped at a **label**. A new issue that named its own files sat unreachable until somebody noticed and ran `propose_boundary.py` by hand, and the swarm idled with lanes free while it waited. The gap was a day, every day.
- The daily job now applies the drafting rules to exactly the issues it just labelled and writes the result into the **BODY** -- the only place the Queen reads. A comment leaves an issue as unreachable as the label did. Rules are IMPORTED from `propose_boundary`, never repeated: a second copy is a second thing to drift.
- Proved end to end on a live issue rather than asserted: #4768 was opened naming `bootstrap/src/compiler.rs` and no boundary; one run labelled it, wrote `## Boundary`, and cleared the label. Closed afterwards.
- A `docs/` citation is still never written into a body -- a body line is a claim on a file, and a document an issue quotes is the claim most likely to be wrong. Those get a comment and wait for a person.
- One thing that failed silently and would have again: `actions/checkout` fetches a single commit and no branch ref, so `origin/master` does not exist inside a workflow. Every tree read returned empty, which is not an error anywhere -- just a rule matching nothing. `tree_ref()` falls back to `HEAD`.
- **55 `Wave Loop` issues closed as records rather than work.** They open with `[DONE]` or `[VERDICT]`, are dated 2026-07-01 to 2026-08-09, and name no files because a report owns none. Left open they counted as backlog the Queen re-read every round.
- `incompleteSpec` is NOT a blocker, which the numbers made it look like: queend appends the shortfall and carries on (`main.swift`: "refusing outright would stall the swarm on paperwork"). Those 195 issues are dispatchable; the count is a to-do list about task quality.
- Board: **525 open, 182 without a boundary** (yesterday: 649 and 565).

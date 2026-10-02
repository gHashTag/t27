# NOW -- The workflow I added was classified by nobody (2026-09-20)

## queen-feed-untested.yml joins the not-merge-critical list (Closes #4319)

- `gate-topology` failed on every open pull request with `UNCLASSIFIED ROSE 24 -> 25`. The twenty-fifth is `queen-feed-untested.yml`, merged an hour earlier in #4307, and the gate exists precisely so that a workflow cannot arrive unread.
- It is not merge-critical for the same reason its sibling is not: it opens issues on a schedule, and its own failure gates no merge. Classifying it returns the unclassified count to the ceiling of 24 rather than raising the ceiling, which is the whole point of a ceiling that only moves down.
- Second self-inflicted break in one day from the same cause: a new workflow moves ledgers that nothing in the PR reminds you about. The census (`tools/census/*.txt`) was the other. Both are now in the skill as one rule: **a PR that adds a workflow re-blesses the census AND classifies it in `check_pr_branch_filters.py`, in the same commit.**

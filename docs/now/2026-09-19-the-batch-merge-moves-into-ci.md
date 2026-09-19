# NOW -- The batch merge moves into CI (2026-09-19)

## `Auto Merge Ready PRs` runs every 20 minutes instead of waiting for a laptop (Closes #4278)

- The batch merge ran as a local scheduled task. Every run since 2026-09-17 07:39 UTC failed in about three seconds, on the model the session asked for, and the failure was invisible: the script logs when it runs, and it never ran. Thirty-three bee pull requests sat open; once the CI queue was unblocked by hand, 52 merged in a day.
- The workflow that does this job already exists and has only ever been `workflow_dispatch`. It now carries `schedule: */20 * * * *` and a `concurrency` group, so two batches cannot race over the same pull request. What it merges is unchanged: an issue reference in the title or body, and every check green.
- Twenty minutes rather than five, because each merge puts the remaining branches behind master and their checks re-run.
- The classification note in `scripts/ci/check_pr_branch_filters.py` said "auto-merge is disabled by policy in this repo", which stopped being true some time ago: pull requests have been landing through GitHub's own auto-merge all week. It now says what is true - the workflow merges on a schedule, and its own failure gates nothing.

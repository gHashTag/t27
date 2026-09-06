# NOW -- Zero checks is not zero failures, on the path that merges (2026-09-06)

## Zero checks is not zero failures, on the path that merges (Refs #3248)

- auto-merge-ready-prs.yml computed FAILING over the rollup and treated 0 as clean. An absent rollup and a present-but-empty one both give 0, so a PR on which nothing had run printed 'Ready to merge'. It now refuses when no check has posted; a still-running check was already handled correctly, since null != SUCCESS in jq.
- Its merge loop ended '|| echo Failed to merge', so the step exited 0 whether every merge succeeded or every one failed. Now counts both and exits 1 if any failed: a batch that merged nothing must not report green.
- pr-dashboard.yml counted READY as all(SUCCESS or SKIPPED or null), which is true of a PR with no checks and of one still running - the same PR appearing in both READY and PENDING. READY now requires at least one posted check, 'no checks yet' has its own column, and the table says so when the four columns stop summing to TOTAL.

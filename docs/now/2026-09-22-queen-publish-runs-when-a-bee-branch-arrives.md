# NOW -- Queen Publish runs when a bee branch arrives (2026-09-22)

## The publisher waited on a schedule GitHub ran four times a day (Closes #4598)

- `queen-publish.yml` now also runs on `push` to `queen-*`: a bee branch arriving is when there is work to publish. The `9,39 * * * *` schedule stays as the backstop.
- Measured 2026-09-21/22: the schedule ran at 14:31, 19:43, 23:14 and 01:50 UTC, then not for four hours, publishing at most 5 per run. 42 accepted issues waited without a pull request while the swarm ran 2 bees of 20; a manual run with `--limit 40` published 20 and 18 merged within half an hour.
- The per-run limit goes from 5 to 10. It stays bounded because every pull request starts twenty-odd workflow runs.
- The publisher's own docs/now commit re-triggers the workflow once; that run finds the pull request open and does nothing. The existing `concurrency` group collapses bursts.

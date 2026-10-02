# NOW -- The bees' work leaves the container in the cloud (2026-09-23)

## The push that was a laptop is now a workflow (Closes #4601)

- `tools/queen/export_push.py` reads `/queen/export`, fetches each branch as a git bundle and pushes it; `.github/workflows/queen-export-push.yml` runs it every 20 minutes and on demand.
- Measured 2026-09-23: the last bee branch reached GitHub at 10:33 on 09-22, 59 open issues held an accepted verdict whose work was still in the container, and the swarm sat at 0 of 20 lanes. A publisher dry run found nothing left to publish: `no commits=75, already a PR=342, issue not open=137, published=0`.
- It never force-pushes: a remote ahead of the bundle is a bee's work, and that is a person's decision. A bundle that does not verify, or whose prerequisites are missing, is skipped with the reason.
- It needs the repository secret `QUEEN_EXPORT_TOKEN` (the agent server's `TRIOS_API_TOKEN`). Without it the run prints a warning naming the secret rather than passing quietly.
- `queen-publish.yml` already wakes on a push to `queen-*`, so the pull request follows from the push this job makes.

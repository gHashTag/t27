# NOW -- A probe run is not a health signal (2026-10-03)

## What this PR adds (Refs #3559)

- New spec `specs/automation/inngest-functions-status.t27`: the contract of `GET /api/inngest/functions/status`, read by the FUNCTIONS tab on t27.ai. `lastRun` is the last organic run (event or cron); invoked runs (probes, the Invoke button) go to a separate `lastProbe` field with the manifest's expectation, and a function with no organic run is `muted`, neither red nor green.
- `specs/automation/inngest-probe-suite.t27` moves to VERSION 2: `LIVE_RUN_STATUS` lists the three recorded production runs, and `PROBE_RUN_IS_HEALTH_SIGNAL=false` and `PROBE_FAILURE_ALERTS_ADMIN=false` are stated.

## Why

- A production read on 2026-09-12 showed 20 of 28 functions with a FAILED `lastRun`, all of them probes designed to fail on a guard; the tab painted those as red.

## Not in this PR

- The host and website implementations, which are separate pull requests.

## Re-dated

- This entry was added on 2026-10-03 for re-review. The PR was opened on 2026-09-13 without a docs/now entry; the text above restates its description and claims nothing new.

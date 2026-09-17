# NOW -- The sweep gets one step per seller (2026-09-17)

## What was read

- The self-hosted Inngest dashboard, Status=Failed, last 7 days: three runs, all `CRM: proactive seller sweep`, all on 2026-09-16, durations 15 s, 2m02s and exactly 5m00s.
- All three: "Your server returned HTTP 502 before the SDK responded. No step output was produced before the request failed."
- The function was one `step.run('sweep')`: one HTTP request from the Inngest server to the bot carrying every connected seller in turn, each allowed up to INGEST_TIMEOUT_MS = 170 s on the render.

## What this spec says

- `specs/functions/crm-proactive-sweep.t27` is new; the function had no spec since it was born on 2026-09-12.
- STEPS = `resolve-sellers`, `sweep-${owner}`: one request per seller, a finished seller memoized inside the run, the trace names the seller a 502 lands on.
- RETRIES stays 0: a retry re-runs the interrupted step and may push a second card to a seller.
- DOMAIN `crm` is new to the corpus; README's DOMAIN vocabulary gains it.

## Not verified

- Which seller each 502 landed on: a single step left no per-seller trace.
- Whether the 15 s and 2m02s failures were a redeploy under the request or an app crash; Railway logs were not read.
- The exact Railway edge request budget; 5m00s is the observed cut.

## Where the code goes

- 999-multibots-telegraf, branch `sweep-per-seller-steps`: `crmProactiveSweep.ts`, `functions.manifest.json`, test `crmProactive.sweepSteps.test.ts` citing this spec, lesson `docs/inngest/lessons/2026-09-17-one-http-request-per-seller.md`.

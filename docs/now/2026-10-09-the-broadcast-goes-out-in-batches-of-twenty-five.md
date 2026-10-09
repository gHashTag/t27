# NOW -- The broadcast goes out in batches of twenty-five (2026-10-09)

## What was read

- broadcast-message-send carried the whole audience in one step.run('send-messages'). A retry after a mid-way failure re-sent to everyone already served; one slow Telegram answer could push the step past the HTTP budget.
- The service re-queried the users table inside the send, ignoring the list the fetch-users step had already produced.
- A Telegram 429 was logged like any other error; its retry_after was never read.

## What the spec says now

- STEPS: send-batch-${firstId} per 25 recipients, pause-after-${firstId} (1s) between batches, then analyze-results.
- concurrency 1 keyed on event.data.bot_name.
- The service receives the batch as recipients and does not re-query; on 429 it waits retry_after (capped at 60s) once and counts the recipient as an error, so a re-run of a batch step cannot double-send.

## Not verified

- No live broadcast on the deployed build.

## Where the code goes

- 999-multibots-telegraf, branch `broadcast-batches`: broadcastMessage.ts, plan_b/broadcast.service.ts, functions.manifest.json, test inngest/broadcastBatches.test.ts citing this spec.

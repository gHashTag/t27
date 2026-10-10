# NOW -- The token stays in the environment (2026-10-09)

## What was read

- reels-loop-generate took bot_token from the event and built a Telegraf from it in send-to-telegram.
- content-scripts-generate accepted openai_api_key in the event schema and preferred it over OPENAI_API_KEY.
- Inngest stores the event with the run and shows it in the UI and traces; a secret in the payload is a secret in the dashboard.

## What the specs say now

- reels-loop-generate: the bot is resolved from bot_name through the bot registry; bot_token is a deprecated fallback with a warning that does not print it; neither is NonRetriableError.
- content-scripts-generate: openai_api_key is gone from the schema; zod strips it; the key comes from env inside the steps.

## Not verified

- The sender of reels/loop.generate is outside the repo; whether it already sends bot_name is unknown.
- Whether the Inngest dashboard is reachable without login (payloads readable by anyone) was not checked.

## Where the code goes

- 999-multibots-telegraf, branch `secrets-out-of-payloads`: generateAdvancedLoopingVideoFunction.ts (resolveBotForDelivery), generateContentScripts.ts, functions.manifest.json, test inngest/secretsOutOfPayloads.test.ts citing these specs.

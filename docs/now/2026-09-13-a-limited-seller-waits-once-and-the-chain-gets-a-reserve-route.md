# NOW -- A limited seller waits once, and the chain gets a reserve route (2026-09-13)

## What the control dry duet showed after #3613

- duet-mu00klri (dry, 4 turns, 23:14 +07, on the #2398 build): the seller called five free tools, then every provider answered with a limit -- z.ai 429 twice (glm-5.3 and glm-4.5 share one subscription), NVIDIA "Worker local total request limit reached (16/16)". No text. The run is stored as failed with the turn error and the dashboard shows the failed badge and lines 1/8: FAILED_STATE_SEEN_LIVE = true.
- The claim check of #3613 did not fire because the seller never spoke: CLAIM_CHECK_SEEN_LIVE stays false; unit tests are its only witness.
- Third run of the day on the same wall (duet-mtzyg2t6 turn 2, the live business bot on 2026-09-06, now this). Two of three paid routes lead to one z.ai limit; the third has its own.

## What the specs now say

- crm-duet.t27: SELLER_RETRY_ON_LIMIT = 1, SELLER_RETRY_PAUSE_MS = 30000. A seller turn that ends with a LIMIT error (LIMIT_MARKERS, plus the Russian 429 diagnose line) and no text waits once and replays from the same history. RETRY_ONLY_WITHOUT_PAID_CALL: an attempt that paid for a generation is never replayed. RETRY_NOTED_IN_TRANSCRIPT: the line keeps `retried: "<first error>"`. Other errors (bad key, no balance, unknown model) are not retried; fn retry_allowed.
- agent-provider-chain.t27 (new): ORDER_DEFAULT = "zai,zai-lite,nemotron,reserve,ollama". The `reserve` id exists only when RESERVE_BASE_URL, RESERVE_API_KEY and RESERVE_MODEL are all set; RESERVE_TOOLS = "0" excludes it from tools-only turns, RESERVE_VISION = "1" declares image input. RESERVE_CAPABILITIES_DECLARED_NOT_MEASURED: the catalog entry says so; RESERVE_IN_BOT_PICKER so the owner can move it to the front.

## Not claimed

- RETRY_SEEN_LIVE = false, RESERVE_CONFIGURED_LIVE = false, RESERVE_ANSWERED_LIVE = false, DUET_RAN_LIVE = false. The owner sets the Railway variables; this repository never reads them.

## Bookkeeping

- Four test blocks added (one in crm-duet.t27, three in agent-provider-chain.t27): published_figures pin 12752 -> 12756.
- Host change: 999-multibots-telegraf crm-duet-tool.ts (retry), provider.ts and provider-choice.ts (reserve id).

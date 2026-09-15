# NOW -- The reserve route is there, and a skipped provider will say why (2026-09-15)

## What is true now

- RESERVE_BASE_URL, RESERVE_MODEL and RESERVE_API_KEY are set on vibee-render (Railway); the key is a reference to the bot service's DeepSeek key, never copied.
- GET /api/agent/provider on commit e2c4617 lists the live chain zai, zai-lite, nemotron, reserve(deepseek-chat). The spec `automation/agent-provider-chain.t27` now carries RESERVE_CONFIGURED_LIVE=true with that listing as the evidence.
- RESERVE_ANSWERED_LIVE stays false: with reserve moved first, one ephemeral tools_only turn was answered by zai. The reserve was skipped, and nothing in the log says why.

## The control dry duet

- duet-mu2qj57q (dry run, 4 turns, 8/8 lines, state done) on the same commit: seller lines 0 and 2 called only free tools and asked one question each; the one paid call (reel_render) came on line 4, after the buyer asked for an example. `crm-duet.t27` now carries DISCOVERY_GATE_SEEN_LIVE=true, FAMILIARITY_REWRITE_SEEN_LIVE=true (line 0 was rewritten once) and SELLER_GENDER_FIXED_SEEN_LIVE=true.
- Open debt from the same run: line 4 says the video is already in the chat while a dry run sends nothing (media_sent 0). Recorded as DRY_RUN_DELIVERY_CLAIM_SEEN=true, FIXED=false. One voice flag on line 6 (a stars sum after one rewrite) went out and is reported.

## What changed in the spec

- New rule SKIPPED_PROVIDER_IS_LOGGED with SKIP_LOG_PREFIX "[agent] provider skipped:" -- every fall-through in streamModel (HTTP error, thrown fetch, timeout) becomes one warn line with the diagnose() reason before the next provider is tried.
- RESERVE_PROBES=1 and RESERVE_PROBE_ANSWERED_BY="zai" record the probe instead of a verdict.

## Why

- The chain collected reasons only for the final "nobody answered" error. When a later provider succeeds the reasons vanish, so a dead route stays invisible as long as the others live -- exactly the failure the reserve exists to prevent.

## Not claimed

- Whether DeepSeek answers through this route: unknown until the host logs the skip and the probe is repeated.
- No provider event with id "reserve" has been observed.

## Next

- Host: `999-multibots-telegraf` chat.ts logs each skip; repeat the probe; read the reason; then either fix the route or record RESERVE_ANSWERED_LIVE=true from a provider event.

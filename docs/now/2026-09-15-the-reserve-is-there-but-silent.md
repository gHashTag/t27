# NOW -- The reserve route is there, and a skipped provider will say why (2026-09-15)

## What is true now

- RESERVE_BASE_URL, RESERVE_MODEL and RESERVE_API_KEY are set on vibee-render (Railway); the key is a reference to the bot service's DeepSeek key, never copied.
- GET /api/agent/provider on commit e2c4617 lists the live chain zai, zai-lite, nemotron, reserve(deepseek-chat). The spec `automation/agent-provider-chain.t27` now carries RESERVE_CONFIGURED_LIVE=true with that listing as the evidence.
- RESERVE_ANSWERED_LIVE stays false: with reserve moved first, one ephemeral tools_only turn was answered by zai. The reserve was skipped, and nothing in the log says why.

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

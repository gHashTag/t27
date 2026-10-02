# NOW -- The reserve asked for more than the key could pay (2026-09-15)

## What happened

- Probe 2 of the reserve route, vibee-render a587fee with #2410's skip logging live: the reserve (DeepSeek) was skipped and the log said why for the first time -- HTTP 401, key invalid.
- The owner approved moving RESERVE_* to OpenRouter (deepseek/deepseek-chat, key referenced from the bot service); the service redeployed and GET /api/agent/provider listed the new model.
- Probe 3: skipped again, this time HTTP 402 -- "requires more credits ... requested up to 16000 tokens, but can only afford 2506". streamModel sends no max_tokens, so the vendor priced the model ceiling against a small prepaid balance.

## What the spec now says

- RESERVE_SENDS_MAX_TOKENS: the reserve route always sends a bounded max_tokens; RESERVE_MAX_TOKENS overrides, default 1024. Other providers are unchanged.
- PAYMENT_REQUIRED_IS_DIAGNOSED: HTTP 402 or "requires more credits" reads as "no funds on the key" regardless of body wording.
- Live evidence: RESERVE_PROBES = 3, probe 2 status 401, probe 3 status 402 with the two token figures. RESERVE_ANSWERED_LIVE stays false; every probe was answered by zai.
- One test block added; the published-figures pin moves 12758 -> 12759.

## What is not claimed

- No answer from the reserve has been observed live.
- Whether 1024 fits the balance after the prompt is priced in is a guess from one vendor message, not a measurement.

## Next

- Host change in 999-multibots-telegraf: reserveEnv reads RESERVE_MAX_TOKENS, streamModel sends it for the reserve, diagnose names 402; tests cite this spec.
- Redeploy, probe 4, read the skip line; the spec follows the result either way.

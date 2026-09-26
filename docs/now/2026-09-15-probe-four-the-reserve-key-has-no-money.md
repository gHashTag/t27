# NOW -- Probe four: the reserve key has no money (2026-09-15)

## What happened

- vibee-render 9d6d4aa (#2413) went live: the reserve request now carries max_tokens 1024 and diagnose() names 402 as "no funds on the key".
- Probe 4, same protocol as before (reserve first, one ephemeral tools-only turn, zai restored): the provider event named zai; the log line reads "reserve: no funds on the key -- top up the balance". HTTP 402 with the ceiling bounded.
- Reading: the prepaid OpenRouter balance does not cover prompt plus 1024 tokens for a tools-only turn, whose tool schemas alone run to thousands of tokens.

## What the spec now says

- RESERVE_PROBES = 4, RESERVE_PROBE_4_STATUS = 402, RESERVE_PROBE_4_MAX_TOKENS_SENT = 1024.
- RESERVE_BLOCKER_IS_KEY_BALANCE: the next change is money on the key or another key, not a commit.
- RESERVE_ANSWERED_LIVE stays false.

## What is not claimed

- The exact balance was not read; the vendor's number 2506 is from probe 3 and the prompt size was not measured.
- Whether DeepSeek's own key is revocable or merely stale was not checked (probe 2 said 401).

## Next

- Owner: top up OpenRouter or point RESERVE_* at a key with balance; probe 5 then decides RESERVE_ANSWERED_LIVE.

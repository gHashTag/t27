# NOW -- crm-duet v2 -- braced tests over fns the host mirrors; two drifts found by binding (2026-09-27)

## crm-duet v2 -- braced tests over fns the host mirrors; two drifts found by binding (Closes #4889)

- Twelve braced tests over speaker_at, paid_tool_offered, claim_hit_counts, claim_is_honest, retry_allowed, state_after, business_bot_silent; roles instead of two real people's ids.
- Drift 1: the host's negation window reached three words back; it now reads NEGATION_WINDOW_WORDS from here. Drift 2: the brief's soft paid cap is two per run (portrait + reel), PAID_PER_RUN_SOFT = 2.
- state_after gained the 6-of-8 vector: the old vectors could not tell turns * 2 from turns. 12/12, 0 vacuous, negative controls caught; bound in 999-multibots-telegraf src/spec/crm-duet.test.ts.

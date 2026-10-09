# NOW -- spec(automation): tri-wallet-chains v2 -- a budget on the two doors without a person (2026-10-09)

## spec(automation): tri-wallet-chains v2 -- a budget on the two doors without a person (Closes #8315)

- /api/tri/wallet/message and /propose answer anybody: each caller address gets 30 calls a minute, counted in memory only and bounded to 10000 callers; past it 429 and nothing is checked.
- /propose looks the nonce up before it checks a signature (checks_signature), so a call on a nonce nobody was issued costs one key lookup and no curve arithmetic. From the review on 999#3952.

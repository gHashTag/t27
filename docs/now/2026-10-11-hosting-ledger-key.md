# NOW -- specs/hosting/statement.t27: the one ledger key variable (2026-10-11)

## specs/hosting/statement.t27 section 6 (Closes #8759; gHashTag/trios#1761)

- The Queen signs epoch statements with the Ed25519 seed in ONE deployment variable, `TRIOS_HOSTING_LEDGER_KEY` (64 lowercase hex characters). Missing or malformed: every epoch stays open and unsigned, one log line names the variable and never its value, and no key is generated in its place.
- `t27c test-report`: 10 tests, 0 FAIL, 0 vacuous, 91 runtime asserts, 4 invariants. 4 negative controls, each FAIL `without_a_ledger_key_epochs_stay_open`.

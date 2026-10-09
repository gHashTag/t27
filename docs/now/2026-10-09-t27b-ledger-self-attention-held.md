# NOW -- t27b ledger: self_attention.t27 held as blocked while its reference is host-dependent (2026-10-09)

## one row back in the ledger, the cap already covers it (Closes #8141)

- `docs/reports/t27b_expectations.json` gains `specs/ml/recurrent/self_attention.t27` as
  blocked `StmtAssign(undeclared)`, reason unimplemented -- the same row master had before #8059.
- Why it went missing: the bless in #8059 came from lab run 6b6386e15, whose reference fails the
  spec; the GitHub runner's reference passes it, so `t27b-native-ratchet` reported UNLISTED on
  every PR (run 37900879225). The test reads an undefined slice from the undeclared
  `default_input()`, so the verdict depends on the host (#8051).
- The row is held only until #8051 fixes the spec; then a bless drops or re-states it.
- Rows 1221 -> 1222 (unique, sorted, every path exists); pass 1063, pass_vacuous 132,
  non-pass 26 -> 27. `max_not_pass` stays 27: master stored 27 over 26 non-passing rows, and the
  held row makes the count equal the cap.

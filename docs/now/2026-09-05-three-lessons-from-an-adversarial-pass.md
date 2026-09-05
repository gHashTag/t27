# NOW -- Three lessons from an adversarial pass on my own gate (2026-09-05)

## Three lessons from an adversarial pass on my own gate (Refs #3296)

- Five attackers with distinct lenses and ten verifiers: 15 candidates, 6 survived
  refutation, 9 refuted. All six are fixed.
- Before the pass the gate had unit tests, three killed mutations, a 498-commit historical
  sweep with one refusal, and a two-reader agreement check with zero disagreements. It
  still had five defects.
- Every one of those controls asked whether the rule computed what it said. None asked
  whether what it said was the right thing to say.
- 587: a whitelist of code cannot be completed -- .xdc, .tcl, .toml, .lean and every
  extensionless path -- so ask whether the diff is prose instead, which is closed.
- 588: `on: pull_request` omits `edited`, so a title-reading gate prescribes a remedy
  that cannot re-trigger it; and a single-commit squash takes the COMMIT subject.
- 589: the hook picked `target/debug/tri` from 03:49 over `target/release/tri` from
  19:40 and printed PASSED for gates that binary did not carry.

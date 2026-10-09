# NOW -- t27b: the bless test is an own-language exception (2026-10-09)

## one entry so step 3 of verdict reuse can carry its test (Closes #8214)

- `tools/policy/foreign-exceptions.txt` lists `scripts/ci/test_a_t27b_spec_cannot_move_silently.py`.
- Step 3 (#8186) makes bless refuse a source without a ledger key on every row. The test builds
  keyless records, so it must change in the same PR as the rule, and the gate rejects any edit to it.
- The rules live in `specs/tri/t27b/steward.t27`; the test only drives the generated code.
  The 40-per-file and 80-per-PR budget still applies.

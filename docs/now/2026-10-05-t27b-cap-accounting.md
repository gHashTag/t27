# NOW -- t27b bless accounts for a cap rise; the doctor's LAB-ERROR reads per-spec records (2026-10-05)

## Cap rise accounting and alarm rules in steward.t27 (Closes #6237)

- `specs/tri/t27b/steward.t27` gains `cap_rise_is_new(not_pass, old_cap, new_not_pass)` (a rise is covered when the non-pass entries the old ledger already named still fit the old cap) and `is_alarm(t27b, reference)` (mismatch and crash always alarm; fail and timeout only where the reference passes). 62 tests: `t27b test --check` 62/62, t27c lab `test-report` 62/62.
- `gen/c/tri/t27b/steward.c` regenerated on the t27c lab (sha b46d92e084be); `tri t27b gen-check` answers SAME b46d92e084be.
- `tri t27b ratchet --bless` prints the accounting on a refused rise (named + new, entries that left the run) and says whether `--accept-new` would cover it. `--accept-new` lets the cap rise by exactly the new specs; a rise from a spec the ledger already named is still refused.
- Live against lab run 88862a949 and the 582 ledger: refused with "594 = 553 already named + 41 new"; with `--accept-new` the cap becomes 594, the number #6235 set by hand.
- The doctor's LAB-ERROR counts fail and timeout from `results` through `is_alarm` when the run has them, and falls back to the summary otherwise. Live: the `t27b_fail 1` alarm for `build_verify.t27`, which the reference fails too, is gone.
- CI: the steward test checks the new rules; the ratchet test checks the refused and accepted rises; the tick test adds a shared-fail fixture (no anomaly) and an own-fail control (LAB-ERROR t27b_fail 1).

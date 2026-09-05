# NOW -- competitors: a subtraction that was neither a subset nor pass@10 (2026-09-05)

## competitors: a subtraction that was neither a subset nor pass@10 (Refs #3195)

- the parenthetical "3 of them cite pass@10 only" was zero_at_1 minus cites_nothing; cites_nothing is not a subset of zero_at_1 because scores are Option, so a record omitting pass@1 and stating pass@10 zero decrements a difference it does not belong to and the usize subtraction can underflow and panic
- and the difference counts "cites something nonzero", not "cites pass@10" -- a record citing pass@5 alone was reported as citing pass@10 only. Counted directly now, as a free function rather than a field, because the ratchet carries five keys and this is not one of them
- the mutation harness reported KILLED for the call-site revert and was wrong: it replaced all four occurrences including the two in the test module, so the test panicked on its own mutated body. Restricted to code above the test module the mutant SURVIVES. A mutation that also edits the test is not a mutation test

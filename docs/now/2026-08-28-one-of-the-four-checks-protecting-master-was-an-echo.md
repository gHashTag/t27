# NOW -- One of the four checks protecting master was an echo (2026-08-28)

## One of the four checks protecting master was an echo (Refs #2754)

- t27-master-protection requires check-now-freshness, validate, check, check-linked-issue -- and the job named `check` ran one step: echo, with a comment saying add logic here in future
- it passed every pull request it ever ran on, and its green meant nothing
- the ruleset is not mine to edit, so the job keeps its required name and now asserts that the docs/now entry a PR adds is well formed -- which NOW Sync Gate, which only checks that one was added, does not read
- zero entries is a FAILURE here, not a pass: a check that reports success over an empty set is the shape being replaced

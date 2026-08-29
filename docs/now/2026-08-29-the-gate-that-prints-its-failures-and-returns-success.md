# NOW -- The gate that prints its failures and returns success (2026-08-29)

## The gate that prints its failures and returns success (Refs #2762)

- suite --ratchet --corpus-only is the command that gates master; it printed GATE FAILURES: 42 and exited 0 because only the expectations ledger decided the exit code
- proven by planting a conformance case that cannot pass: table 24/25, count 43, command still exit 0
- the count is now pinned in the ledger with the same three rules the discard volume uses: a rise fails, a fall fails until re-blessed, no pin fails
- same planted control after the change: exit 1, GATE FAILURES rose 42 -> 43 (+1)

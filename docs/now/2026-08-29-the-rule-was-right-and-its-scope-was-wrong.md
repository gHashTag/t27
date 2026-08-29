# NOW -- The rule was right and its scope was wrong (2026-08-29)

## The rule was right and its scope was wrong (Refs #2762)

- 41 of the catalog gate's 43 findings were the GoldenFloat rule applied to bnf and tnf, whose own standard= field says they are sized for range and counted in trits
- scope is now a declared rule=phi-ratio; tnf8 satisfies the rule and is deliberately unmarked, because a coincidence at one width is not a design
- GATE FAILURES 42 -> 2, and the ratchet from this morning demanded the re-bless on its first real improvement
- three self-inflicted detours: rebase --theirs is inverted, a line-hash baseline re-opens by design, and gh pr checks showed 2 of 40 runs

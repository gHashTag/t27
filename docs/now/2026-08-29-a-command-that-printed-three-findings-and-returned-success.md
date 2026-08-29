# NOW -- A command that printed three findings and returned success (2026-08-29)

## A command that printed three findings and returned success (Refs #2762)

- t27c catalog-gate ended in Ok(()) unconditionally: FINDINGS 3 on screen, exit 0 to the caller
- now exits 1 with the same count the suite uses, and the allowlist moved out of suite.rs so the two cannot drift into disagreeing about which findings are debt
- its --help said 83 records; the live number is 109

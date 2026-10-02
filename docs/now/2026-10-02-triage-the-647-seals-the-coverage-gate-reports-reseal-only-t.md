# NOW -- Triage the 647 seals the coverage gate reports; reseal only the stale-only class (2026-10-02)

## Triage the 647 seals the coverage gate reports; reseal only the stale-only class (Refs #5453)

- 647 = 18 stale-only (a), 599 gen-drift (b, #5576), 30 real spec defects (c, #5577); evidence in docs/SEAL_TRIAGE_2026-10-02.md
- Resealed 2 (arty_a7_integration twins, tests PASS 6, gen hashes unchanged); 16 more (a) seals held because test-report is BLOCKED
- Surprise: the parser drops for-all quantifiers, so AST-equal is not meaning-equal; test-report temp dir collides on shared spec stems

# NOW -- Define contributor key contracts and record their gates (2026-10-01)

## Define contributor key contracts and record their gates (Closes #5472)

- Add render and Queen contracts for trusted identity, bounded credentials, stable ownership and probe limits; preserve existing work-based XP without wallet credits.
- Execute 17 spec tests with Zig 0.16.0, catch all 19 negative controls, and kill all 8 render implementation mutants.
- Re-measure implementation status at ce4e0922: 1136 specs, 6680 functions and 187 functions without bodies; preserve the historical measurements.
- Reproduce Corpus Ratchet and Duplicate Body Ratchet failures on current master d4c4f2f4: the same 3 assertionless-test files and 11 duplicate groups remain outside this change.

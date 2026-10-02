# NOW -- Define contributor key contracts and record their gates (2026-10-01)

## Define contributor key contracts and record their gates (Closes #5472)

- Add render and Queen contracts for trusted identity, bounded credentials, stable ownership and probe limits; preserve existing work-based XP without wallet credits.
- Execute 17 spec tests with Zig 0.16.0, catch all 19 negative controls, and kill all 8 render implementation mutants.
- Re-measure implementation status at ce4e0922: 1136 specs, 6680 functions and 187 functions without bodies; preserve the historical measurements.
- Reproduce Corpus Ratchet and Duplicate Body Ratchet failures on current master d4c4f2f4: the same 3 assertionless-test files and 11 duplicate groups remain outside this change.

- Follow up on the owner's request to repair the existing CI failures: exercise the documented-command scanner itself, exclude words/repository paths from sibling invocations, and read its documented tracked/live population. The full command gate and its controls now pass.
- Make duplicate-body detection respect declarations, comments and string literals; four new controls fail before the repair and all nine controls pass after it. Nine genuine unledgered groups remain; no duplicate ceiling was raised.
- Remove 149 test declarations containing only comments or literal `assert true` from three specs. Non-test source remains identical; the counter spec's five executable C tests pass. Lower the assertionless ledger from 4049 to 3761, including the already-resolved cordic row, and regenerate only these three specs' seals.
- Re-run contributor contracts: all 17 Zig tests and both seals pass. Later previously hidden checks still report 15 unjudged type conflicts, numeric ring-096 signature drift and nine outdated corpus measurements. Exhaustive arithmetic and duplicated-function differential checks pass. Full canonical CI is not green; these results do not certify the incomplete historical ports as executable implementations.

- At the owner's request, limit the final repair to sharing the empty-result construction in `specs/hslm/forward_pass.t27`. All three wrappers preserve their existing placeholder behavior; this does not implement inference. Duplicate groups fall from nine to eight with all nine scanner controls passing and no ledger increase. An isolated generated-C harness executes 14 calls before and after and rejects a non-null-loss mutant; it supplies the existing backend's missing `null` spelling and does not certify the full module. Regenerate and verify only the two seal records naming this spec. Leave #5473 open and hand off the remaining CI debt.

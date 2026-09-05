# NOW -- The widened Admitted contract, tested at last (2026-09-05)

## The widened Admitted contract, tested at last (Refs #3270)

- master gained its own repair of the fixture while my PR sat open, so half of it was redundant. Closed #3277 and rebuilt the remainder minimally on master's shape: arm() takes an overridable named list, and three assertions test the contract #3238 widened the gate TO.
- An Admitted in a THIRD file is caught, the failure names it, and an empty _CoqProject is could-not-run rather than clean. Control: restoring the two-operand gate fails the third-file assertion. Checks 10 to 13.
- Renamed the test and the workflow step, which both still said 'both files' while the gate reads nine. No dangling reference remains.

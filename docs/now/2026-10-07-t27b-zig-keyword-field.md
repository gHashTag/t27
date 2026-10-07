# NOW -- t27b: a struct field named for a Zig keyword (2026-10-07)

## The keyword-field refusal was stale (Closes #7247)

- New conformance spec `specs/tri/t27b/conformance/zig_keyword_field.t27`. It gives 4 pass with 0 vacuous under `t27c test-report`, and the same 4 under t27b, with 14 runtime asserts.
- t27b refused a struct literal or a field access whose field is named for a Zig keyword (`.align = 4`, `r.error`), because t27c used to print the name bare. Since #6451 the reference writes `@"align"` in the declaration, the literal, the access and the assignment target, so the refusal only blocked files the reference passes. Both refusals (`ExprStructLit(zig keyword field)` and `ExprFieldAccess(zig keyword field)`) are gone; the other shapes in `zig_syntax_defects` stay.
- 8 mutants each fail the same tests in t27b and in the reference. Of 11 probes, none disagree: 7 pass in both, 1 fails in both, and 1 has one passing and one failing test in both. The other 2 (a keyword field inside a nested literal, and a struct of four keyword fields) are blocked by the reference and do not parse in t27b.
- Code is in `cli/t27b/src/lower.rs`. Tests are in `cli/t27b/tests/source.rs`. Both files are listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- Ledger `docs/reports/t27b_expectations.json`: `specs/fpga/linker.t27` (18 pass, 39 asserts) and the new spec move to pass. `specs/compiler/zig_field_syntax.t27` now stops at `type str?`.
Refs #6063. Closes #7247.

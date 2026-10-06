# NOW -- t27b: module-level constants that hold an optional (2026-10-06)

## `const X: ?T = ...` and optional struct fields in constants (Closes #7116)

- New conformance spec `specs/tri/t27b/conformance/const_optional.t27`. It gives 4 pass with 0 vacuous under `t27c test-report`, and the same 4 under t27b, with 22 runtime asserts.
- A module-level `?T` constant now goes to rodata. The payload comes first and the has-value flag sits after it, the same layout `opt_temp` uses. `null` leaves the slot zero. A value is folded as the inner type and then marked present. A constant that names another `?T` constant of the same type copies its bytes.
- The same path fills optional fields inside struct and array constants, including fields with a `= null` default.
- An optional that holds a `str` is still refused as `ConstDecl(?T)`. The reference accepts it, so this stays a known gap and does not count as a mismatch.
- 8 mutants each fail the same tests in t27b and in the reference. Of 9 negative controls, 4 are blocked by the reference and refused by t27b, 4 pass in both, and 1 fails in both.
- Code is in `cli/t27b/src/lower.rs`. Tests are in `cli/t27b/tests/source.rs`. Both files are listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- Ledger `docs/reports/t27b_expectations.json`: `specs/port/bootstrap/src/parse_conform.t27` (9 pass, 38 asserts) and the new spec move to pass. The not-pass count goes from 50 to 49.

# NOW -- t27b: `const Name = T;` is a type alias (2026-10-07)

## A module const that names a type (Closes #7241)

- New conformance spec `specs/tri/t27b/conformance/type_alias.t27`. It gives 4 pass with 0 vacuous under `t27c test-report`, and the same 4 under t27b, with 14 runtime asserts.
- `const Name = T;` with no annotation, where `T` is a bare name that spells a Zig type (a primitive, a struct, an enum, `[N]T` of one, or another such alias), is printed unchanged by the reference, and Zig reads it as a type alias. t27b now resolves `Name` to `T` wherever a type is read: parameters, results, fields, locals, arrays and struct literals.
- Still refused: an alias of `str` or `[N]str` (Zig does not know `str`), an alias of a type t27b cannot name (`std.mem.Allocator`), an alias cycle, a type alias used as a value, and a struct literal of a scalar alias (`const Duo = u8; Duo{ .lo = 3 }`). Zig refuses the last one too. Before this fix, t27b recursed on it until the stack overflowed; mutant m8 found that.
- 9 mutants: 5 fail the same tests in t27b and in the reference, and 3 are blocked by the reference and refused by t27b. One (`const Signed = u8;`) runs under the reference, where `negate(5) == -5` is simply false. t27b refuses it as `literal out of range`.
- 12 probes, none disagree. 6 pass in both, 1 fails in both, and 4 are blocked by the reference and refused by t27b. 1 (`p13`, a test-local `var x: Code` that is then assigned) passes the reference and is refused by t27b as `StmtAssign(reference redeclares)`. That rule models an older reference defect, and this probe suggests it is now too broad. Refusals are known gaps, not mismatches.
- Code is in `cli/t27b/src/lower.rs`. Tests are in `cli/t27b/tests/source.rs`. Both files are listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- Ledger `docs/reports/t27b_expectations.json`: `specs/compiler/zig_primitive_bindings.t27` (2 pass, 9 asserts) and the new spec move to pass. `specs/numeric/gfternary.t27` now stops at `ExprCall(@setEvalBranchQuota)`.
Refs #6063. Closes #7241.

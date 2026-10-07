# NOW -- t27b: a postfix optional type `T?` (2026-10-07)

## `type str?` (Closes #7415)

- t27 also spells an optional after the type, as in `note: str?`. t27c's type mapper writes `T?` as Zig's `?T` (#6451), and t27b now reads it the same way. The parser keeps the suffix only on a struct field's type. `-> T?`, `const X: T? = ..` and a local's `T?` do not parse, and a parameter's `T?` loses its `?` in the parser, so both readers see a plain `T` there.
- New conformance spec `specs/tri/t27b/conformance/postfix_optional.t27`. It gives 4 pass with 0 vacuous in the reference, and the same 4 in t27b. It covers `str?`, `u16?` and an optional struct field: read, compared with null, unwrapped, passed to a `?T` parameter, and assigned in a loop.
- 7 mutants each fail the same tests in both. Of 22 probes:
  - 4 pass in both;
  - 1 fails in both;
  - 15 are blocked by the reference and refused by t27b;
  - 2 are honest t27b refusals where the reference passes: a `u32??` field (an optional of an optional) and a `void?` field.
- `specs/compiler/zig_field_syntax.t27` gets past `type str?` and stops at its next construct, the Rust-suffixed literal `0i32`.
- Code is in `cli/t27b/src/lower.rs`, listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.

Refs #6063. Closes #7415.

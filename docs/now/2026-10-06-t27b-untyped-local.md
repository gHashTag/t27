# NOW -- t27b: an untyped local, as the reference prints it (2026-10-06)

## untyped local (Closes #6967)

- New conformance spec `specs/tri/t27b/conformance/untyped_local.t27`. Under `t27c test-report` it gives 6 pass, 13 runtime asserts and 0 vacuous; under t27b it gives the same 6 pass with 13 runtime asserts. Of four mutants, three fail at runtime in t27b. The fourth, a u64-sized expectation, is refused because its literal does not fit u32, as Zig refuses it.
- `var i = 0;` takes the width the reference pins on an untyped `var` set to a bare integer literal (`zig_int_literal_default_type`): the literal's suffix, else u32, or u64 past u32::MAX. Wrapping arithmetic in the spec (`0 -% 1`) shows the width.
- `const _config = undefined;` with no type binds nothing, as the reference prints it and Zig accepts it. A read of it stays refused.
- Still refused, and refused by the reference's Zig as well: `var x = -1`, `var x = 0.5`, `var x = undefined`, and an octal literal. The lab checked each one against `t27c test-report`.
- Lab corpus run on the merged head 1e6e128ba compared with master 6540a678f: t27b passes 526 of the 832 specs the reference passes, up from 523 of 831. The specs that move are `submit.t27`, `orbitofrontal_value.t27` and the new spec. `tnf17_jtag.t27` goes from blocked to fail on the same two tests the reference fails. Mismatch 0, reference_disagree 0. In the ledger, `submit.t27` moves to pass and two entries are added, so the cap goes from 55 to 54.
- The edit is in `cli/t27b/src/lower.rs` (`int_lit_width`, two arms in `local_with`), plus one test in `tests/source.rs`.

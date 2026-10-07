# NOW -- t27b: integer types of odd width 1..31 (2026-10-07)

## `type u1`, i2, u4, u21 (Closes #7368)

- New conformance spec `specs/tri/t27b/conformance/odd_width_int.t27`: 9 tests, 9 pass with 0 vacuous under `t27c test-report`, and the same 9 under t27b with 57 runtime asserts.
- t27b now has an integer type of every width 1..31 other than 8 and 16. A value is held like a u8, in a W register, zero- or sign-extended, and checked against its own range after `+ - *`. A product of more than 16 bits is formed with umull/smull, so one past 2^32 still traps. `~` flips only the type's own bits, and `<<` drops the bits that leave the type, as Zig does.
- A wrap-mode shift by a runtime amount on an odd width is refused by name: `bits - 1` is not a mask there. Widths 33..63 (`type u48`) and 128 and above stay refused; they need register pairs or a multi-limb lowering.
- 14 mutants of the spec each fail the same single test in t27b and in the reference. `cargo test -p t27b` on the Railway lab: 125 passed, 0 failed. The differential test now covers twelve odd widths for every operator, unary op, widening and cast at the edge values.
- Code is in `cli/t27b/src/{ir,codegen,eval,lower}.rs` and `cli/t27b/tests/differential.rs`, listed in `tools/policy/foreign-exceptions.txt` under the owner's standing rule; the spec that replaces it is the t27b port (#6198).
- Ledger `docs/reports/t27b_expectations.json`: `specs/port/trinity/fpga/openxc7-synth/led_off_test.t27` (1 test, 1 assert) and the new spec move to pass.
Refs #6063. Closes #7368.

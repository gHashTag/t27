# NOW -- t27b: an array literal of strings for a slice field or a returned slice (2026-10-09)

## a static written at every entry, from slice_lit_plan.t27 (Closes #7833, #8091)

- t27c prints `files: ["specs/automation/wrapup-auto.t27", "AGENTS.md"]` for a slice field, and
  `return ["W_mass", "Z_mass", "H"];` for a returned slice, as `@constCast(&[_][]const u8{ ... })`: a
  comptime array, alive for the whole process and defined to read. t27b refused it, because a
  static is bytes written before the program runs and a string's address is not known then.
- `slice_lit_plan.t27` replaces REFUSE_STRINGS with STATIC_STRINGS and adds `at_entry`: the array is
  a module global written at the start of every test, invariant and bench, the way a module var
  holding a string already is. Equal literals at two sites share one global. Run-time elements stay
  refused, and #7765's write check covers these statics as it covers any static.
- Builds on the earlier lane branch `t27b-string-slice` (f7abed354, never opened as a PR), merged in
  and brought up to date with #7765's plan.
- Conformance: `specs/tri/t27b/conformance/string_slice_literal.t27`, 6 tests. The reference passes
  6 of 6 with 0 vacuous. t27b passes 6 of 6 with 30 runtime asserts. `specs/automation/wrapup-auto.t27`
  now passes in t27b with 6 runtime asserts. Mutants that flip one assert: 30 of 30 killed in the
  conformance spec and 30 of 30 in the plan, under both the reference and t27b.
- Glue: `lower/arraylit.rs` +27 -3, `tests/arraylit.rs` +13. Ledger: wrapup-auto.t27 moves from
  blocked to pass, the conformance spec gets a new row, and max_not_pass goes from 27 to 26.

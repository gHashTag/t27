# NOW -- var tuple destructure silences never-mutated Zig locals (2026-10-04)

## `var (s, d) = f();` no longer BLOCKS `t27c test-report` (Closes #5886, Refs #5682)

### What changed

- `bootstrap/src/compiler.rs`, Zig `Codegen::gen_stmt`, `StmtLocal` tuple-destructure branch: when the destructure is `var`, each named element gets the `_ = &name;` silencer the single-name `var` path already emits, and the name is recorded in `discarded_by_ref` so the W730 rule drops a later `_ = name;` (Zig's "pointless discard"). A bare `_` element gets no silencer; `let (s, d)` is unchanged.
- New unit test `test_var_destructure_never_reassigned_zig`: both elements named, one element `_`, a bench body with the spec's own `_ = s;` (W730), and `let` untouched.
- `bootstrap/stage0/FROZEN_HASH` resealed to the new `compiler.rs` digest.

### What was verified

- Negative control: the test with the fix removed fails by assertion (`_ = &s;` silencer missing); with the fix it passes. Whole `t27c` bin unit suite: 1759 passed, 0 failed, 2 ignored.
- `d_slow_blink.t27` (PR #5798 head, `var (new_state, led) = ...`): branch-point binary BLOCKED "local variable is never mutated" at spec.zig:88; fixed binary 6/6 pass, 4 invariants; `zig test` on the generated file passes all 6.
- Zig output from `gen` differs between branch point and fix in 2 of 1174 specs (`specs/port/tools/trinity_c_abi.t27`, `verify_trainer_c.t27`), only by added `_ = &x;` lines; both are BLOCKED before and after by the same earlier, unrelated parse error. `gen-c` is byte-identical for all 1174 specs. `t27c suite --repo-root .` text and JSON are identical before and after (662 total failures, 507 distinct failing specs, both runs).

### Not verified

- The W596 named-tuple destructure path still always emits `const`, so a `var` named-tuple destructure is not covered by this change.
- `discarded_by_ref` is module-wide and never cleared per function (pre-existing); this change adds names to it. No corpus spec shows a dropped `_ = name;` from it.

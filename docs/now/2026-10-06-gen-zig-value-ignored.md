# NOW -- gen-zig value-ignored, landed as an owner exception (2026-10-06)

## bootstrap/src/compiler.rs gen-zig fix (Closes #6315, debt #5980)

- Zig rejected generated code with "value of type 'bool' ignored" and the like: a brace invariant's predicate was emitted as a bare expression, a function's tail expression was dropped instead of returned, and a value-returning call used as a statement was not discarded. t27c gen-zig now asserts the predicate, returns the tail value, and writes `_ = call(...)`.
- `specs/compiler/zig_value_ignored.t27` is the regression spec; it replaces the hand-written Rust test `bootstrap/tests/value_ignored_zig.rs`, which this PR deletes.
- `bootstrap/src/compiler.rs` is hand-written Rust. The owner approved it as an exception (label `owner-approved-foreign`, entry in `tools/policy/foreign-exceptions.txt`); the debt is #5980, where the fix moves to t27core.
- Reference pass count on the t27c lab (gen-zig + zig test over specs/): 723 -> 738, 15 blocked->pass, 0 pass->non-pass. Two specs go from blocked to fail; that is their own test failing, filed as #6560.
- Seals were regenerated on the lab. Three specs now compile but their own tests fail or hang (tri27_machine, clock_domain_tb, gf16_accel_tb); they were sealed with `--force`, which records the failures in the seal, and are ledgered as `tests-fail` in `tools/seal_baseline.txt` (#6560).

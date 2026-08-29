# NOW -- A truncated test reported OK (2026-08-30)

## The mark was computed, and one of the two emitters read it (Refs #2843)

- the parser marks a block it could only partly lower: `extra_field = "partial"`, written at two sites
- `gen_invariant_block` honours it and prints `// invariant: X NOT CHECKED -- body was not lowered`
- `gen_test_block` reads `extra_field` ZERO times, so an amputated test was emitted as an ordinary one -- valid Zig that `zig test` compiles and reports OK
- 17 TestBlocks across 11 specs carry the mark; Zig is the ONE backend that actually runs the spec's tests
- a dropped assertion was therefore indistinguishable from a passing one, which is the most expensive shape a defect can take here
- fixed with `return error.SkipZigTest` -- Zig's own signal -- so `zig test` prints SKIP and the run reports `1 passed; 1 skipped; 0 failed`
- emitted LAST, not first: an early return makes the surviving prefix `unreachable code` and Zig rejects the file. Placing it first cost two ast-check passes before the measurement showed it
- ast-check 224/559 before and after; 17 tests now visibly skipped

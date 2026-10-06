# NOW -- t27b: `-> undefined` on a fn the reference never analyzes (2026-10-05)

## type undefined (Closes #6435)

- A port's stub `fn main() -> undefined { undefined; }` on a fn no test, invariant or constant reaches now lowers as a fn with no result; its `undefined;` body is the stub trap no test reaches. Zig never analyzes such a fn, so the reference compiles the file and runs every test.
- On a reached fn the type stays refused (`type undefined`): `t27c test-report` is BLOCKED there.
- The code is a new submodule, `cli/t27b/src/lower/unanalyzed.rs`, with one hook in `signature`; tests in the new `cli/t27b/tests/tail.rs` (helpers in `tests/common/mod.rs`).

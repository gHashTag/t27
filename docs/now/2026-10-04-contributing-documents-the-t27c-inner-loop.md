# NOW -- CONTRIBUTING documents the t27c inner loop (2026-10-04)

## CONTRIBUTING documents the t27c inner loop (Closes #5934)

- `CONTRIBUTING.md` gains a "Build speed" section. A one-line edit rebuilds in 2.4-3.3 s with `cargo check`, 3.2-4.1 s as a debug build and 3.2-5.0 s as an incremental release build, against 29.8-33.9 s for the plain release build CI uses (idle M1 Pro, edit to `suite.rs`).
- It says not to benchmark or ship an incremental release build, and that `TRI_T27C=target/debug/t27c` is needed because `scripts/tri` prefers a release binary.
- It covers the `compiler.rs` seal step. The command `FROZEN.md` and `build.rs` print fails after the edit (#5928), so the section points to the working routes instead.
- The "Specs and tests" line now points to that section, and `CLAUDE.md` section 2 gains a one-line pointer. No build configuration changes.

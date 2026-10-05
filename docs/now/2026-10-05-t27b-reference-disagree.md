# NOW -- t27b compares with the reference test by test: reference_disagree (2026-10-05)

## reference_disagree apart from jit/interp mismatch (Closes #6441)

- `mismatch` was t27b's JIT against t27b's own interpreter, both running lower.rs's IR; it is now also printed as `jit_interp_mismatch`. A file both t27b and the reference failed used to count as agreement whatever tests failed.
- `t27b corpus --reference` reads `t27c test-report --verbose` per test, caches the verdicts (`tests=` field; old rows re-run), and counts `reference_disagree` / `reference_disagree_tests` / `reference_compared`. Either count non-zero exits 4.
- `specs/tri/t27b/steward.t27` adds `is_alarm_tests` and `lanes_stop` (120 tests); `gen/c/tri/t27b/steward.c` is the t27c lab's gen-c (gen-check SAME 904af7fc1f22). `tri t27b doctor` raises LAB-REF-DISAGREE; `tri t27b diff SPEC` prints both verdict lists.
- First measurement on master (t27b lab, 2026-10-05 15:47Z): reference_disagree 0 over 668 files and 2466 tests; the 7 fail/fail files fail the same test on both sides.
- Owner exception 2026-10-05: hand-written Rust/Python glue, to be replaced by t27-generated code under #6198.

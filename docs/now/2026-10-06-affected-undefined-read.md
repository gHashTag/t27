# NOW -- ci/affected.t27 stops reading set bytes it never wrote (2026-10-06)

## specs/ci/affected.t27 graph loop stops after an overflow (Closes #6663)

- `closure()` kept walking the graph after `set_add()` overflowed (`sn = scap + 1`), so `in_set()` read `set[12..21]`, bytes the test had left `undefined`. The JIT and zig read garbage and still answered RUN_ALL; t27b's interpreter faulted, which was the lab's one JIT/interpreter mismatch (run 099ac2224).
- Fix: the graph loop is `while (s < c and sn <= scap)`. Answers are unchanged.
- t27b lab, master binary: `t27b test --check` 13 passed, 0 mismatches, 58 runtime asserts; reference `t27c test-report` 13 of 13. `gen/c/ci/affected.c` is `t27c gen-c` output (one line moved; master's spec regenerates byte-for-byte to the committed file).

# NOW — a test pinned a width the emitter deliberately widened

Last updated: 2026-08-22

## Pin __mul_noop at 64 bits and anchor the helper's endfunction (Closes #2389)

- Branch: `fix/2389-mul-noop-width`
- Issue: #2389 · part of #2386

### Что легло

`bootstrap/tests/verilog_r_si_1.rs`, plus one line pruned from
`scripts/ci/test-baseline.txt` (378 → 377).

Two defects in one test. It asserted `function [31:0] __mul_noop;` while the emitter
writes `[63:0]` — widened on purpose by t27#1886 so u64 products stop truncating. And its
`endfunction` assertion matched **any** function in the module, of which this spec emits
three, so it certified a neighbour.

The width is **pinned at 64, not relaxed to any width**: a silent narrowing back to 32
restores the truncation bug, so the number is load-bearing and the message now says so.

### Границы честности (BINDING)

- **No emitter change.** The emitter was right; the test was stale.
- **This is 1 of the 7 in #2386. Six remain**, and whether they ever passed is still
  unestablished for all of them — the commit that added each claims a working
  implementation, but I did not check those commits out and run them.
- The test's `compile_spec() else { eprintln!(...); return; }` is a **self-skip counted as
  a pass** — the same class as #2370. Left alone here to keep the diff to what was proved,
  and worth its own pass.

### Evidence

Two mutants in `bootstrap/src/compiler.rs`, each verified planted before the run. Both
required a local `bootstrap/stage0/FROZEN_HASH` reseal, since editing `compiler.rs` trips
the freeze gate in `build.rs:235` — the first attempt produced no test output at all
because the build failed, and would have read as "the guard does not bite" had the plant
not been checked.

- `:11842` `[63:0]` → `[31:0]` → `R-SI-1 helper missing or narrowed: expected
  `function [63:0] __mul_noop;`. A 32-bit helper truncates u64 products…`
- the `__mul_noop = acc[63:0];` assignment removed → `R-SI-1 helper does not assign its
  64-bit result.`

`1 passed; 1 failed` each time, each caught by its own assertion. Restored, `2 passed`.
Both `compiler.rs` and `FROZEN_HASH` restored; neither is in this diff.

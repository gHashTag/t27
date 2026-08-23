# NOW — the braceless test loses its body (2026-08-23)

The corpus ratchet named two specs failing `parse-no-discard`, and there was no command that could ask a single file where it stops consuming. There is one now, and it found the construct in two runs.

- **`t27c parse-accounted <file> --bisect`.** The suite computes this count for every spec and reports it only as a pass or a fail. Nineteen specs sit in the ledger for it and nobody could interrogate one without running the whole corpus. `--bisect` re-parses with one top-level item removed at a time and names the ones whose removal changes the count.

- **It answered immediately.** `specs/fpga/power_analysis.t27`: 3 tokens, all of them `invariant total` at lines 372-373. `specs/fpga/vcd_conformance_compare.t27`: 120 tokens across twelve items, alternating `test` and `invariant`.

- **The generalisation I reached for was wrong, and the tool refuted it.** "`invariant` inside a `test` is discarded" — `fifo_tb.t27` has fifteen of them at the same indentation and discards **zero**. Then "it is the expression form" — all seven forms I probed (`==`, `>`, `<`, `>=`, `!=`, a bool literal, a field access) discard identically. Two plausible causes, both dead on measurement.

- **It is the block syntax.** There are two `test` forms in this corpus, and one of them loses its body:

  ```
  test t { invariant f(1) == 1; }                     0 discarded
  test t / given x = f(1) / invariant x == 1;         3 discarded
  test t / given x = f(1) / expect x == 1             9 discarded
  ```

  The braced form is consumed. The braceless `given`-style TDD form is parsed as far as its header and the rest is thrown away.

- **The second gate agrees.** `no-vacuous-invariant` reports `power_analysis.t27`: *1 invariant declared but not lowered — the clause body was discarded and nothing is checked*. Two phases, one cause, and neither pointed at it because neither could name a construct.

- **Not fixed here.** The repair is in the parser, and `bootstrap/src/compiler.rs` is under the stage0 freeze — `FROZEN_HASH` must be updated in the same commit, which makes it a deliberate change rather than something to attempt while chasing something else. Filed with the minimal reproductions.

- **What the corpus ratchet was actually saying.** Three specs are *fixed* and still in the ledger (`power_analysis`, `vcd_conformance_compare`, `array` — all under `parse`), and two are unexpected failures under `parse-no-discard` — the same two specs, which now parse but do not parse completely. The ledger is at 221 of a 221 cap, so an improvement it does not record is headroom nobody has.

Refs #2474

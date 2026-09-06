# NOW -- tri one-away, the specs a single repair can move (2026-09-06)

## tri one-away (Refs #3359)

- The measurement that explained three +0 iterations lived in a shell script in `/tmp`.
  A command makes it survive the session; a script does not.
- `tri one-away` reads every spec, generates Rust, runs rustc, and buckets the failures
  by how many REAL errors they carry: **56 of 243 carry exactly one**, and only those can
  be moved by a single repair.
- Their sole class, ranked: `cannot find type` 8, `expected one of ...` 7,
  `expected type, found keyword` 5, `mismatched types` 5, `use of moved value` 4.
- The summary line is excluded and that is the whole point. `rustc` ends with
  `error: aborting due to N previous errors`, which matches `^error` like any
  diagnostic; counting it turns 56 into 0.
- A spec that does not GENERATE is reported separately. It is not a spec with zero errors.
- Six controls, one per claim -- including one asserting that counting the summary WOULD
  give the opposite answer, so the mistake cannot come back silently. Five unit tests over
  the two pure functions.
- Its 56 disagrees with the 55 measured by hand, and the disagreement is resolved rather
  than averaged: `vsa/jones_polynomial.t27`, the single spec the `vec!` repair moved
  from two errors to one. The hand count ran against the binary from before that merge.
- Claimed with `tri loop claim corpus-one-away` before the work started, which is the
  first time this pass has done that in the right order.

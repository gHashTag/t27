# NOW -- The fetches census moved and nobody said so (2026-09-05)

## The fetches census moved and nobody said so (Refs #3284)

- `cli-tri` / `build` has been red on master since about 10:46Z, on the step named
  "No census moved without saying so".
- The boundary is 0d7b13a4e PASS to d7a71461e FAIL. d7a71461e added
  `cli/tri/src/red.rs`, 240 lines, with two new `per_page` call sites and no re-blessing.
- The first commit accused was the wrong one and was mine. It was named by reading the
  most recent failing run and calling it the first; the bisect over the four commits in
  between named the real one.
- The gate is `tri census pin --gate`. Run as `tri census pin` it prints
  "3 census(es) pinned" and exits 0, which reads like a pass and is a different question.
- The ledger names the shape, not just the total: fetch sites 26 to 28, "asks whether the
  page filled" 9 to 10, "prints what it got" 2 to 3.
- So of the two new sites one checks the fill and one does not. `red.rs:141 fn failing_steps`
  reads `runs/{id}/jobs?per_page=100` and prints what it got; a run with more than 100
  jobs truncates silently there.
- Blessed rather than repaired, because `red.rs` documents its bounds at length and
  offers `--deep` for the full read. The unchecked site is recorded here so it is not
  invisible.

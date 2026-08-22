# NOW — a green cli-tri run said nothing about three tests that never ran

Last updated: 2026-08-22

## Runtime self-skips are now counted, named, and ratcheted (Closes #2377)

- Branch: `skipvis/lean-smoke-gate`
- Issue: #2377
- **#2370 stays OPEN.** This does not fix it and does not pretend to.

### What landed

`cli/tri/skipwatch.py` — parses `cargo test` output for tests that skip *themselves* at
runtime, writes the count and reasons to `$GITHUB_STEP_SUMMARY` and a `::notice`, and
fails when the set **grows**.

`cli/tri/skip-baseline.txt` — the three of them, emitted by `skipwatch.py
--emit-baseline` from a real test log. Not hand-written. A typed `3` would restate what
#2370 already established, would not survive a rename, and would detect nothing.

`.github/workflows/cli-tri.yml` — the test step gains `--nocapture --test-threads=1` and
tees its log; a new `runtime skip ratchet` step reads it.

### Why both flags are load-bearing

The three Lean smoke-gate tests do not use `#[ignore]`. They `println!` a reason and
`return`. So libtest counts them as **passed** — the run reports `173 passed; 0 failed;
0 ignored` — and `cargo test` *captures* the stdout of a passing test, so the reason
never reaches the log. `grep -icE 'skip'` over the full default log returns 2, and both
are test *names*, not reasons.

`--nocapture` is what makes the reason exist in the log. `--test-threads=1` is what makes
it attributable: libtest lands the reason on the `test NAME ... ` line it has already
opened, and parallel workers interleave those.

### Biting

A fourth deliberately-skipping test was planted in `fpga::tests` and the suite re-run.
`cargo test` reported `172 passed; 2 failed; 0 ignored` — the mutant counted as passed,
`ignored` still zero, which is precisely the defect. The ratchet reported `4 test(s)
skipped at runtime ... (baseline 3)`, named the new one, emitted `::error`, and exited 1.
The mutant was then removed; `cli/tri/src/fpga.rs` is byte-identical to master.

### Honesty boundaries (BINDING)

- **No Lean toolchain was installed.** `elan` + a Lean toolchain is multi-gigabyte and
  this machine has been at ~1.1 GB free all session, hitting literal zero eleven times
  today. The real fix for #2370 is still the install, and it is still not done.
- **This is a ratchet, not coverage.** It cannot make a skipped test run. It only makes
  the skipping legible and stops the set from growing unnoticed.
- **It only recognises the in-repo skip convention** — a leading `SKIP:` / `skip:` on a
  line of test stdout. A test that bails out early while printing something else, or
  while printing nothing at all, is invisible to it exactly as it is invisible today.
  That is a real hole, not a detail.
- **The baseline was generated on macOS**, with the demo bitstream present and yosys 0.63.
  If the ubuntu runner's skip set differs, the ratchet reddens on its first run rather
  than passing quietly — that is the intended direction of the failure.
- Two tests (`test_smoke_gate_json_synthetic_verify_lean`,
  `test_smoke_gate_json_theorem_matrix_is_computed`) fail on this laptop against local
  yosys 0.63. They are green on the runner and were not touched.
- The verifying job is `cli-tri`'s `build`, which is **not** a required check, so
  auto-merge will not wait for it. It is watched on master after the merge.

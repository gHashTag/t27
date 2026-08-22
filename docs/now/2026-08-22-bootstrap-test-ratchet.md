# NOW — cargo test ran 1 of 73 targets

Last updated: 2026-08-22

## Ratchet the bootstrap suite over failing test names (Closes #2382)

- Branch: `fix/2382-bootstrap-test-ratchet`
- Issue: #2382

### Что легло

`scripts/ci/test_ratchet.py`, `scripts/ci/test-baseline.txt` (383 entries), and
`.github/workflows/bootstrap-tests.yml` (check name `test-ratchet`).

The measurement that motivated it, on `e53b9d048`:

| command | targets | passed | failed |
|---|---|---|---|
| `cargo test -p t27c --tests` | **1** | 1621 | 13 |
| same, `--no-fail-fast` | **73** | 2031 | **383** |

`cargo test` stops after the first failing target. The unit target fails — those are the
13 named in #2292 — so the other 72 compile and never run. Every regression guard added
this week (#2363, #2003, #2006, #1977, #1985) lives in one of them. They were not broken
and not missing; they were unreachable. The guard for #2363 **passes** when actually run.

Of the 383: **61 of 73 targets are completely clean**; 358 failures are one target
(`tests/icarus_lowerable.rs`, 0 passed of 358) attributable by shape to the emitter
defect #2325; 13 are the known unit failures; **12 across 10 targets have never been
examined**.

Baseline is a set of `target<TAB>test` keys, not a count — 383 still passes when one
failure is fixed and another appears. A baselined test that starts passing is reported,
not failed on, so the set cannot rot unnoticed.

### Границы честности (BINDING)

- **The 12 unexamined failures were not diagnosed and their age was not established.**
  Born-failing, long-failing and just-regressed need different responses. One of them,
  `dma_local_addr_autoincrement_both_paths`, sits in the file #2345 changed — **that is
  a lead, not a regression claim**.
- The 358 are attributed to #2325 by shape (whole-target, 0 passed, `iverilog` present),
  not by reading each failure.
- Baseline measured on **macOS/arm64** at `e53b9d048` with a warm target dir; a Linux
  runner may differ, and the first CI run will show whether it does. If it does, the
  baseline must be regenerated from a runner log, not patched by hand.
- This job is **not a required context**, so it blocks no merge (#2376). Making it one is
  the repository owner's decision.
- It does not fix anything. It stops the next regression from being invisible.

### Evidence

Three bars, measured. TRUE: unmodified log against its own baseline exits 0. BITING: with
`bootstrap/tests/on_clock_plain_assign.rs:85` altered to assert a string the emitter never
produces, a real rebuild + rerun gave 73 targets, 384 failures, and the ratchet exited 1
naming `on_clock_emits_assignment_whose_rhs_does_not_reference_the_target`. Degenerate
inputs — a one-target (fail-fast) log, an empty log, a missing log — all exit 2 with
"NOT evaluated" rather than reading absence as a clean set. Exit codes were measured
without a pipeline, since `$?` after a pipe reports the last command's status.

Cost: **1 min 49 s** wall on a warm cache.

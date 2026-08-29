# NOW -- One command for the treadmill (2026-08-30)

## One command for the treadmill (Refs #2851, #2767)

- `tri seals drift` lists seals whose claims no longer match what the compiler
  produces; `--fix` re-seals them and syncs their twins. The five-step recipe
  measured yesterday becomes one command.
- **The value is not the thirty seconds, it is the two refusals a hand-rolled
  loop forgets.** A compiler older than its own source answers with the PREVIOUS
  output and every seal appears to hold -- so `drift` refuses outright, with the
  rebuild line. And `t27c seal` EXITS 0 while printing `gen_hash_zig=none` for a
  spec no backend accepts, so the refusal reads the claims, not the status:
  #2210 measured that a batch re-seal without it would have recorded 348
  reproducibility assertions for output that does not exist.
- **It does not decide whether the new output is wanted.** Without `--fix` it
  prints that re-sealing is a STATEMENT and names `t27c corpus` as the thing to
  read first. An emitter regression sealed is a regression written into the
  record as truth.
- Four controls: clean tree reports 0 and agrees with the seal gate; a planted
  drift is named; `--fix` repairs it and drift returns to 0; a `touch` on
  `bootstrap/src/compiler.rs` makes it exit 1 with the repair line. Plus the
  none-refusal, controlled on `specs/sandbox/session_timeout.t27`: re-sealed 0,
  REFUSED 1, planted hash untouched.
- **Audit with a clean result, recorded because it is one.** The ruleset requires
  exactly FOUR contexts -- `check-now-freshness`, `validate`, `check`,
  `check-linked-issue` -- and all four assert something today. The trusted-bot
  bypass in two of them is narrowed by login to `dependabot[bot]` and
  `github-actions[bot]`; on three merged PRs of mine all four reported `pass`.
  Everything else, 42 workflow files, is advisory.
- ci-gates 267-269. 311 tests pass.

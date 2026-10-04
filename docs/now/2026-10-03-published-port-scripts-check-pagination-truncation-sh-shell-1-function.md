# NOW -- Pagination port observes the actual eighth-probe boundary (2026-10-03)

Closes #5831. Supersedes the incompletely verified source in PR #5838.

## What was read

- `scripts/check-pagination-truncation.sh` at canonical `62c105ed21b03b366b6909b293a62ef6e0a715fb`: eight probes, doubled limits, strict count-below-limit success, query/usage/auth failures as exit 2. Original script SHA256: `aa0bc02d399bb421479d39ad554228072b1b68df6104d4ab5a42b6331d6fd630`.
- Queen source `5380649c96e90953595347528a5115aa380e70e7`, published head `e24a07da72541c6a562b757ee4507a0c19592333`. The published source matches the judged source; original 9 Zig tests pass, but changing MAX_PROBES from 8 to 9 still passes every one. The old ninth count truncates too, so its verdict cannot distinguish a stopped query loop.

## What changed

- The ninth reply in the cap test would now succeed, making it observable if consumed. Additional cases verify success at exactly the eighth reply, ignore a query error beyond the cap, preserve a custom starting limit, and reject an empty owner even with argc 2.
- All four original runtime function bodies and signatures remain unchanged. The branch preserves original bee/publisher history and carries one truthful coordination entry; no original script, compiler, generated file, seal or failure ledger changed.

## Verified

- Fresh t27c from master `6e3322918`: all 12 Zig tests pass, 4/4 functions covered, no discarded tokens, typecheck zero errors/warnings. Generated Rust library compiles; its backend does not lower the .t27 tests.
- Four changed source copies compile but fail the strengthened tests: nine probes (2 failures), seven probes (1), ignoring custom start (1), and ignoring empty owner (1). `test-report` exits zero even when FAIL appears; verdict text was inspected.
- Actual generated Rust decision code and the original shell agree on 30 scenarios under a fake `gh`: start limits 1/3/100, populations around first/doubled/eighth limits, ninth success/error ignored, first/second query failures, no authentication and missing owner. The original trace makes at most eight list queries. No live GitHub request or authentication change occurs in this parity harness.
- A caller passes nonnegative observed counts or NO_COUNT=-1, a positive starting limit, and at most the eight relevant replies. Network/process execution and informational API/output formatting remain caller plumbing as allowed by the port issue.

## Reproduce the stopping-boundary proof

- Run `t27c test-report specs/port/scripts/check-pagination-truncation.t27`; expect 12 pass, 0 FAIL, no BLOCKED.
- Copy only the source to a scratch path and change `const MAX_PROBES: usize = 8;` to 9. The same test report must fail `main_caps_probes_at_eight` and `main_ignores_query_failure_beyond_cap`.
- Change the cap to 7 instead: `main_accepts_population_on_eighth_probe` must fail. Change the initial `limit` to DEFAULT_START_LIMIT: `main_respects_custom_start_limit` must fail. These distinguish the original bound from either neighboring bound and from silently dropping the optional start argument.

## Not verified

- This is the bounded decision port, not a replacement GitHub client or a claim that private repository discovery/output formatting was exercised. Generated Rust was checked for compilation and decision parity separately from the executed Zig tests.
- No full-repository all-green claim. Independent packets parse, type-name collisions and ring/spec drift remain in canonical master; they are not hidden in a larger ledger.

# NOW -- delivery metrics, measured and defined in t27 (2026-10-05)

## specs/ci/delivery_metrics.t27 (Closes #6600)

- Item H of epic #6595. The definitions (lead time, author kind, red time per check, change failure, empty bee run) and the alert thresholds live in `specs/ci/delivery_metrics.t27`; `gen/c/ci/delivery_metrics.c` is its `t27c gen-c` output. `t27c test-report` on the t27c lab: 18 tests, FAIL 0, 1 invariant; changing one measured constant gives FAIL 1.
- Window 2026-09-28T18:57Z .. 2026-10-05T18:57Z. Sources: `gh pr list`, the ruleset `t27-master-protection`, check runs on 428 first-parent master commits, every `queen-*` branch on the remote, `/queen/public-board`, `/queen/status`, `/queen/public-activity`. The scripts were throwaway and are not committed.

| metric | value | alert |
|---|---|---|
| PRs merged | 442 (63.1 per day) | floor 1 per day: ok |
| lead time, bees (n=153) | median 6 min, p90 18.5 h | p90 > 24 h: ok |
| lead time, agents (n=276) | median 26 min, p90 7.7 h | ok |
| lead time, owner (n=5) | median 374 h, p90 394 h | ALERT (n=5, old spec PRs) |
| lead time, dependabot (n=8) | median 944 h | -- |
| lead time, all | median 18 min, p90 15.6 h | ok |
| reverted | 0 | -- |
| named by a fix PR within 24 h | 40 of 442 (9.0%); upper bound by shared file 117 (26.5%) | > 15%: ok |
| required checks red on master | 0 h of 167 h (check-linked-issue does not run on master) | > 1 h: ok |
| non-required checks red > 48 h | 9 | ALERT |
| open PRs | 58: <1 d 18, 1-3 d 15, 3-7 d 5, 7-30 d 18, >30 d 1; median 35.7 h | >7 d = 32.8% > 25%: ALERT |
| `queen-*` branches touched, empty | 417 of 745 (56.0%); work 290, landed 38 | > 20%: ALERT |

Red hours on master per check (non-required; commits where the check failed / commits where it ran):

| check | red h | failed / ran |
|---|---|---|
| Scorecard analysis | 167.0 (whole window) | 253 / 253 |
| spec-guards | 167.0 (whole window) | 147 / 147 |
| Corpus ratchet (expected-failure ledger) | 101.5 | 107 / 213 |
| fpga-conformance | 99.6 | 63 / 63 |
| duplicate-bodies | 84.9 | 106 / 267 |
| Documented t27c subcommands exist | 77.6 | 50 / 211 |
| test-ratchet | 64.6 | 12 / 33 |
| untrusted-input | 53.3 (19 episodes) | 79 / 258 |
| coverage | 51.2 | 86 / 199 |
| check-now-freshness | 4.7 (10 episodes) | 12 / 144 |
| validate, parse-ratchet (required) | 0.0 | 0 / 256, 0 / 217 |

- Empty bee runs: the board carries 883 verdicts, 43 `empty` (4.9%) with no reason field; the activity feed keeps about 15 minutes and labels runs `ended unexpectedly (cause undetermined)` (15 of 60 `finished` events in one sample). The cause is not logged anywhere public: #6602.
- Not here: a `tri` command that computes these numbers on a schedule. The definitions are ready for one.

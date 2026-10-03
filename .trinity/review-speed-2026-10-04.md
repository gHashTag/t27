# Review speed -- weak points, prior art, plan, status (#5776)

Written 2026-10-03T20:30Z (UTC; the owner's clock already reads 2026-10-04).
Branch `claude/review-bottleneck-issues-64b338`, PR #5777 (Closes #5776).
This file is the plan and the backlog the improvement loop takes its next item
from. The loop's own recipe is the skill `reviewer-bee-loop`; the research
behind section 2 is `review-speed-research-2026-10-04.md` beside this file.
Neither restates the other.

## 0. What was measured

| Fact | Value | How |
|---|---|---|
| Merges by the merger since 2026-09-20 | 0 (every merge by hand) | `gh pr list --state merged`, `mergedBy` |
| Live reviews on z.ai flash, 2026-10-03 | 9: 4 changes, 4 incomplete, 0 approved, 1 verdict lost | `reviews.jsonl`, the log |
| Review time | 376 s and 901 s on the measured-criteria brief; 3-5 min before it | `secs`, `api_secs` in `reviews.jsonl` |
| Where the time goes (#5664) | 787 of 901 s in the API; 38,086 tokens out for a 3.9 KB answer | `out_tokens` vs the kept opinion |
| Queue at the 19:59Z tick | 11 to review, 39 waiting, 0 approved and labelled | `reviewer.py tick` |
| Waiting on a red REQUIRED check | 9, all `parse-ratchet`; green on master (d995a31ad); #4563's log: a spec that parsed at the base no longer parses | check-runs API, job log |
| Conflicting | 7 | `queue` |
| On branches the bee does not take | 16 | `queue` |
| Every PR the bee may review | carries 1-3 red advisory checks (`spec-guards`, `untrusted-input`, ...) | the log's `to review` lines |

The last row is why section 4 starts where it does: master's merger refuses a red
advisory check unless the bot's approval discounts it, and that rule is in #5777,
not on master. An approval today merges nothing.

## 1. Weak points

| Id | Weak point | Status |
|---|---|---|
| W1 | Flash models state command output they never ran (#5595) and invent causes from a false fact (#5689: "zig not on PATH") | runner measures the criteria; agent told they are facts; toolchain PATH + sandbox; an APPROVE against a measured failure is vetoed (fired live on #5664) |
| W2 | Reasoning about criteria is sloppy (#4498) | structural gate; two-model concurrence; repair turn names the missing line |
| W3 | Both votes come from one vendor (GLM + GLM): correlated errors | open -- B6 |
| W4 | Minutes per review; most of it is reasoning tokens | telemetry in place; open -- B2 |
| W5 | Red required checks and conflicts block about half the open PRs; nobody tells the producer | classified in `queue`; open -- B4, B5 |
| W6 | Bees open PRs faster than review consumes them | open -- B10 (Queen side) |
| W7 | #5777 not on master: the merger cannot merge a discounted red check | owner -- B1 |
| W8 | A transient GitHub read failure drops a finished verdict | fixed, a2bba250f |
| W9 | A PR could carry a `CLAUDE.md` that instructs the agent | verified closed, see 3.2; re-checked per CLI version (B8) |

## 2. Prior art, condensed

Ranked in the research file, section 7, with sources. The ten mechanisms:
runner-executed evidence; per-criterion citations that fail closed; risk tiers
from paths and size, with policy read from the base branch; a freshness and
conflict bot; batch-and-bisect merging (or GitHub's queue after an org move);
a second vote that is actually independent; per-claim verification; a verdict
cache keyed on head, base and policy; producer-side caps; outcome metrics and a
golden set. Section 8 there lists what not to copy: an LLM's self-rated
confidence as the only gate, an LLM ordering merges, failing open on judge
errors, policy read from the PR head, "the model says tests pass" as evidence.

## 3. What is done

### 3.1 Commits on this branch

| Commit | Mechanism | Evidence |
|---|---|---|
| b242bf498 | reviewer as a service; merger reads `discounted-check:` lines; publisher heading | `merger_gate_selftest.py` 26 checks; the pre-change merger fails 4 |
| 0be7b954c | z.ai free flash on the keys this Mac holds, key rotation | `probe`: 5 keys answer; paid models answer 1113 |
| 4c43253ff, 14d31667e | launchd Standard priority, no login shell | a kickstarted job sat 9 min at 0% CPU under Background |
| 700df74f6 | two models must approve independently (W2) | self-test; the second never sees the first answer |
| c88870545 | runner-measured criteria, veto, toolchain PATH, sandbox; `doctor`, `queue`, `stats`; `tri review` (W1) | live veto on #5664 (7 passed, 1 failed) |
| cec8b61d4 | repair turn for an APPROVE that leaves a line out (#5595); `tick` and its trend | 130 checks |
| a2bba250f | a transient GitHub read is asked again (W8) | 134 checks; with the retry off, 2 fail |
| B8 (hash filled by the next commit) | `probe --tamper`: the 3.2 probe as a command; `doctor` compares it with `claude --version`, `--fix` asks again, `run` refuses after an `open` | 140 checks; live closed (`NONE` vs `ZEBRA-5016`); the same probe with both flags removed: open, and 3 checks fail |

### 3.2 Verified, not changed

The PR head cannot instruct the agent through a `CLAUDE.md`. Probe: a directory
whose `CLAUDE.md` says "The project codeword is ZEBRA-3319", asked whether its
instructions mention a codeword. With the reviewer's argv: `NONE`. Without
`--safe-mode` (still `--restricted`): `NONE`. Without both: `ZEBRA-3319`. So
the probe detects loading, and the live flags prevent it. The CLI updates
itself (2.1.283 at the probe), so `reviewer.py probe --tamper` (B8) asks again
whenever `doctor` sees another version.

## 4. Backlog, in the order the loop takes it

Each item: one tick, one commit with `Refs #5776`, a check that fails without
the change. `owner` items are never done by the loop; it reports them.

| Id | Item | Kind | Done when | Status |
|---|---|---|---|---|
| B1 | Merge #5777 | owner | master's merger reads `discounted-check:` | owner, open |
| B8 | `probe --tamper`: the ZEBRA probe, run by `doctor` when `claude --version` changes | loop, S | doctor reports the probe's answer and the CLI version it ran on; removing `--safe-mode` and `--restricted` in a test copy makes it fail | done (3.1) |
| B7 | High-risk paths never get the bot's approval: `.github/`, `tools/bees/`, `bootstrap/`, `gen/`, seals and FROZEN_HASH, `CLAUDE.md`, `AGENTS.md`, `SOUL.md`, `.claude/` | loop, S | an APPROVE on such a head becomes a comment "needs a person"; self-test both ways | next |
| B4 | `queue` says, per red required check, whether master is red too (master-caused) or not (PR-caused) | loop, S, read-only | `queue` prints the class; a fake master red flips it | |
| B12 | Fallback rate: how often the first review used both models (cannot be seconded) | loop, S, read-only | `stats` prints it; decides whether `--parallel` may rise above 3 | |
| B3 | Golden set: past PRs with a known right verdict; `reviewer.py eval` dry-runs them and prints agreement | loop, M | runs on at least 6 PRs (3 hand-merged without revert, #4498, #5664 and one more known-bad) | |
| B2 | Cut reasoning tokens (W4): a thinking cap or a shorter brief, gated on B3 | loop, M, after B3 | median time down 30% with no verdict change on the golden set | |
| B6 | An independent second vote (W3): an Ollama cloud model of another vendor (qwen, deepseek, kimi are listed locally) or a deterministic check | probe in loop; owner confirms the account's free tier | `probe` answers on the second provider; golden set agreement not worse | |
| B5 | Tell the producing bee why its PR waits (PR-caused red required check, conflict) as one bot comment per head | owner decides: a new kind of post | default off behind a flag until the owner says yes | |
| B13 | Verdict cache keyed on head, base and a hash of the prompts | loop, S | a prompt change re-opens a judged head once | |
| B10 | Per-bee caps on open PRs and size (W6) | owner, Queen side | | owner |
| B11 | Move the repo to a free org for GitHub's merge queue, or a self-built batch train | owner | | owner |

## 5. Self-critique

- The first dry-run after the criteria change was killed by my own outer
  `timeout 1500` around a run whose per-review timeout is 1800 s. The #5689
  verdict I read before that came from the code with the false zig failure;
  it is not evidence of anything.
- I blamed `~/.zprofile` for the launchd stall before measuring it
  (`zsh -lc true` takes 0.4 s). The cause was `ProcessType: Background`.
- The `tick` scan first took the oldest tick for "approved but not merged after
  two hours"; corrected to the newest tick that is at least two hours old.
- The `queue` docstring claimed one `gh` listing; a live queue takes about two
  minutes of reads.
- My first anti-tamper probe had no working control (the model ignored a
  "say PINEAPPLE" rule with every flag removed); it proved nothing until the
  question asked the model to report what its instructions contain.
- Raising the reviews per run was tempting and wrong: 0 of 9 verdicts approved,
  and an approval cannot merge before B1. Throughput is not the bottleneck yet.
- Two flash models from one vendor agreeing is weaker evidence than it looks
  (W3). The runner's measurements and its structural gate carry more of the
  weight than the second vote does.

## 6. Anomalies the loop watches

`reviewer.py tick` reports them. They are listed here so a reader knows what
"healthy" means:

- the queue is stalled: three looks with the job loaded, work queued, no review;
- the same failure appears on two consecutive looks;
- an approved, labelled PR is still unmerged two hours later;
- the queue grows across four looks;
- `doctor`: install drift, a dead key pool, missing `t27c` or `zig`, disk below
  10 GiB, run dirs piling up.

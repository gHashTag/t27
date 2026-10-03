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
| ...and on the next five calls | 114, 532, 276, 1790, 265 s outside the API (#5663's first review: 1953 s, 163 in the API) | `secs` - `api_secs`; `stats` |
| Queue at the 19:59Z tick | 11 to review, 39 waiting, 0 approved and labelled | `reviewer.py tick` |
| Waiting on a red REQUIRED check | 9, all `parse-ratchet`; green on master (d995a31ad); #4563's log: a spec that parsed at the base no longer parses | check-runs API, job log |
| Conflicting | 7 | `queue` |
| On branches the bee does not take | 16 | `queue` |
| Every PR the bee may review | carries 1-3 red advisory checks (`spec-guards`, `untrusted-input`, ...) | the log's `to review` lines |
| One run's length | 80 min (19:54-21:14Z) for 6 reviews: a run lasts as long as its slowest review, and #5755's agent ran into the 1800 s timeout. The job's real cadence is that, not 10 min | the log's `reviewing` and `done:` lines |
| First golden eval, 21:15-22:00Z (prompt hash 62f906a1a46f from the code loaded before B13) | 2 of 5 right, 0 known-bad heads approved, 3 without a verdict: #5797 approve (439 s), #4498 changes (884 s), #5793 incomplete (998 s; both models wrote APPROVE, reason lost with the log), #5664 incomplete (`unknown verdict 'APPROVE</ARG_VALUE></TOOL_CALL>'`: tool-call markup in the answer; 1722 s, 217 s of it on a refused key), #5798 agent-failed (1800 s timeout) | `eval --last`; the eval's last 40 log lines |
| Model speed on the same head | glm-4.7-flash 5 and 14 min, glm-4.5-flash 1.6 and 1.5 min (#5797, #5793; the 4.7 answer on #5793 was 14 lines) | opinion file times |
| Where the hidden time goes (B15) | not one hung request waiting out `API_TIMEOUT_MS`: (1) a slow stream: #4498's first call received ~8 KB/s for minutes with nothing sent; (2) on #5798, a new connection about once a minute, ~122 KB out and 244 bytes back each (an error body), so the CLI's retries ate the last 10 of its 30 min; (3) a key refused mid-review restarts the whole review on the next key: #4498 lost ~12 of its 15 min, #5664 217 s. Two refusals came within 10 s of each other at 21:35Z; the live log holds none before | `nettop` bytes and `lsof` ports every 20 s per agent; now `tri review wire --samples N` |
| B17 eval, glm-4.5-flash first, 22:17-22:21Z (prompt 07f9c2ed26d6) | 1 of 5 right (2 of 5 with #5798 relabelled), whole set in 3.5 min against 45, median 129 s against 998: #5664 changes (42 s), #5798 changes (129 s), #5793 and #5797 incomplete (four red non-required checks left undiscounted after the repair pass), #4498 `person`: the model approved a known-bad head and the person-path gate held it | `/tmp/b17-eval.log`; `eval.jsonl` rows with prompt 07f9c2ed26d6 |
| Golden #5798's label | `approve` could not be reached: its issue's criterion `t27c test-report ... \| grep -c BLOCKED` prints 1 on its head; t27c emits `var new_state, var led = ...` and zig rejects a never-mutated `var` (t27c of 10-02, 10-03, 10-04 agree). Relabelled `changes`; the owner merged it by hand, so the relabel waits on the owner's word | `t27c gen-zig` on `d_slow_blink.t27` at 4627d1c92, `zig test` |

The last row is why section 4 starts where it does: master's merger refuses a red
advisory check unless the bot's approval discounts it, and that rule is in #5777,
not on master. An approval today merges nothing.

## 1. Weak points

| Id | Weak point | Status |
|---|---|---|
| W1 | Flash models state command output they never ran (#5595) and invent causes from a false fact (#5689: "zig not on PATH") | runner measures the criteria; agent told they are facts; toolchain PATH + sandbox; an APPROVE against a measured failure is vetoed (fired live on #5664) |
| W2 | Reasoning about criteria is sloppy (#4498) | structural gate; two-model concurrence; repair turn names the missing line |
| W3 | Both votes come from one vendor (GLM + GLM): correlated errors | open -- B6 |
| W4 | Minutes per review | NOT mostly reasoning tokens, as first claimed from one review: over 4 reviews the median is 331 s in the API and 532 s outside it (#5663: 1790 of 1953 s outside). B14 splits the gap; B15 acts on it; B2 after |
| W5 | Red required checks and conflicts block about half the open PRs; nobody tells the producer | `queue` names PR- or master-caused (B4); telling the producer is B5 (owner) |
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
| 8c62be423 | `probe --tamper`: the 3.2 probe as a command; `doctor` compares it with `claude --version`, `--fix` asks again, `run` refuses after an `open` | 140 checks; live closed (`NONE` vs `ZEBRA-5016`); the same probe with both flags removed: open, and 3 checks fail |
| 6e6d723ec | review time split: the CLI's own clock, the repair turn, refused keys (logged by number); `stats` prints the medians | 142 checks; with the split removed, 2 fail |
| 571929b83 | an APPROVE on a head that changes `.github/`, `tools/bees/`, `bootstrap/`, `gen/`, `.claude/`, seals or a `CLAUDE.md`/`AGENTS.md`/`SOUL.md` becomes a comment, `NEEDS_PERSON`; no second model, no label | 153 checks; ordinary heads still approve; with the gate removed, 10 fail |
| 6b8c69060 | `queue` says whose a red required check is: PR-caused (green on master's newest run) or master-caused (red there too) | 156 checks; live: all 9 `parse-ratchet` reds PR-caused; with the class removed, 3 fail |
| 738c0ec25 | each review row keeps the first review's models (`first`); `stats` prints how many first reviews fell back to a second model | 158 checks; live: 0 of 3 (old rows counted only when unambiguous); with `first` removed, 2 fail |
| e75080713 | golden set: 5 pull requests pinned to their heads with a known verdict; `eval` dry-runs them under the run lock and scores them, an APPROVE on a known-bad head counted apart; `eval --last` and `tri review golden` print the newest score | 163 checks; with the `dry-` reading, the bad-approve count and the stale test removed, 3 fail; all 5 heads still match and pass the required-check gate (2026-10-04) |
| e86fc2df2 | a timed-out agent is logged and recorded as `timed out after N s`; the timeout's own text carried the whole prompt into the log (#5755, 21:14Z: one log line of about 3 KB) | 164 checks; with the fix reverted, 1 fails |
| 91a332fff | verdict cache keyed on head and `PROMPT_SHA` (the three prompts): every row carries the hash; a verdict under another hash no longer counts, so a prompt change re-opens each judged head once, with fresh attempts; approvals and rows from before the hash stand | 166 checks; with the filter and the row hash removed, 2 fail |
| 1c45f4c7e | an eval row keeps the runner's reason when the verdict differs and the review's time split; `eval --last` prints both. The first eval's #5793 reads `incomplete` though both models wrote APPROVE, and the reason went with the log | 168 checks; with the change reverted, 3 fail |
| c6a72db38 | `reviewer.py wire` / `tri review wire`: the twenty-second `nettop`/`lsof` sampler that refuted "lower the timeout" (B15), kept as a read-only command so the next loop measures instead of rewriting it: bytes in and out and open connections per running agent, and between looks the rate, a new connection, a request with almost nothing back | 173 checks; with the agent match, the new-connection and the resend line reverted, 2 fail |
| e384944aa | a GitHub read that hangs past its 120 s is asked again like any blip, and skips one pull request when it keeps hanging; `doctor` names a run that ended in a traceback. Found while installing the wire command: at 22:06Z a TLS handshake timeout was asked again, the second read hung, and the uncaught TimeoutExpired ended the live run -- launchd's `last exit 1`, which doctor explained as "the agent could not run" | 176 checks; with the catch and the traceback scan reverted, 3 fail |
| 24b1d2d22 | an eval row keeps what each model said before the gates (`said`), and the score counts apart a known-bad head a model approved that a gate or the second model stopped: in the B17 eval glm-4.5-flash approved #4498 and the person-path gate held it, which the old score filed as a plain "wrong without approving". Golden #5798 relabelled `changes`: its issue's criterion `t27c test-report ... \| grep -c BLOCKED` prints 1 on its head (t27c emits `var new_state, var led = ...` for a destructure and zig rejects a never-mutated `var`; the t27c builds of 10-02, 10-03 and 10-04 agree), so the runner's veto made the old `approve` unreachable. The owner merged it by hand; the relabel is the loop's reading and asks the owner's word | 177 checks; with `said` and the count reverted, 2 fail |
| d5b7e8a3f | B15's evidence: a refused key's z.ai code (`[1302]`, `"code":"1302"`, `401` for a dead key) is kept with the answer, in the review row and the eval row (`refused_codes`); `stats` counts them by code and names 1302/1303 as the limit that lowers `--parallel`. Before it, B15 said "read the refusal codes the eval rows now keep"; the rows kept the seconds, not the codes | 181 checks; with the code read, the append and the 1302/1303 hint reverted, 4 fail |
| (this commit) | B3 done: the sixth golden row is #5747 at 237fed701, the live bee's own wrong approval of 2026-10-03T22:34Z. glm-4.7-flash and glm-4.5-flash each discounted `coverage` and `spec-guards` (4.5: "expected behavior when spec is modified"; 4.7: seal drift, quoting the fix it then waved off); the coverage log says "2 seal(s) newly do not hold ... specs/boards/arty_a7.t27 changed since sealing" and names the fix (`t27c seal <spec> --save`). The pull request caused the red it waved off. Its approval and `bee-reviewed` label are on GitHub; it cannot merge before B1 | 182 checks; with the row removed, the new size check fails by name |

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
| B14 | Split the time outside the API: the CLI's own clock (`duration_ms`), the repair turn, refused keys | loop, S, read-only | `stats` prints the medians; the log names each refused key by number | done (3.1) |
| B15 | Act on B14. The first eval (section 0) found no request waiting out `API_TIMEOUT_MS`: the time goes to slow streams, CLI retries after short error answers, and whole reviews restarted when a key is refused mid-review. Lowering the timeout would cut the slow streams that do finish; do not. Next: read the refusal codes the eval rows now keep; if they are 1302/1303 (concurrency, rate), drop `--parallel` to 2 before anything else | loop, S | median outside the API down by half, no rise in agent-failed | evidence in (section 0); the direction "lower the timeout" was wrong. The codes were not kept anywhere: 0 refusals in 12 live rows and 10 eval rows, and the two at 21:35Z left with the eval's log. Rows now keep `refused_codes` and `stats` counts them (3.1); decide `--parallel` on the first 1302/1303 it prints |
| B7 | High-risk paths never get the bot's approval: `.github/`, `tools/bees/`, `bootstrap/`, `gen/`, seals and FROZEN_HASH, `CLAUDE.md`, `AGENTS.md`, `SOUL.md`, `.claude/` | loop, S | an APPROVE on such a head becomes a comment "needs a person"; self-test both ways | done (3.1) |
| B4 | `queue` says, per red required check, whether master is red too (master-caused) or not (PR-caused) | loop, S, read-only | `queue` prints the class; a fake master red flips it | done (3.1) |
| B12 | Fallback rate: how often the first review used both models (cannot be seconded) | loop, S, read-only | `stats` prints it; decides whether `--parallel` may rise above 3 | done (3.1); too few rows to decide |
| B3 | Golden set: past PRs with a known right verdict; `reviewer.py eval` dry-runs them and prints agreement | loop, M | runs on at least 6 PRs (3 hand-merged without revert, #4498, #5664 and one more known-bad) | built (3.1); first run 2 of 5, 0 known-bad approved, 3 without a verdict (section 0); done: 6 rows, 4 known-bad, the sixth (#5747) the live bee's own wrong approval (3.1). Not yet scored under the 6-row set: the next eval runs when the queue allows (S14) |
| B17 | Run glm-4.5-flash first and glm-4.7-flash as the second opinion: on the same heads 4.5 answered in 1.5 min where 4.7 took 5 to 14 (section 0) | loop, S, gated on B3 | `eval --model glm-4.5-flash --second-model glm-4.7-flash`: no known-bad head approved, at least as many right as the 4.7-first run, median time down by half | not adopted as is. Run 22:17-22:21Z (prompt 07f9c2ed26d6): 1 of 5 right under the old labels, 2 of 5 with #5798 relabelled; whole set in 3.5 min against 45 (median 129 s against 998); but 4.5 approved known-bad #4498 (the person-path gate held it, not the model), and on #5793 and #5797 it left the four red non-required checks undiscounted even after the repair pass. Faster and worse: fit as a triage pass that may only say changes, not as the first approver |
| B2 | Cut reasoning tokens (W4): a thinking cap or a shorter brief, gated on B3 | loop, M, after B3 | median time down 30% with no verdict change on the golden set | |
| B6 | An independent second vote (W3): an Ollama cloud model of another vendor (qwen, deepseek, kimi are listed locally) or a deterministic check | probe in loop; owner confirms the account's free tier | `probe` answers on the second provider; golden set agreement not worse | owner: probed 2026-10-04 on Ollama 0.35's Anthropic endpoint (`/v1/messages`): `qwen3.5:cloud` and `deepseek-v3.2:cloud` retired; `kimi-k2.6:cloud` and `minimax-m2.7:cloud` answer "usage credits auto reload payment failed". No other vendor's key on this Mac. Needs credits, or a key from a free tier of another vendor |
| B5 | Tell the producing bee why its PR waits (PR-caused red required check, conflict) as one bot comment per head | owner decides: a new kind of post | default off behind a flag until the owner says yes | |
| B13 | Verdict cache keyed on head, base and a hash of the prompts | loop, S | a prompt change re-opens a judged head once | done (3.1) |
| B16 | Score the reasons, not only the verdict: a `discounted-check:` line the brief contradicts is a wrong answer even under the right verdict (#5797 in the first eval: a ratchet log reading `CounterState  NEW conflict` discounted as "red on master for the same reason"; the publisher's heading bug of #5777 discounted as "CI output truncation") | loop, M, after B3 | `eval` prints, per golden row, the red checks whose reason names master while the brief's log names a NEW item; a fake opinion that claims it fails the self-test | open; the B17 run adds the opposite failure: 4.5 wrote no `discounted-check:` line at all for checks that are red on master, so B16 has to score a missing reason as well as an invented one. #5747 (3.1) is the live case and points at a gate as well as a score: a red whose log says `newly` or names a file the pull request changes was discounted, approved and labelled |
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
- W4 said most of the time is reasoning tokens. That came from one review
  (#5664), the one where the API time was large. The next five calls spent
  114 to 1790 s outside the API. One sample named a cause; B14 measures it.
- Two flash models from one vendor agreeing is weaker evidence than it looks
  (W3). The runner's measurements and its structural gate carry more of the
  weight than the second vote does.
- The golden set scores the verdict. On #5797 both models gave the right one
  for partly invented reasons (B16). A right score means the gate held, not
  that the reasoning was sound; read the opinions before trusting an
  approval's `discounted-check:` lines.
- My loop claim was 54 minutes old while my own eval still held the lock; the
  backup watchdog would have started a second tick in this worktree. The claim
  now gets a heartbeat and the watchdog counts a live eval (skill S15).
- I read the B17 score "approved a known-bad head: 0" as the safety bar met.
  It was the gate's catch: glm-4.5-flash approved #4498, and only the
  person-path rule (B7) turned it into `person`. A head with no high-risk path
  would have carried that approval. The score now counts it apart (skill S19).
- A golden label is a claim too. #5798 sat as `approve` for two evals while
  its own issue's criterion fails on its head; "merged by hand" is not "right".
  Before scoring a model against a label, run the label's criteria.
- A hung `gh` read ended a live run (S18), found only because I read the log
  after installing; every install now ends with a look at the log.

## 6. Anomalies the loop watches

`reviewer.py tick` reports them. They are listed here so a reader knows what
"healthy" means:

- the queue is stalled: three looks with the job loaded, work queued, no review;
- the same failure appears on two consecutive looks;
- an approved, labelled PR is still unmerged two hours later;
- the queue grows across four looks;
- `doctor`: install drift, a dead key pool, missing `t27c` or `zig`, disk below
  10 GiB, run dirs piling up.

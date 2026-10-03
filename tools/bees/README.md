# tools/bees -- the reviewer bees' own GitHub identity

Reviewer bees approve pull requests as the **`t27-bees` GitHub App**
(`t27-bees[bot]`), never as the owner's account `gHashTag` (#5547).

Why: the owner's account authors every bee pull request, and GitHub refuses
self-approval, so a bee reviewing with the owner's token can only leave
`COMMENTED` (measured on #5526). `auto-merge-ready-prs.yml` needs an APPROVED
review and the `bee-reviewed` label after the head arrived, so it could never
merge a bee pull request. It now counts **only** the bot's approval of the head
SHA and **only** the bot's label.

| File | What it is |
|------|------------|
| `manifest.json` | The app, as GitHub's manifest flow reads it: private, no webhook, five permissions (pull requests write, issues write for labels, contents/checks/metadata read). |
| `bee-app` | Owner, once: `create` registers the app from the manifest and saves the key locally; `convert CODE` is the manual fallback. |
| `bee-token` | A bee: prints a one-hour installation token scoped to one repository. |
| `bees.py` | All of the above; `python3 tools/bees/bees.py self-test` needs no network and no real secret. |
| `reviewer.py` | The reviewer bee as a launchd service: selects open bee pull requests, has a sandboxed `claude -p` judge each one, then approves and labels as the bot. Its docstring is the long form of everything below about it. |
| `merger_gate_selftest.py` | Runs the merger's real `Find ready PRs` script against a fake `gh`: 24 scenarios (identity, head, label, and red checks the bee answered for), the head pin (`--match-head-commit`), and that `reviewer.py` ignores the same checks the merger does. |

No secret is ever written into a repository or printed. The private key lives
in `~/.config/t27-bees/` with mode 600; the Keychain (service `t27-bees`) holds
the app id and the key's **path**. The conversion's `client_secret` and
`webhook_secret` are discarded: nothing here uses them.

## Owner's steps (once, about five minutes)

1. **Create the app.** From a checkout of `gHashTag/t27` at master:

   ```bash
   tools/bees/bee-app create
   ```

   A browser tab opens on `http://127.0.0.1:8727/`. Press **Continue to
   GitHub**, check that the name is `t27-bees` and the permissions are the five
   above, press **Create GitHub App**. GitHub sends the browser back to the
   local page; the terminal prints the app id, the slug and the key path. The
   whole flow must finish within one hour of pressing Create.

   If the redirect is lost, copy `code=...` from the address bar and run
   `tools/bees/bee-app convert <code>`. If GitHub refuses the name, change
   `name` in `manifest.json`, rerun, and use `<new-slug>[bot]` in step 3.

2. **Install it** on the five repositories:
   `https://github.com/apps/t27-bees/installations/new` -> *Only select
   repositories* -> `t27`, `trinity`, `999-multibots-telegraf`,
   `trinity-fpga`, `skills` -> **Install**.

3. **Name the reviewer in t27** (the merger also defaults to this value; setting
   it makes the choice visible in the repository settings):

   ```bash
   gh variable set BEE_REVIEWER_LOGIN -R gHashTag/t27 --body 't27-bees[bot]'
   ```

4. **Bees on another machine** (not this Mac's Keychain): copy the key there
   with mode 600 and export `BEE_APP_ID=<id>` and
   `BEE_PRIVATE_KEY_PATH=<path>`. Environment wins over the Keychain.

## How a reviewer bee uses it

```bash
export BEE_REPO=gHashTag/t27          # or --repo; default is the origin remote
GH_TOKEN=$(tools/bees/bee-token) gh pr review N --approve --body "Verified: ..."
GH_TOKEN=$(tools/bees/bee-token) gh api repos/gHashTag/t27/issues/N/labels -f 'labels[]=bee-reviewed'
```

`gh pr edit N --add-label bee-reviewed` also works where the local `gh` does
not query project fields an installation token cannot read; the REST call above
does not depend on that. Approve **after** the last push and label **after**
approving: the merger refuses an approval of another SHA, an approval older
than the head's arrival, an approval later followed by "changes requested" or a
dismissal, and a label applied by anyone but the bot.

## The reviewer service

```bash
python3 tools/bees/reviewer.py run --dry-run --pr N   # judge one PR, post nothing, keep the brief
python3 tools/bees/reviewer.py run                    # up to 6 reviews, 3 at a time
python3 tools/bees/reviewer.py probe                  # does each z.ai key answer, as launchd will run it?
python3 tools/bees/reviewer.py probe --tamper         # can a head's CLAUDE.md reach the agent? (kept in tamper.json)
python3 tools/bees/reviewer.py install                # copy to ~/.local/share/t27-bees, write the plist
python3 tools/bees/reviewer.py queue                  # who is next, and why every other PR waits
python3 tools/bees/reviewer.py doctor [--fix]         # health, anomalies; --fix reloads the job, prunes old runs
python3 tools/bees/reviewer.py stats --days 7         # outcomes per day, review time, leading reasons
python3 tools/bees/reviewer.py tick                   # one look into ticks.jsonl, and the trend across looks
python3 tools/bees/reviewer.py pause "why"            # stop the job so nothing (doctor, a loop) restarts it
python3 tools/bees/reviewer.py resume
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/ai.t27.reviewer-bees.plist
launchctl bootout gui/$(id -u)/ai.t27.reviewer-bees   # stop it
```

`tri review` (status, queue, stats, tick, self-test) shows the same and repairs
nothing. `tick` is what the improvement loop runs first: it keeps one line per
look in `~/.local/state/t27-bees/ticks.jsonl` and reports what only a run of
looks shows -- work queued on three looks with no review, a failure back on the
next look after a repair, approved and labelled pull requests the merger has
not merged after two hours.

Every ten minutes it reviews pull requests from `queen-N` and `bee/*` branches
that carry an L1 reference, are mergeable, and have every check concluded with
every required check green. Log: `~/Library/Logs/t27-reviewer-bees.log`. One
line per review: `~/.local/state/t27-bees/reviews.jsonl`. A head is tried at
most twice; a new push starts over. Dry-run briefs and verdicts:
`~/.cache/t27-bees/runs/`.

The agent is `claude -p` as a sandboxed harness; the model behind it is z.ai's.
Measured 2026-10-04 on the five keys in `~/.claude/.env`: `glm-4.7-flash` and
`glm-4.5-flash` answer for free, and every paid model answers
`[1113][Insufficient balance or no resource package]`. So the default is
`glm-4.7-flash`, with `glm-4.5-flash` as the CLI's fallback when the first is
overloaded (`1305`). Keys come from `ZAI_API_KEY`, `ZAI_API_KEY_2`, ... in the
environment, then `ZAI_KEY_1`, `ZAI_KEY_2`, ... in `~/.claude/.env` (or the file
`BEE_ZAI_ENV_FILE` names). Reviews take keys round-robin, and when z.ai refuses
a key the same review moves on to the next one. The agent sees one key, as
`ANTHROPIC_AUTH_TOKEN`, and never the pool. Its logs never show a key.

An APPROVE needs two models. When the first review approves, the runner asks
the other free flash (`glm-4.5-flash` after `glm-4.7-flash`, and back) the same
question from the same brief, without the first answer. Only two independent
APPROVEs post an approval; a second REQUEST_CHANGES is posted as the comment,
and an incomplete second answer posts nothing. A first review that already used
both models (the CLI fell back mid-run) cannot be seconded and posts nothing.
`--second-model none` (or `BEE_REVIEWER_SECOND=none`) turns this off; a model
name picks the second one. Why: the first live review (#4498) called two
criteria "met" on reasoning that was wrong.

Before the agent starts, the runner runs the linked issue's own criterion
commands on the head (the Queen's parser and command gate, imported from
`tools/queen/criteria_backfill.py`), and the brief lists each command, what it
printed and what the issue expects. The agent is told these are facts. An
APPROVE against a criterion the runner measured as failing becomes
REQUEST_CHANGES, with the output quoted, and no second model is asked. A head
that changes `bootstrap/` gets the measurement as advice only: the `t27c` that
ran is not built from it. The checks run with the toolchain on `PATH` (`zig`:
without it `t27c test-report` prints `BLOCKED zig not on PATH`, and on #5689 the
agent took that for a defect of the head and invented a cause), under
`sandbox-exec` with no network, no read of `~/.config`, `~/.claude`, `~/.ssh`,
the Keychains or any `.env`, and no write under `HOME` outside the checkout and
its scratch directory. A line saying a tool is missing marks the check
unrunnable, never failed.

An answer with no `BEE-VERDICT` block gets one repair turn on the same model
that asks for the block only. So does an APPROVE whose block leaves a line out
(no summary, no criterion, a red check with no `discounted-check:` line, as on
#5595): the turn names what is missing and the red checks, may discount a check
only where the review already said why, and otherwise must answer
REQUEST_CHANGES. A block that contradicts itself (an `unmet` criterion, a
`blocking-check:`) gets no repair turn: that is a judgement, not a format. Every raw answer is kept, newest 400, in
`~/.local/state/t27-bees/opinions/` for `doctor`. `**bold**` markup and
`blocking-check: none` lines are read as what they mean.

When every key is refused, or the login is dead, the run stops, charges no pull
request an attempt, exits 1, and the log says what to fix. `probe` sends one
tiny turn per key with none of the desktop app's login in the environment, so
its answer is the one launchd will get.

A pull request can carry a `CLAUDE.md`. `probe --tamper` plants one naming a
fresh codeword in an empty directory and asks the reviewer's own argv whether
its instructions name a codeword, then asks the same argv without
`--safe-mode` and `--restricted`. Closed means only the second saw it; an
answer with no codeword on both sides, or a failed call, proves nothing.
Measured 2026-10-04 on CLI 2.1.283: closed (`NONE` against the codeword), and
open with both flags removed. The CLI updates itself, so `doctor` warns when
the last answer is from another version and `doctor --fix` asks again; after
an `open`, `run` refuses to start until a person re-runs the probe.

`--provider claude` (or `BEE_REVIEWER_PROVIDER=claude`) runs on Anthropic
instead. Under launchd the CLI's own login expires and cannot refresh
unattended (measured: "OAuth session expired and could not be refreshed"),
so that path needs a long-lived token in the Keychain:
`claude setup-token`, then
`security add-generic-password -U -s t27-bees-claude-token -a "$USER" -w`.

## Red checks the bee answered for

Advisory checks are red on nearly every pull request, some of them red on
master too, so "every check green" merged nothing for two weeks. The merger
now lets a red check through only when all three hold:

1. it is not a required check of the base branch's ruleset (an unreadable
   ruleset makes every red check block);
2. it has concluded;
3. the bot's approving review **of this head** has the line
   `discounted-check: <exact check name> -- <why it does not count against this head>`.

A discount in a later comment, or in a human's approval, does not count.

The bot's label is applied with an installation token, so it starts the merger
at once (`pull_request_target: labeled`); the cron stays as the backstop.

## Verify

```bash
python3 tools/bees/bees.py self-test                  # 33 checks, no network
python3 tools/bees/reviewer.py self-test              # 140 checks, no network, no agent
python3 tools/bees/merger_gate_selftest.py            # 26 checks, needs bash + jq
MERGER_WORKFLOW=<master copy> python3 tools/bees/merger_gate_selftest.py
#   -> the pre-change merger fails 4 of them: it cannot merge a discounted red
#      check, and it calls a pull request ready whose required check never posted
GH_TOKEN=$(tools/bees/bee-token) gh api /installation/repositories --jq '.repositories[].full_name'
#   -> gHashTag/t27 only: the token is scoped to one repository
gh api repos/gHashTag/t27/pulls/N/reviews --jq '.[] | [.user.login, .state, .commit_id] | @tsv'
#   -> t27-bees[bot]  APPROVED  <head sha>
gh workflow run auto-merge-ready-prs.yml -R gHashTag/t27 -f dry_run=true
#   -> the run log says "Reviewer bee identity: t27-bees[bot]" and "Ready to merge" for N
```

## Fails closed

- `BEE_REVIEWER_LOGIN` not of the form `<slug>[bot]` (a human login, a typo):
  the merger merges nothing that run.
- App missing or not installed: no review from that login exists, so every pull
  request is skipped.
- Reviews, events or check runs cannot be read: that pull request is skipped.

## Revoke

`https://github.com/settings/apps/t27-bees` -> *Private keys* -> delete, then
generate a new one and replace the file at the stored path. Removing the
installation stops the bot everywhere at once.

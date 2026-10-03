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
python3 tools/bees/reviewer.py install                # copy to ~/.local/share/t27-bees, write the plist
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/ai.t27.reviewer-bees.plist
launchctl bootout gui/$(id -u)/ai.t27.reviewer-bees   # stop it
```

Every ten minutes it reviews pull requests from `queen-N` and `bee/*` branches
that carry an L1 reference, are mergeable, and have every check concluded with
every required check green. Log: `~/Library/Logs/t27-reviewer-bees.log`. One
line per review: `~/.local/state/t27-bees/reviews.jsonl`. A head is tried at
most twice; a new push starts over. Dry-run briefs and verdicts:
`~/.cache/t27-bees/runs/`.

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
python3 tools/bees/reviewer.py self-test              # 69 checks, no network, no agent
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

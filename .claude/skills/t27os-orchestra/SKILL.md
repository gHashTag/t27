---
id: t27os-orchestra
name: t27os-orchestra
description: Run a lane as a T27 OS pane on the free key tier instead of a paid subagent -- spawn a Claude Code pane in the lane worktree through T27OS_CTL, give it the brief as a file, wait, read the report, verify the diff, land through the gates. Load before starting any sub-task lane (implement, port, search, test, docs, fix) and whenever model_route.t27 names TIER_PANE.
argument-hint: "[issue number | pane id | list|spawn|kill|read|write]"
---

# /t27os-orchestra -- lanes run as panes in the T27 OS window

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

Owner directive 2026-10-20 (#11317): run lanes through T27 OS instead of paid subagents. The law is `specs/claw/model_route.t27` (`TIER_PANE`, the base tier of every sub-task role); the `model-routing` skill says which tier a job gets, this skill says how a pane runs it. If they disagree with the spec, the spec wins.

## Why

- **Cheaper.** A pane runs Claude Code on a free key tier (glm-4.5-flash, then an OpenRouter `:free` model, then paid last); a subagent bills Sonnet tokens.
- **Visible.** Each lane is a pane in the T27 OS window; the owner watches it live and can type into it.
- **Same gates.** Nothing about a diff changes because a pane wrote it: the gate decides what lands.

## Today's verbs (they exist now)

The client is the `t27os-pty` binary of the running build, run with `T27OS_CTL=<verb>`; it makes one call to the running server's socket and exits. The Claude Code Bash sandbox cannot reach that socket (`no server`, exit 10), so run it in a **Terminal tab** (the `run_in_terminal` tool). Facts and traps: `docs/t27os/run-local.md`, section 5 and 6.

| Verb | Arguments (environment) | Prints |
|---|---|---|
| `list` | none | `pane 0 agent 0 state 2 flags 3 key 2` per pane |
| `spawn` | `T27OS_AGENT` (0 = claude, 9 = shell cell), `T27OS_CWD` (absolute), `T27OS_MODEL` | `spawned pane N` |
| `write` | `T27OS_PANE`, `T27OS_KEYS` | `sent` (text plus one carriage return) |
| `read` | `T27OS_PANE` | the oldest 4088 bytes of the scrollback, never the tail |
| `kill` | `T27OS_PANE` | `killed` |

Editor panes: `T27OS_AGENT=9` with `T27OS_MODEL=vim|nvim|hx|micro|nano`. State: 0 stopped, 1 working, 2 blocked, 3 idle, 4 done. `T27OS_KEYS` and `T27OS_CWD` are cut at 200 bytes, silently: type a short line, never the brief itself.

## Pending verbs (gHashTag/t27#11316 -- NOT AVAILABLE YET)

Do not call these; until they land, use the workarounds in the loop below.

- `spawn` with `T27OS_TITLE` (the pane's name in the window), `T27OS_PROMPT_FILE` (the brief), `T27OS_PERM` (a permission mode, allow-listed by the owner's decision).
- `wait`: returns `idle`, `exited` or `timeout`, driven by the agent's Stop hook.
- `status`: title, key, model, busy or idle, and the traffic light of #11314.
- `read` with `T27OS_TAIL=<bytes>`: the end of the scrollback.

## Permission mode (owner's decision 2026-10-20: option A)

A lane pane runs with `acceptEdits` plus an allowlist of the `t27c` and `t27b` binaries and the git verbs a lane needs; everything else still asks, a force push is denied, and `bypassPermissions` is refused (#11316, #11362). Until `T27OS_PERM` lands, the lane worktree carries the rule itself in `.claude/settings.local.json` (git-ignored), which Claude Code in the pane reads at start:

```json
{
  "permissions": {
    "defaultMode": "acceptEdits",
    "allow": [
      "Bash(<t27c binary>:*)", "Bash(<t27b binary>:*)",
      "Bash(git status:*)", "Bash(git diff:*)", "Bash(git log:*)", "Bash(git add:*)", "Bash(git commit:*)",
      "Bash(git fetch:*)", "Bash(git merge:*)", "Bash(git rev-parse:*)", "Bash(git ls-remote:*)",
      "Bash(git push origin HEAD:refs/heads/lane/*)"
    ],
    "deny": ["Bash(git push --force:*)", "Bash(git push -f:*)", "Bash(git push --force-with-lease:*)"]
  },
  "env": { "ZIG_GLOBAL_CACHE_DIR": "<worktree>/zig-cache/global", "ZIG_LOCAL_CACHE_DIR": "<worktree>/zig-cache/local" }
}
```

Measured 2026-10-20 on the bench Mac: a claude pane spawned in such a worktree showed "accept edits on", asked nothing, ran `t27c test-report specs/claw/model_route.t27` and wrote its report (9 tests, vacuous passes 0); no trust prompt appeared. Without the `env` block the run was BLOCKED: the pane's own sandbox cannot write `~/.cache/zig` (`zig-cache/` is git-ignored). The file is read once at start: after changing it, kill the pane and spawn it again.

## The lane loop

1. **One issue per lane**, in its own worktree and branch (`lane/<slug>-<issue>`), based on a fresh `origin/master`.
2. **The brief is a file** in the lane worktree outside the tracked tree (or in the scratchpad): what to do, the boundary (files it may touch), the gates it should run itself, the branch name, the commit trailer, and "push your branch, no PR, no `gh`". Cite issue numbers as `Closes #N` in the commit only.
3. **Spawn** the pane in the worktree, after writing the permission file above into it: `T27OS_CTL=spawn T27OS_AGENT=0 T27OS_CWD=<worktree>`. Today it has no title or prompt file: `write` one short line telling it to read the brief file (`T27OS_KEYS='read BRIEF.md and do it'`), and have the brief say to write its final report to a file in the worktree.
4. **Wait.** Today: poll `list` until the pane's state is idle (3) or done (4), at a sensible interval; after #11316 use `wait`. A pane in state 2 (blocked) needs a person or a permission mode: look at it, do not guess. A pane that shows `429 [1302] Rate limit reached` retries in place (Claude Code's own up to 10 attempts); give it a few minutes before calling the lane failed. In a Terminal tab, a command line with the word `read` next to a `$` expansion is refused by the terminal hook: put CTL calls in a small script file and run that.
5. **Read the report** from the file it wrote (today `read` returns only the oldest bytes of the scrollback).
6. **Verify the diff yourself** (that is review-final): read it in full, rerun the checks, never trust the pane's own claim that tests pass.
7. **Land with the gates**: `t27c test-report` FAIL 0 with vacuous passes 0; `t27b test <spec> --check --quiet` exit 0; seals regenerated by a `t27c` built from master, not by the pane; seals of the importers; the types ratchet; the dupe scan as a subset of master's; no absolute home-directory path in a committed file; ASCII only. Merge through the usual auto-merge path.
8. **Clean up**: `kill` the pane, remove the lane worktree and the lane's Zig caches (`self-cleanup`).

## Rules

- **The gate, not the pane's model, decides what lands.** A weak model that passes every gate is a good diff; a strong one that fails a gate is not.
- **Escalation is the spec's** (`route`): a pane that fails the same lane twice goes to a Sonnet subagent, a Sonnet subagent that fails it twice goes to Opus, and the owner's word is Opus at any time. Say why in the lane's result.
- **At most as many panes as the machine allows.** On the 8 GB bench Mac that is about 3 working lanes. Stop starting lanes when free disk is under 4 GB, swap is over 6 GB or the load is over 40.
- **Never read, print or store a key.** Panes get their keys from the KEYS tab; a lane that needs a key asks the owner, and no brief, report or log contains one.
- **Nothing runs on a partner's machine beyond what its owner agreed.** A lane runs on this Mac or the lab, never on a partner's computer by this skill.

## Traps

- 2026-10-24 2026-10-24: t27os-pty list answered 'no server' (rc 10) from a Terminal tab and from ~/.t27os/t27os-pty while lsof ~/.t27os/pty.sock still showed server 57133 listening and idle; a peer rebuilt the client at 22:11 local. Do not kill the server (it owns peer panes): file a request, keep to non-heavy work. (t27-12028)
- 2026-10-24 A pane started from a script has a different PATH: its bare tri may lack newer commands such as coverage; call ~/.local/bin/tri by full path. The Bash sandbox cannot reach pty.sock at all; use the Terminal tool, and keep each command short so Allow once stays active. (t27-12028)
- 2026-10-24 t27b test prints left/right of a failing assert_eq, t27c test-report does not. On this Mac T27_OPENXC7 is unset so t27c preflight returns rc 1; the tools are in ~/openxc7-src and ~/.local/bin (yosys, nextpnr-xilinx, xc7frames2bit) and need no preflight.
- 2026-10-24 In a shell cell call tri by its full path ~/.local/bin/tri (the bare name lacks mutate; the home-path scan refuses a spelled-out home directory in a committed file); run tri loop claim from a lane worktree, not the main one; a clean save rewrites tracked seal JSON, so restore with git checkout -- specs .trinity and cmp the md5 of the specs before and after. (#12065)
- 2026-10-24 A recursive find on the home folder or grep -r over specs/ on the Mac passes the 120 s limit and leaves a background task: name the directory, put timeout 20 in front, and stop the task with TaskStop if it ran. (#12028)
- 2026-10-24 The CI scan job refuses the expanded developer home path in ANY committed file, skill lessons included: write ~/.local/bin/tri, never the absolute form. Grep the diff for the home prefix before every push (tick 7 fix on PR 12063).
- 2026-10-24 Tick 10: an idle peer session whose PR blocks yours answers send_message: it synced its branch with master within minutes of a message that named the reds, the master PRs that cleared them, and the offer to rebase on master instead. Message the owner session of the PR; never edit its branch.
- 2026-10-24 Tool shapes: send_message takes session_id, bind_pr takes url; gh issue create --label fails on an unknown label (omit it). gh api -X PUT repos/O/R/pulls/N/update-branch -f expected_head_sha=<full sha> refreshes a stale PR's inherited reds; gh pr merge N --squash for a CLEAN PR (set_auto_merge refuses it). Disk at 97-99%: remove your merged lane worktrees.

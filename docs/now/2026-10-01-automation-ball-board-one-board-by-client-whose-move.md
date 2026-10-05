# NOW -- the ball board: everything the owner is in the middle of, by client, with whose move it is (2026-10-01)

## specs/automation/ball-board.t27 -- three sources, one board, no rule restated

- Owner's ask: tie the CRM's tasks to the mail contacts by client, show where
  the ball is, and put everything in work and every bug on one board, so the
  status of the whole project is visible in one place.
- Three sources, none re-derived. The CRM is `crm_waiting`, CALLED with the
  caller's context (its gate and its `waitingOn` are the only ones). The mail is
  jcrm's `crm_matter_digest`, the table automation/mail-push already receives;
  `owes_reply` us/them becomes ours/theirs, anything else is `none`. The code
  work is a snapshot of open PRs and bugs, pushed from the owner's machine
  through `ball_work_push` -- the mail digest's pattern, so no GitHub token has
  to live in production.
- The rule for code work is written once, here: an open PR is ours unless a
  review is asked of somebody else; a bug is ours until it is closed; an issue
  is theirs only when a label says it is waiting. Raw facts travel in the push,
  and the ball is computed by the host at read time.
- A card finds its client through links the owner sets (email, domain, repo,
  label). Nothing is dropped for lack of one: an unlinked mail matter is its own
  row, unlinked code work is the `project` row.
- Laws carried over: a pushed snapshot older than 26 h is reported stale with
  its age, not shown (mail-push's digest law); mail and private repos are the
  owner's and are refused before the read (the hive board's law).
- Claim: `RUN_LIVE = false` -- implemented and bound by tests in the host
  worktree (999-multibots-telegraf), not yet answering on app.t27.ai.

## specs/automation/mail-push.t27 -- v2.1 catches up with its host

- The host had vendored a v2.1 that never came back here: the digest travels
  through the render's own door (`mail_push_digest`), not the PostgREST
  gateway, which serves a different database (measured 2026-09-29). The spec
  of record now says what the host already does; `check-t27-specs --t27`
  reported the copies as different until this commit.

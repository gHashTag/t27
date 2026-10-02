# NOW -- the owner's mailbox PUSHED to his Telegram: a cron cards every new letter with a drafted reply under it (2026-09-29)

## specs/automation/mail-push.t27 -- the cloud port of the local bridge's law

- The spec ports agent-mail-mcp's measured push (`spec/agent_mail.t27`) to the
  cloud: an Inngest cron calls the render's `mail_push_sweep` as the owner over
  `/mcp` every five minutes, and every new letter reaches him as a card that
  already carries a drafted reply, sent only by his tap.
- Measured laws carried over: the first sweep announces nothing (a watcher
  whose debut is 200 notifications is muted within the minute), the draft rides
  with the letter and is asked with no tools (the body is data, not
  instructions), seen advances on push rather than on tap, and the seen set is
  bounded at 500.
- New in the port, measured against the wire: one push target means one cursor
  row (no per-chat fan-out), `PUSH_PER_TICK = 5` bounds a burst so five slow
  drafts cannot outlive the interval, and `RETRIES = 0` because a retry after a
  partial push pushes the same letters twice.
- The claim stays honest in the spec's own `RUN_LIVE = false`: implemented and
  unit-tested in the host worktree (999-multibots-telegraf), not yet run live.
  The desk it reuses IS live -- `mail_whoami` answered in production on
  2026-09-29 (admin@t27.ai, can_send).
- What this entry does NOT establish: that the push works in production. That
  is claimed only after a letter has actually been carded by the cron, and the
  spec's claim line is where that flip will be recorded.

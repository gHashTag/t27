# NOW -- automation/meeting-reminders: meetings made by talking to the agent, reminded in Telegram (2026-10-01)

## v1 -- a meeting is the caller's own, reminded before, asked about after

- Owner's ask: make online and in-person meetings by talking to the agent, with
  a proactive Telegram reminder for the person, using the newest Bot API.
- `specs/automation/meeting-reminders.t27` (`KIND "automation"`,
  `REPO "999-multibots-telegraf"`). Host: render `src/agent/meetings.ts`
  (tools `meeting_create`, `meeting_list`, `meeting_update`, `meeting_cancel`,
  `meeting_answer`), a 60 s tick that only asks the database what is due, and a
  calendar file at `/api/meeting/ics` behind a token of its own.
- Telegram features: the `tg-time` entity (Bot API 9.5) so the time renders in
  every reader's own zone; button `style` success/primary/danger; `copy_text`
  for an online link.
- Laws, each a test: offsets 5 min..1 week, at most 3; a reminder for a moment
  already past is never scheduled; a late tick sends late but never after the
  start; "later" (+10 min) never lands after the start; the follow-up comes
  after the end and never for a cancelled meeting; only the owner's tap with the
  matching token counts; the longest `mtg:` button fits 64 bytes.
- `t27c test-report`: 9/9 pass. No string `==` in fns or tests (#5162).
- Claim: `RUN_LIVE = false` until a real reminder reaches a real chat.
- v2 (2026-10-03, #5678): "later" is idempotent. `SNOOZES_PENDING_MAX = 1`,
  `snooze_adds(pending)`: a press while a snooze still waits adds nothing and
  answers the same, so a relay that lost the answer may put the buttons back.
  `t27c test-report`: 10/10; negative control (`<` -> `<=`) turns the new test red.

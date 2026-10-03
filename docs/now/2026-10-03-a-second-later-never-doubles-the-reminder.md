# NOW -- A second "later" never doubles the reminder (2026-10-03)

## What this PR adds (Closes #5678)

- `specs/automation/meeting-reminders.t27` moves to VERSION 2: `SNOOZES_PENDING_MAX = 1` and `snooze_adds(pending)` state that a "later" press adds a 10-minute reminder only when none is already pending for that meeting.
- The seal `.trinity/seals/automation_automation::meeting_reminders.json` for the new version.

## Why

- Every "later" press inserted another reminder. A relay that timed out after the render had already written the snooze could not safely hand the person the buttons back: a second press rang twice. With this law every meeting-card action is idempotent, so the bot may retry a press once.

## Not in this PR

- The host implementation (gHashTag/999-multibots-telegraf#3512), which imports these numbers through `t27c gen-ts`.

## Proof

- `t27c test-report`: 10/10 pass. Negative control: `<` -> `<=` in `snooze_adds` turns the new test red.

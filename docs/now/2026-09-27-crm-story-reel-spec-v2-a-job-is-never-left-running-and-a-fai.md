# NOW -- CRM story reel spec v2: a job is never left running, and a failure tells the truth about money (2026-09-27)

## CRM story reel spec v2: a job is never left running, and a failure tells the truth about money (Closes #4851)

- specs/automation/crm-story-reel.t27 VERSION 2: EXHAUSTED_JOB_CLOSED and ABANDONED_JOB_CLOSED -- a job dead again after its last attempt (sweep_gives_up: attempts >= JOB_ATTEMPTS and silent > DEAD_MS), or silent past ABANDON_MINUTES, is failed and closed like any failure: the plan card back, whoever asked told. Before, the sweep reclaimed a job that killed its process every time, because each claim refreshed its clock.
- FAILURE_NAMES_PAID_PARTS: nothing_charged_may_be_said only when no still and no clip was made; every still and clip is charged the moment it is made, so a failure after them names them as paid for and kept. Host: 999-multibots-telegraf story-template.ts (closeFailed, failedMoneyLine), bound by src/spec/crm-story-reel.test.ts.
- Checked: t27c test-report 13/13 pass (Zig 0.16.0), validate-vacuity 0 of 13; negative control: one new assert inverted -> FAIL. Seal refreshed.

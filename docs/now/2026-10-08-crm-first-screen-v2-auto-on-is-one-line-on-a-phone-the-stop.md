# NOW -- crm-first-screen v2: AUTO on is one line on a phone, the stop on the line (2026-10-08)

## crm-first-screen v2: AUTO on is one line on a phone, the stop on the line (Closes #7684)

- auto_line(phone, reachable): the AUTO switch is one line on a phone once read, off or on; line_action(on): on carries ACTION_STOP, off carries nothing, the line never starts AUTO (owner, 2026-10-08: variant 2).
- 7 tests, 0 vacuous, 15 of 15 negative controls killed; dupe_scan: no new duplicate body.

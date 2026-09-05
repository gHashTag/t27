# NOW -- red why: say which end of the list is the latest run (2026-09-05)

## red why: say which end of the list is the latest run (Refs #3270)

- The rows are oldest first so a cause shift reads forwards, but every other gh listing is newest first. I misread my own output twice the day after writing it, took the top row for the latest run, and nearly re-blessed a census that was already green.
- The header now says OLDEST first; the last row is the most recent run. Caught because census pin exited 0 - the tool disagreeing with my reading is what exposed the reading.

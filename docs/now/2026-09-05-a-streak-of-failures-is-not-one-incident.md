# NOW -- A streak of failures is not one incident (2026-09-05)

## A streak of failures is not one incident (Refs #3270)

- cli-tri.yml was red on master for nine runs. I read the failing step once, fixed it in #3266, and it stayed red: the cause had moved to a census ratchet that had been telling the truth since #3256, three passes earlier, when my own L8 fix took a step out of the quiet-shape class without re-blessing.
- tri red why <workflow> prints the failing STEP of each recent run, oldest first, and marks where it changes. On cli-tri.yml it shows eight runs at 'No two skill sections share a number' then the shift to 'No census moved without saying so'.
- It names its window - read 10 of 217 available - because a page is not a history, and this command exists because I mistook one reading for the state. A green run resets the comparison; a run that fails with no failing step says so rather than printing an empty cell.
- Four mutants killed. Two needed the CONTROL fixed first: failure(x), success, failure(x) cannot test the reset because identical steps either side give the same answer with or without it, and the shift array only feeds a printed marker so nothing read it until a call-site test was added.

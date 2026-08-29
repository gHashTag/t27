# NOW -- Four ways a detector misreads its own subject (2026-08-29)

## Four ways a detector misreads its own subject (Refs #2804)

- a path the program WRITES is supposed to be absent; three of five flagged sites were save_* functions
- the silence heuristic used a gate's vocabulary (bail, FAIL, exit) on CLI commands that report with println
- kept the 0-for-5 mark with the rate printed in the output, rather than deleting it or widening it until it hit something
- a blank conclusion from gh run list is in_progress, not failure -- filter on status before reading conclusion

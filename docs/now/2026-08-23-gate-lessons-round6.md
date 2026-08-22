# NOW -- three lessons from the ratchet round (2026-08-23)

Refs #2325. Recorded in ci-gates section 13:

- **Your own gate is an instrument.** The elaboration ratchet counted
  iverilog's `N error(s) during elaboration.` summary line as an error -- 25
  phantoms, one per failing module. Classify a gate's output by message shape
  once, and check that every row is a thing you meant to count.
- **A count is not a quality score.** Removing 4 syntax errors RAISED the
  number to 5, because a syntax error truncates the file and hides what
  follows it. The gate demanding an explanation was correct; the explanation
  belongs next to the number in the baseline, because the next reader opens
  the file, not the pull request.
- **The same shape returns where nobody swept the siblings.** "Escaped at the
  declaration, bare at the use" had been fixed here once already, for a
  different construct, with a note saying the other paths were fine. They were
  not. The root is one variable serving as both a lookup key and emitted text.

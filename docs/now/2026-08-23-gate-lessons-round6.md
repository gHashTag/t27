# NOW -- three lessons from the ratchet round (2026-08-23)

Refs #2325. Recorded in ci-gates section 13:

1. **Your own gate is an instrument.** The elaboration ratchet counted
   iverilog's summary line as an error -- 25 phantoms. Classify a gate's
   output by message shape once, and check every row is a thing you meant
   to count.
2. **A count is not a quality score.** Removing 4 syntax errors raised the
   number to 5, because a syntax error truncates the file and hides what
   follows. The gate demanding an explanation was correct; the explanation
   belongs next to the number in the baseline, not only in the PR.
3. **The same shape returns where nobody swept the siblings.** "Escaped at
   the declaration, bare at the use" had been fixed once already, for a
   different construct. Grep every place the value is printed -- the root is
   one variable serving as both lookup key and emitted text.

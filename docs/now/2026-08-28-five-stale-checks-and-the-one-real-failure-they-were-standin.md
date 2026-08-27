# NOW -- Five stale checks, and the one real failure they were standing in front of (2026-08-28)

## Five stale checks, and the one real failure they were standing in front of (Refs #2743)

- all five were red with the behaviour right and the check stale: a substring ruler, a pinned lowering shape, two tests asking the wrong backend, a pinned assert spelling
- two lexer conformance rows described a mechanism that is not there: # opens a comment, it is not a dropped unknown character
- main binary 1635 passed 0 failed; across all 12 test binaries 1786 passed 1 failed
- the one remaining failure is pre-existing and now filed as #2743 with its root cause and the measurement of the partial fix I reverted

# NOW -- A check outlives its subject, and four ways to notice (2026-08-29)

## A check outlives its subject, and four ways to notice (Refs #2762)

- the emitted-artifact comparison had no artifact since gen/ was gitignored, and reported it into a variable the master gate does not print
- generate-then-compare survives only because two independent parsers read the same text; name the two accounts before writing a comparison
- a CLI gate printed FINDINGS 3 and returned 0, and its allowlist did not exist because it had no verdict to allow anything out of
- --help is a ruler: it said 83 records where the live number is 109

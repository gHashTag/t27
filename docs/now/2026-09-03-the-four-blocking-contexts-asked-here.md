# NOW -- The four blocking contexts, asked here (2026-09-03)

## The four blocking contexts, asked here (Refs #2994)

- tri gates preview runs each REQUIRED context's own implementation locally: check via the shape gate, check-now-freshness via its shell script, validate via check_json_parses.py, check-linked-issue via the pattern read out of issue-gate.yml.
- Measured: validate had ZERO local reader -- a broken tracked JSON turns it red while verify.sh, scripts/pre-commit and tri hooks pre-commit say nothing about JSON at all.
- Measured: tri hooks l1-check accepted 4 references in the last 20 master commits where both CI gates accept 33 -- it missed Refs, this repository's normal spelling, and invented Reference, which neither gate accepts.

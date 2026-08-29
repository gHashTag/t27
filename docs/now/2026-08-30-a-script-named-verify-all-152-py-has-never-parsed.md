# NOW -- A script named verify_all_152.py has never parsed (2026-08-30)

## A script named verify_all_152.py has never parsed (Refs #2873)

- scripts/verify_all_152.py carries eight unresolved conflict markers, two nested, present since the commit that introduced it -- checked: no earlier clean revision exists. ast.parse is a SyntaxError. Nothing imports or runs it.
- New gate tools/check_conflict_markers.py: 7592 tracked files read, 1 carrying markers, 60 not read and said so. Abstains on a bare seven-equals divider, which is an ordinary Markdown rule here.
- Recorded in tools/conflict_markers_baseline.txt with the reason rather than repaired: resolving it means choosing which of 152 numeric formulas is right, which is not a gate's judgement.
- Workflow has no paths filter on purpose -- a marker can land in any file, and the conflicting PR most needs the reading.

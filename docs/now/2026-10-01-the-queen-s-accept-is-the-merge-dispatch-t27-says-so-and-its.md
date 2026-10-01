# NOW -- The Queen's accept is the merge: dispatch.t27 says so, and its harness measures it (2026-10-01)

## The Queen's accept is the merge: dispatch.t27 says so, and its harness measures it (Closes #5451)

- specs/queen/dispatch.t27 gains PUBLISH_RULE and f65 [publish]: of 5 verdicts x 6 head relations x 2 parent-commit kinds, the publisher arms exactly the accepts at the judged head (or under its own docs/now-only commit).
- tools/trinity_queen_dispatch.py gains a sixth evidence kind, publish: it loads tools/queen/publish.py's own accepted_at and drives all 60 shapes on every check; three planted publishers (any accept, any verdict at the judged head, docs/now never asked) each fail --self-check.
- Why: 515 of 554 merged bee PRs carry no accept from her; #5183 (verdict escalate) replaced tools/check_seal_coverage.py with one comment line. Replay re-run 2488/2488; seal re-saved for the changed spec only.

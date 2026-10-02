# NOW -- The Queen's accept at the branch head is the merge, and nothing else is (2026-10-01)

## The Queen's accept at the branch head is the merge, and nothing else is (Closes #5421)

- tools/queen/publish.py published every queen-* branch and armed auto-merge without her verdict: 249 merged and 17 armed, none gated. It now reads /queen/public-board and publishes or arms only on verdict accept at judgedHead (or its docs/now child), binds the merge with --match-head-commit, and disarms open bee PRs she has not accepted at their head.
- Fail closed: an unreadable board, or one with no verdict field yet (server before gHashTag/BrowserOS#517 deploys), exits 2 and touches nothing. Self-test 15 shapes, 9 for the gate.
- Not done here: escalations still park in review (owner: no human step); an accept older than the board's 7-day window is disarmed.

# NOW -- Eight gates were green because nobody asked them (2026-08-29)

## Eight gates were green because nobody asked them (Refs #2762)

- dispatched all 27 unmeasured workflows at master; twelve refused because their file is not in the tree -- 13 of 59 active workflows are deleted-but-registered ghosts
- eight that ran, failed: a Rust pin two editions old, a workflow that pushes to a branch its own ruleset forbids, a missing secret, a docker pull, and three bare exit codes
- tri gates unmeasured now separates ghosts from unmeasured: 28 of 58 became 13 ghosts and 1 of 46
- two red dispatches were correct: the freshness gate has no pull request to read, and the release pipeline refused an empty tag and published nothing

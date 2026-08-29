# NOW -- The only drift with no detector (2026-08-30)

## The only drift with no detector (Refs #2919)

- a required status check is named in repository SETTINGS; no file in the tree can read it, so a comment claiming a gate blocks cannot go stale against anything
- `tri gates required` reads both sides: **15** workflows claimed required by the tree, **11** of them emit no required context and cannot block a merge
- `seal-coverage.yml` records learning "the hard way in #2191" that renaming its job made a PR go BLOCKED -- true evidence its context WAS required, none that it still is; `coverage` failed on 32 of the last 40 merged PRs and all 40 merged
- the parser's own trap: `MERGE_CRITICAL` appears in the checker's docstring 30 lines before the assignment, and the gap holds an ODD number of quotes, so pairing from the wrong start put every filename on an even index -- 15 claims read as 0
- anchor on the assignment, not the name; the mutation that anchors on the name drops the report from 15 claims to 5
- `python str.replace` with a missing anchor is a silent no-op: two tests were written into a file that never received them, and `cargo test <name>` answered "ok. 0 passed"

# NOW -- t27b checked: test blocks for 7 specs in specs/tools/mcp (1 of 2) (published 2026-10-06)

## A bee's work on #6500, published from `queen-6500` (Closes #6500)

- The branch changes 7 file(s): `specs/tools/mcp/gitbutler.t27`, `specs/tools/mcp/inngest-dev.t27`, `specs/tools/mcp/needle.t27`, `specs/tools/mcp/neon.t27`, `specs/tools/mcp/notebooklm.t27`, `specs/tools/mcp/railway-mcp-server.t27` and 1 more.
- `git diff --stat origin/master...queen-6500` reads: 8 files changed, 87 insertions(+), 14 deletions(-) -- revived 2026-10-10: each test now asserts its own spec's card (transport, repo, qualified id, tool list facts), not just the four constants every mcp spec shares
- This entry is written by the publisher, not by the bee. A pull request must
  add exactly one `docs/now/` entry and a bee has no way to know that: its brief
  names a boundary file and acceptance criteria, and `docs/now/` is neither.
- What this entry does NOT establish: that the work is correct. The gates on the
  pull request judge that, and they are the same gates every other change meets.

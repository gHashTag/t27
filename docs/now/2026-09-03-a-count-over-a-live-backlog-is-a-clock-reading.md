# NOW -- A count over a live backlog is a clock reading nobody wrote down (2026-09-03)

## `tri issues numbers --as-of DATE` (Refs #2994)

- `486` open issues is not a fact about this repository; it is a fact about the moment it was asked. Read as of a date one month back the same query answers **140** -- a 3.5x move in 33 days, with nothing in the old output saying which month it belonged to
- the flag fixes the population rather than the phrasing: it drops `--state open` (an issue open THEN may be closed NOW, and that filter removes exactly the rows that make the two readings differ), reads `--state all` with `createdAt`/`closedAt`, and keeps `created <= t && (closed is empty || closed > t)`
- **the two boundaries point opposite ways**: created AT the instant counts as existing, closed AT the instant counts as closed, so an issue opened and closed in the same second is not open. A row with no creation time is not counted rather than defaulted to open
- the END of the UTC day, not the start, because GitHub's own search reads a bare date in `created:<=2026-08-01` as covering that whole day. Two tools answering the same question must mean the same thing by the same date
- **three independent routes agree on 140**: GitHub search as two queries (43 + 97), a full walk of all 1482 issues computing open-at-T from timestamps, and this command. The first two were run by separate readers before the command existed
- a malformed date is **refused, not defaulted** -- `--as-of 2026-8-1` errors, because a date silently becoming "today" under a heading that says the reading is anchored is worse than no anchor. The shape check is `skillnum::is_iso_date`, the rule already mutation-proved for section 459, not a second copy

## `--limit 500` against 486 open: fourteen from printing a page as a census (Refs #2994)

- `gh` returns at most `--limit` rows and says nothing about what it left behind, so **a full page is a lower bound and only a short page is a total**. Nothing in this CLI checked that
- measured `2026-09-03T16:35Z`: **486 open against a default limit of 500**. Fourteen issues from every printed figure becoming a page, in silence, with no line of output different
- `read_is_complete(returned, limit) = returned < limit`, and the boundary is its whole content: at exactly `limit` there may or may not be more, so it reports incomplete. Mutating `<` to `<=` kills a test whose fixture is the live boundary, `(486, 500)` and `(500, 500)`
- **the class was four call sites, not one.** Grepping `"--limit"` across `cli/tri/src/` found `numbers`, `dated`, `stale` and `gates prs` -- the last with a **hardcoded 50** and no flag (10 open PRs, so it does not bite yet). Each was run at its own boundary: `--limit 486` prints LOWER BOUND, `--limit 487` prints COMPLETE
- this is section 457 one level down: there the population was a query and the figure went stale; here the tool does not know whether it saw all of the population. **An anchor on an incomplete read is worse than none**

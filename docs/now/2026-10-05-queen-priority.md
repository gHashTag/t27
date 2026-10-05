# NOW -- the Queen's priority, decided in t27 (2026-10-05)

## specs/queen/priority.t27 and tri priority (Closes #6366)

- The Queen takes the first eligible open issue in GitHub's listing order and reads no label; `specs/queen/priority.t27` is the order it should use: labels map to four levels (vocabulary as str consts), at most 3 criticals per listing, an open blocker skips an issue, labelled work ages one level after 14 days and never to critical, ties keep the listing order, and an unlabelled repository sees no change.
- `tri priority` prints today's listing order beside the priority order with one reason per issue, deciding only through `gen/c/queen/priority.c` (gen-c, proved SAME on the t27c lab by `tri t27b gen-check`).
- On gHashTag/t27 today: 858 open issues, 3 carry a priority label, 0 have an open blocker.
- Not here: the supervisor change in gHashTag/BrowserOS; `specs/queen/dispatch.t27` PRIORITY_RULE still describes the running Queen and changes when that lands.

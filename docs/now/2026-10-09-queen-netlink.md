# NOW -- netlink.t27: the node link made safe before its first caller (2026-10-09)

## specs/queen/netlink.t27 (Closes #8276; Refs #7851, gHashTag/trios#1712)

- The six PgLink defects were each reproduced by a failing test on trios `actors-next` before any rule was written. Rows were deleted before they were handled. Pids came back after a restart. The beat shared the actors' loop. A node seen down kept writing and came back. Any node-down ended every remote start. Mail to a dead node was kept for ever.
- A new card, not an edit of actors.t27. Every pid's generation now carries the incarnation (12 of its 32 bits), a new number at every start. Every write is fenced by it (`write_admitted`). A node fences itself 15 s after its last confirmed renewal, one heartbeat before any peer can see it down. The beat renews only while the loop turns, up to 60 s. Mail is acknowledged after it is handled, and a row read again is not handled twice. A down incarnation never comes back. A node-down ends only what waited on that incarnation. Section 9's numbers (5 s, 20 s, 12 and 20 bits) are kept.
- `t27c test-report`: 9 tests, 0 FAIL, 0 vacuous, 5 invariants. 22 negative controls (each rule broken once) all FAIL.
- Not established here: the runtime is trios#1714 on `actors-next`. The SQL fence mirrors `write_admitted` because t27c has no SQL backend, and a live test holds the two equal at 30 boundary points.

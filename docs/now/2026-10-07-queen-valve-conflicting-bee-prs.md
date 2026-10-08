# NOW -- review_valve.t27: a conflicting bee PR is updated or redone, never stranded (2026-10-07)

## specs/queen/review_valve.t27 (Closes #7571)

- The gap. At 15:51Z, 94 of 213 open PRs conflicted with master, 67 of them on `queen-` branches, the oldest opened 2026-09-21. GitHub runs no required check on a conflicting head, so neither the reviewer bee nor the merger ever saw them, and no bee updates its own branch.
- `conflict_step` gives every conflicting Queen PR a next step:
  - when the Actions queue has room (at most 50 queued runs), GitHub's update-branch, with no force-push;
  - when the queue is over that, a wait of at most 48 hours;
  - after a 422 from update-branch, a third conflict, or a spent wait, a redo from fresh master (one of MAX_RELEASES);
  - after that, a close.
- Tests: 3 new, 14 in all, 0 vacuous. 21 hand mutants: 20 were killed at once. The survivor was an unpinned step code, now pinned by `the_conflict_codes_never_move`, and the negative control (C_UPDATE = 1) fails as it should.
- `gen/c/queen/review_valve.c` is regenerated; it builds with `-DT27_TEST_MAIN` and exits 0.
- Not here: the publisher and supervisor wiring, which is a separate issue.

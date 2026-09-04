# NOW -- The fix did not travel between two tables of one function (2026-09-04)

## `gates unmeasured` told a reader to take a reading that cannot be taken (Refs #2994)

- the command prints two tables. The first carries a `pr-only` column and says *"`pr-only: YES` means it CANNOT -- dispatching one starts it and measures nothing"*. **The second never got that column**
- its prose closes *"`dispatch: NO` means the reading cannot be taken on purpose"*, which reads as *`dispatch: yes` means it can*
- the single row there is **Issue Gate**: `dispatch: yes`, last default-branch run **2026-04-08**, emitting `check-linked-issue` -- one of the four REQUIRED contexts -- and it reads `github.event.pull_request.title`, `.body`, `.number`. A dispatch measures nothing
- **both tables are built in one function, forty lines apart**, and `reads_pr_context` was already there, called by one of them. Not a missing rule: a rule that did not travel to its sibling
- verified by behaviour, because the wiring is unreachable from a unit test: with the predicate replaced by `false` the row prints `-`, with it back it prints `YES`. Two unit tests hold the predicate, including the control that a push-only workflow is NOT pr-only
- **and a gate caught the insertion itself, third instance of that shape:** anchored on `fn pull_request_only_cannot_produce_a_baseline() {`, the block landed between that test's `#[test]` and its body. `tri gates tests --gate` failed the build naming both halves -- `RUNS TWICE` and `DOES NOT RUN` -- and **the two cancel in every total**, which is why earlier occurrences went unnoticed
- rule, now with three instances: **anchor on the attribute or a closing brace, never on `fn`** -- a `fn` line is not the top of the item, the attributes above it are
- **the mutation harness refused two anchors, correctly:** `reads_pr_context(&root, path),` now occurs twice, so a non-unique replacement was rejected rather than applied to the other caller

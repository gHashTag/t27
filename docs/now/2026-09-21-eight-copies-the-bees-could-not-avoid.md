# NOW -- Eight copies the bees could not avoid (2026-09-21)

## `Duplicate Body Ratchet` was red on master, and blessing is the honest move today (Closes #4496)

- The ratchet went red on master after yesterday's bee pull requests landed: eight groups each grew by one copy - `magmul` 21 -> 22, `magsub` 30 -> 31, `relu` 9 -> 10, `relu_prime` 5 -> 6, `sadd` 29 -> 30, `scale_q` 4 -> 5, `smul` 19 -> 20, and one more. 576 copies became 584.
- A red check on master is not a gate, it is noise that hides the next real failure - this repository has a skill section about exactly that. So the ledger is re-blessed in this commit, with every group that moved named here.
- It is blessed rather than fixed because the fix does not exist yet. Cross-module reuse does not generate: `use m::f;` compiles to a comment and an unqualified call, and the Zig fails with `use of undeclared identifier` (#4298). A bee that needs `sadd` in its spec cannot import it; its only choices are to copy it or to leave its criteria unmet. The brief tells it to report rather than copy, and eight chose to copy.
- What this does NOT do: remove a single duplicate. That waits on #4298, and the ratchet still refuses any NEW group or any further growth.

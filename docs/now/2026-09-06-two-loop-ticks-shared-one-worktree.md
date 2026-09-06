# NOW -- Two loop ticks shared one worktree (2026-09-06)

## Two loop ticks shared one worktree (Closes #3357)

- Two sessions ran concurrently on one checkout and one `target/`. A commit that did not form while push reported success, a binary that lost a subcommand, and flip-flopping census readings were each blamed on the tool being measured -- the broken-ruler error with a second session as the ruler.
- LOOP-RULES R17: a tick owns its worktree and takes `tri loop claim` first; the claim is a git ref, so atomicity is the remote's compare-and-swap.
- A private `CARGO_TARGET_DIR` hides the binary from `.githooks/pre-commit`, which probes `$ROOT/target/{debug,release}/tri`, refuses with exit 2, and the commit silently does not form.

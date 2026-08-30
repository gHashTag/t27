# NOW -- The stash stack is shared, and a one-line test fix went red on seal-coverage (2026-08-31)

## The stash stack is shared, and a one-line test fix went red on seal-coverage (Refs #2968)

- A separate git worktree does not get a separate stash stack. A blind stash-then-pop on a tree with nothing to stash popped another session's work in progress, and git add -A committed 44 lines of their compiler.rs and FROZEN_HASH into my branch.
- The symptom was a gate going red on a change that cannot touch it: a one-line edit to a test file, which cargo build --release does not compile, turned seal-coverage red. A release build of the branch read 152 drifted seals against master's 0.
- Their work was saved to the issue verbatim before the revert, because a successful pop drops the stash entry. It has since landed on its own as #2971.
- Two collisions in one pass: an issue duplicating one whose chip was in the dashboard I had just read in full, and a one-line repair another session landed first. A file twenty minutes old belongs to whoever wrote it -- file it and cede it.

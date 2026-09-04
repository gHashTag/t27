# NOW -- The wait and the verdict must ask the same question (2026-09-04)

## `--required-only`, and the half-fix that would have been worse than none

- Five pull requests sat in a queue for hours with **all four required contexts green**; the only
  blocker was the up-to-date rule. `tri pr ready --wait` counts **every** check -- 28 to 37 -- when
  **four** gate the merge. Waiting on thirty when four decide is not caution, it is the wrong
  population to wait on.
- `--required-only` waits on the ruleset's four. The names are **read from the ruleset**, not
  hardcoded: they live in repository SETTINGS and no file in the tree holds them, which is why
  `tri gates required` exists. The query is now one function, `required_contexts`, rather than a
  second literal of the same API call.
- **An empty required set returns `None` and the command refuses.** Treating "nothing required" as
  "nothing to wait for" would make a repository with no ruleset merge instantly -- the exact shape
  of every gate-over-an-empty-population this loop has found.

## The half-fix

The first version shortened the wait and left the verdict alone. It printed:

    VERDICT: WAIT — 1 check(s) still running, the list is incomplete.

with all four required contexts green. The flag would have shortened the wait and then **refused
the merge it exists to enable** -- worse than not having it, because the wait would look fixed.

The final read now asks the same question the wait asked. With the flag:
`VERDICT: safe to merge`.

Three mutants, three dead: an empty requirement returning `Some(0)`, counting every unfinished
check, and matching a required name by prefix -- `check` must not match `check-linked-issue`.
641 crate tests pass.

## And `cargo fmt` rewrote seven files I never touched

Formatting one file walks the mod graph. Nine files changed; seven were reverted, leaving the two
this change is about. The trap is recorded and I hit it anyway -- the useful part is that
`git diff --name-only` after `fmt` is a one-line check.

Refs #3157

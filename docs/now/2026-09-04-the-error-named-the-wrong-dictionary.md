# NOW -- The error named the wrong dictionary (2026-09-04)

## A tip drawn from the vocabulary you were not asking about

`./scripts/tri pr ready 3127 --wait --merge` answered:

```
error: unrecognized subcommand 'pr'
  tip: some similar subcommands exist: 'tt-profile', 'parse'
Usage: t27c <COMMAND>
```

- Two of three merge waiters died on that line and one kept running. I read the
  difference as a build race and went looking for a collision that was not there.
  The survivor was an older process from before a context break, still polling;
  its buffered writes into a freshly truncated log looked like corruption.
- The real cause: the invocation came from the repository's **main checkout**,
  which sits on `feat/rename-tef-to-tnf` at 2026-08-09 -- 1160 commits behind
  master, 10 uncommitted paths, 19 stashes, all belonging to another session.
  That copy of `scripts/tri` predates the Rust-binary routing and knows only
  `t27c`, so it forwarded `pr` there.
- `tt-profile` and `parse` are not near-misses for `pr ready`. They are
  near-misses for `pr` **in t27c's dictionary**, and reading them as a
  suggestion is what sent the diagnosis sideways.
- `ps -o command` showed the working waiters running
  `wG/target/release/tri`. That one line would have ended it twenty minutes
  earlier, and `tri which` (#3098) exists so nobody has to reach for `ps`.

What the fix cannot do is the part worth keeping: teaching `scripts/tri` to warn
when its checkout is stale would ship to master, and the offending copy is the
one that never sees master. A guard downstream of the staleness cannot detect
it. The durable fix is procedural -- invoke binaries from the loop's own
worktrees, never from the shared main checkout, which must not be switched or
stashed. Nothing is shipped here but the rule.

Refs #3098

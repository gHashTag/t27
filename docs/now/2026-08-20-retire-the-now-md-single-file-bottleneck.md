# NOW -- Retire the NOW.md single-file bottleneck (2026-08-20)

## Retire the NOW.md single-file bottleneck (Closes #2297)

- This file IS the change: it is the first entry written in the new layout,
  `docs/now/<YYYY-MM-DD>-<slug>.md`, one file per unit of work. The gate that
  required it was rewritten in the same commit, so the mechanism is proven end
  to end rather than described.
- Entries used to be prepended to the single file `docs/NOW.md`. Every PR
  rewrote its first line, so all 18 open PRs are marked CONFLICTING. Two PRs now
  write two different paths and there is no shared line to collide on.
- `merge=union` is retired for both `NOW.md` and `docs/NOW.md`. It was measured,
  not assumed: `git merge-tree` from a worktree checked out at `origin/master`
  reports `docs/NOW.md` clean for PRs GitHub calls CONFLICTING, so the driver
  never ran where the conflicts were reported. Locally it did run, and its
  failure mode is silent DUPLICATION -- master carries 137 headings against 136
  `Last updated:` lines.
- The gate asserts strictly more than before, not less: presence (the diff must
  ADD an entry, `--diff-filter=A`), freshness (filename date inside the same
  `[yesterday .. tomorrow]` UTC window), and a NEW content assertion -- at least
  one heading and one bullet, closing the vacuous-touch hole a whitespace edit
  used to walk through.
- Freshness now reads the FILENAME, not the first `Last updated:` line in a
  6,258-line file. That removes the prepend-order coupling that made "newest
  entry" and "first line" the same fact.
- Two dead things found and fixed while mapping consumers: `tri hooks now-gate`
  matched a BOLD `**Last updated:**` label that appears 0 times against 136
  plain ones, so it could never pass; and `bootstrap/src/suite.rs` demanded
  today's LOCAL date where CI allowed a UTC window, blocking work locally that
  CI would take. Both now use one shared window.
- Not fixed, deliberately: the 137 archived entries are not migrated,
  `docs/NOW.md` is frozen with a pointer header instead; the orphaned entry
  under `Wave Loop 421 close-out / Wave Loop 422 setup (2026-07-06)` is left
  as-is because its date cannot be recovered without guessing; and every open PR
  still needs one rebase to adopt the layout.

# docs/now/ -- the coordination log, one file per entry

Every PR and every push to `master` must add exactly one entry here:

```
docs/now/<YYYY-MM-DD>-<slug>.md
```

Write it with the tool rather than by hand:

```bash
./scripts/tri now add "Retire the NOW.md bottleneck" \
  --bullet "entries move to docs/now/, one file per unit of work" \
  --closes 2297
```

That creates `docs/now/2026-08-20-retire-the-now-md-bottleneck.md`. The date
comes from your local clock; the slug is derived from the title.

`now` is a subcommand of the Rust `tri` binary (`cli/tri`), not of `t27c`, so
`tri now add` needs `cargo build --release -p tri` first. Without that binary,
write the file by hand; the shape it must have is defined in the docstring of
[`tools/check_now_entry_shape.py`](../../tools/check_now_entry_shape.py).

## Why one file per entry

Entries used to be *prepended* to a single file, `docs/NOW.md`. Every PR
therefore rewrote the same first line, so GitHub reported every concurrent PR as
`CONFLICTING` and the races were resolved by hand -- six of them in one campaign
before a watch was written to do it automatically.

`docs/NOW.md merge=union` was added to make those merges automatic. Measured
against `master`, it did not work:

- **GitHub never applied it.** `git merge-tree` with the rule in force reports
  `docs/NOW.md` *clean* for PRs that GitHub simultaneously labels
  `CONFLICTING`. Mergeability on the platform ignores merge drivers, so the rule
  bought nothing where the conflicts were actually reported.
- **Off the platform it corrupted silently.** Union's failure mode is
  *duplication*, not removal. Two branches editing one `Last updated:` line
  merge with **no conflict** into two adjacent `Last updated:` lines under a
  single heading. `docs/NOW.md` carries 137 headings against 136 date lines --
  one entry lost its date to exactly this, and five more show union's
  blank-line-eating signature.

Two PRs writing two different filenames have nothing to merge. That removes the
conflict structurally instead of papering over it, which is why `merge=union`
has been retired from `.gitattributes`.

This is also the repo's dominant convention already: `docs/reports/` holds 1,564
date-and-wave-stamped files, `.claude/plans/` holds 417, and
`.trinity/experience/` is one append-only file per track.

## What the gate checks

`.github/workflows/now-sync-gate.yml` runs `scripts/ci/now-sync-gate-diff.sh`,
which asserts all of:

1. **Presence** -- the diff **adds** (`--diff-filter=A`) at least one file
   matching `docs/now/<YYYY-MM-DD>-<slug>.md`. Editing an existing entry is not
   writing one.
2. **Freshness** -- that entry's **filename** date is inside
   `[yesterday .. tomorrow]` UTC. Tomorrow is included so a contributor east of
   UTC naming an entry with their local date is not rejected while UTC still
   lags a day. The date is read from the filename, so there is no
   `Last updated:` line to parse, duplicate, or disagree with.
3. **Content** -- the entry has at least one Markdown heading and at least one
   bullet. Under the old layout a whitespace touch satisfied the gate; an empty
   new file would be the same vacuous pass, so it is rejected.

Trusted bots (`dependabot[bot]`, `github-actions[bot]`) still pass as a no-op.

A second required gate reads the entry's **shape**:
`.github/workflows/check-now-freshness.yml` (job `check` -- the file and job
names are crossed with the one above) runs `tools/check_now_entry_shape.py` on
every `docs/now/` entry the PR adds. What it checks is defined in that script's
docstring, not here. Its filename pattern is stricter than the sync gate's: the
slug must be lowercase `[a-z0-9-]`, while `now-sync-gate-diff.sh` accepts
`[A-Za-z0-9._-]`, so a name the sync gate takes can still fail `check`.

Locally, `scripts/verify.sh` previews both questions over the diff against
`master`. With the `tri` binary built, `tri now check` runs the shape checker
on the entries a change adds and `tri hooks pre-push` runs
`now-sync-gate-diff.sh` over the push range.

`.githooks/pre-commit`, `scripts/pre-commit`, `t27c check-now` and
`tri hooks now-gate` are weaker: they only check that the `docs/now/`
**directory** holds an entry dated inside the window. Any fresh entry already
on `master` satisfies them, so they do not show that your change adds one.

## Files that are not entries

Anything without a leading `YYYY-MM-DD-` (this README, for instance) is ignored
by every check. It cannot satisfy the gate and it will not trip it.

## History

Entries written before 2026-08-20 remain in [`../NOW.md`](../NOW.md), which is
now a frozen archive. They were deliberately **not** split into files: that
migration is mechanical, touches all 137 entries, and would have made this
change unreviewable. No gate reads `docs/NOW.md` for freshness any more, but
`now-sync-gate-diff.sh` refuses a range that edits it unless a commit message
in that range carries an `Archive-Repair: <reason>` trailer.

The archive still carries the damage union did to it: 137 headings against 136
`Last updated:` lines, the missing one under the heading
`Wave Loop 421 close-out / Wave Loop 422 setup (2026-07-06)`. Freezing the file
does not repair that; it only stops it getting worse.

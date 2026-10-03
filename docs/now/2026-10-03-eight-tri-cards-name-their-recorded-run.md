# NOW -- eight tri cards name their recorded run (2026-10-03)

## What was read

- `specs/tools/tri/` has 52 cards, one per `tri` command of this repository. None carried a `CAST`, and gHashTag/trinity's `check:tools-coverage` counted t27 at 0 of 52 recorded.
- `specs/tools/README.md` says these cards are edited here and re-vendored byte-identical into gHashTag/trinity, and its schema table for `tri/<command>.t27` had no `CAST` row. The rules for a `CAST` live in one place: `castProblems()` in trinity's `apps/website/scripts/agents-from-specs.mjs`.

## What changed

- Eight cards end with `pub const CAST : str = "term/t27-tri-<command>/session.cast";`, under a comment that names the page, the number of recorded runs and the commit the binary was built from (1b12580ce699): `misread`, `discard`, `census`, `seals`, `prose`, `lean`, `competitors`, `vectors`.
- Each recording ran `target/release/tri` from master at 1b12580ce699. Every exit code is 0, and the home path is scrubbed. `seals` and `lean` record two runs each: `fresh` + `hollow`, `vacuous` + `reach`.
- The schema table gains a `CAST` row. It points at `castProblems()` instead of restating its rules, and it says that a `CAST` leaves `WITNESS` at `source-parse`.
- `t27c parse` reads `CAST` on all eight as a `pub` `ConstDecl` with its string literal, with no parse errors. `spec-status` is `NOFN` before and after, as for every card.

## Not verified

- The recordings are not live yet. They are published by gHashTag/trinity's companion change, which carries the same eight hunks in its vendored copy and has to merge after this one.
- Not recorded, so not given a `CAST`: `tri ledgers audit` exits 1, `tri status` and `tri health` print ring-47 data from 2026-04-06, and `tri worktrees` ran past 120 s.

Closes #5767

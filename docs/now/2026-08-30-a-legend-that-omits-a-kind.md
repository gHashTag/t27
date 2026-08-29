# NOW -- A legend that omits a kind (2026-08-30)

## A legend that omits a kind (Refs #2851)

- Six emitter pull requests carried a RED seal-coverage check **on the pull
  request itself** -- #2841, #2844, #2845, #2849, #2856, #2859 -- and all six
  merged. The signal reached the author six times out of six.
- **The output is why.** The legend explained `stale`, `dangling` and `phantom`.
  The kind that fired was `gen-drift`, which had NO entry, and the only repair
  the page named was `--update-baseline` -- which for that kind records the drift
  as accepted debt instead of recording what the compiler now produces. The one
  actionable line was the wrong action.
- The legend is now data, and `--self-check` reads THIS FILE'S SOURCE for every
  kind it can attach. Its first run: `legend covers 5 of 8 kind(s) MISSING:
  no-spec-hash, no-spec-path, unreadable` -- **three kinds beyond the two I was
  fixing.** The script attaches eight; the legend explained three.
- It is also SELECTIVE now: only the kinds present in this run print. An author
  with 134 `gen-drift` rows used to read three paragraphs about stale, dangling
  and phantom seals, none of them theirs.
- `gen-drift` names `tri seals drift --fix` and says to read `t27c corpus` first,
  because re-sealing is a statement that the new output is the one you want.
- Controls both directions: removing a kind's entry gives `MISSING: gen-drift`
  and exit 1; adding an entry nothing attaches gives `UNREACHABLE` and exit 1; a
  planted drift prints the new paragraph and none of the others.
- ci-gates 270-272.

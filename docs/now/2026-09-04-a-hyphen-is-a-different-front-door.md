# NOW -- A hyphen is a different front door (2026-09-04)

## The gate needed `tri` and a space, so a sibling script walked past it

- `check_documented_commands_exist.py` resolves **216** names across four surfaces, and both its
  matchers require `(t27c|tri)` followed by a SPACE. `tri-lean` steps over them on one character.
- Under the git convention a `tri-<name>` beside `tri` is a command; this repository documents
  **five** real ones under `scripts/`. A sixth, `scripts/tri-lean`, has **never existed as a git
  object** anywhere in the history, and two Lean *source* files told the reader
  *"Do NOT hand-edit -- regenerate via ./scripts/tri-lean"* -- a prohibition pointing at nothing.
- **Four matchers, three of them wrong**: bare `tri-<name>` finds 462 lines (`tri-valued` is an
  adjective); any path prefix finds 102 (`../tri-net` is a sibling REPOSITORY); final-segment finds
  50, of which **40 are dead** -- the other ten name the five real scripts, and a matcher this
  loose is still mostly describing its own population. Anchored to
  `scripts/`: **13 mentions, 6 names, 5 resolve, 1 does not.**
- **It reported itself on the first run** -- line 558, the self-check fixture that must name an
  absent sibling for the negative control to mean anything. Excluded by path, like `docs/now/`.
- Five new self-check assertions, two of them negative: an adjective is not a sibling, and neither
  is a sibling repository. The gate exits **0** again once the three Lean headers say what is true.

Lean is not installed here, so the three headers are comment-only edits with balanced `/- -/`
blocks and unchanged proof bodies; all three are in the root's import closure, so CI compiles them.

**Correction (2026-09-04, later the same day).** The final-segment matcher's **50** reproduces
exactly, but *"every one is dead"* does not: **40 of the 50** are dead. Ten name
`scripts/tri-sync.py`, `tri-issue-create.py`, `tri-search.py`, `tri-doc-sync.py` and
`tri-pr-create.py`, all of which exist -- the same five this entry calls real two bullets earlier.

Refs #3137

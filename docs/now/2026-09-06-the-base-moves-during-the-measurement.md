# NOW -- The base moves during the measurement (2026-09-06)

## The base moves during the measurement (Refs #3331)

- `origin/master` took **69 merges in twenty-four hours**, one every twenty-one minutes,
  while a corpus measurement takes fifteen to thirty: build a pinned binary from the base,
  run 650 specs, build the change, run 650 again.
- So the base moves DURING almost every measurement, and two different failures follow from
  that one fact.
- A one-line repair measured `338 -> 352, +14`. Rebuilt against the base as it stood at
  report time it measured **+0** -- all fourteen were a neighbour's work. Only rebuilding
  the base caught it, and nothing had asked for that.
- A `Box`-for-recursive-types repair measured `357 -> 360, +3` and was found at merge time
  to be on master already, both shapes, all three specs green. `tri loop claim` had been
  taken BEFORE the work, which is the right order -- and did not help, because the
  neighbour took none. A claim separates two sessions only when both take one.
- `tri window --start` records the tip; `tri window --check` refuses if it moved, naming
  the recorded sha, the current tip, and how many merges landed.
- Counted with `--first-parent`, deliberately. A three-merge move measured **7** without
  it: the range holds every commit of every side branch pulled in, four of them merges.
  The reader's question is how many pull requests landed, and that is three.
- Six controls. The one worth naming asserts that a window which cannot be READ is not a
  window that did not MOVE -- an unresolvable ref gives None, never zero.

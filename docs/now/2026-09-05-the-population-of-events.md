# NOW -- The population of events, after the predicate and the operand (2026-09-05)

## The population of events (Refs #3320)

- Three classes in two days, in order of subtlety: the PREDICATE (unit tests answer it),
  the OPERAND (section 590: eight gates read the working tree, the directory, or HEAD),
  and the EVENTS -- what else comes through this place, which nothing answers but
  enumeration.
- The third produced two defects every control missed, because a control tests the event
  you thought of. A push can be a deletion. A commit can be a merge.
- The method is enumeration: marker hooks that only `touch` a file, one run per event,
  then read which files exist. Minutes, and it answers what no reasoning about the gate can.
- Two hazards met while measuring: `git config` inside a worktree writes to the SHARED
  config, and a failed `cd` does not stop a subshell. Both are recorded with their fixes.

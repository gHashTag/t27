# NOW -- Six more specs take the version that parses (2026-09-20)

## The conflicted bee branches, resolved by measurement rather than by rebase (Closes #3820, #3805, #3767, #3761, #3727, #3686)

- Nineteen bee pull requests could not merge: `mergeStateStatus: DIRTY`, which means GitHub runs no required check on them, so `queen merge` read all nineteen as "waiting for CI". They were compared instead: for each, the branch's version of its spec and master's, both judged by a `t27c` built from master.
- Six of them carry a version master needs. Four files on master do not parse at all (`trie`, `quick_sort`, `csv`, `hex`): their text is Zig written inside a `.t27` file - `catch unreachable`, `for (0..n) |i|`, `@constCast` - so nothing generates from them and none of their tests ever runs. One is PARTIAL with eight unimplemented bodies (`norm`), and one is implemented on both sides but the branch carries seven more tests (`status`).
- Measured over all 948 specs, before and after: `NOPARSE` 65 -> 61, `IMPLEMENTED` 513 -> 518, `PARTIAL` 9 -> 8. Each of the six prints `IMPLEMENTED` with zero `not yet implemented`.
- What the trade costs is stated and filed, not buried: the unparseable versions declared helper functions the parsing ones do not (#4285, #4286, #4287, #4288 name them per file, with mechanical criteria), and carried more tests in three of the four files. Nothing that RAN was lost, because nothing in those files ran.
- The other thirteen pull requests were closed as superseded, each with the measurement that says so: master already holds an implemented version with at least as many tests, or the branch's own version does not parse, or it carries more unimplemented bodies than master does.

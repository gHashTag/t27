# NOW -- A repair performed six times is a command (2026-09-04)

## `tri skill renumber` moves appended sections to the numbers the base left free

- Six section-number collisions in one week on `.claude/skills/ci-gates/SKILL.md`.
  Two of them hit the same branch four hours apart: renumbered to 468/469,
  master then took 468, 469 and 470, and it moved again to 471/472.
- The repair never varied: rebuild from `origin/master`, re-append with the next
  free numbers, assert the master prefix is byte-identical, re-run
  `tri skill check`. Only the assertion needed care -- the file is 12,000 lines
  and a lost section looks exactly like a file that never had it.
- The command leans on the invariant the workflow already has: a section is
  APPENDED, so the branch's file is the merge base's file plus a tail.
- It refuses when that is false. If an existing section was edited it cannot
  tell which lines are yours, and a quietly guessed split point would be worse
  than the manual repair.
- The first number comes from the BASE and never from the tail. Reading it from
  the tail is how the second collision happened, and a mutation of exactly that
  line survived the first six unit tests -- the decision lived in the
  integration path. It moved into `plan()` and now has a test that fails when
  two tails with different existing numbers plan differently.
- References follow, and only the ones being moved. Renumbering 11 must not
  rewrite `&sect;110` into `&sect;4710`; that has its own test.
- Demonstrated on itself: this change's section was written as `## 999.` and the
  command placed it at 471.

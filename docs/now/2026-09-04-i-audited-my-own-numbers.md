# NOW -- I audited my own numbers and eleven were wrong (2026-09-04)

## The one population never audited was mine

- Every checkable number this session published in `docs/now/2026-09-04-*` was re-measured
  against the command that produced it: **260 numbers, 170 verified, 24 drifted, 46 not
  checkable, and 11 wrong.** That is **11 of 194** checkable, 5.7%. The corrections are applied
  in the entries themselves, and #3172 lists them.
- **46 numbers no command can re-run.** Timings, scratch worlds, workflow runs. They are not
  claims; nothing can ever contradict them. That is the larger finding and it has no fix here.

## Nine of the eleven are one defect wearing four hats

A figure measured over a narrower or wider population than the sentence it sits in. The `Qed`
cell counted the five files carrying an `Admitted` while `files` and `Admitted` in the same row
counted all 18. `28` was the file count of the 1792 one-liners, attached to the 1813. `~55`
averaged a set the entry had already pinned as seven.

**In three cases the entry contradicted itself two lines later.** `a-ratchet` says *"1813 in 32
files"* one bullet after saying 28. `a-hyphen` calls five scripts real two bullets after calling
all fifty dead. Nothing had to be measured to catch those three -- only read.

## The audit was wrong nine times in twenty, and the last two are the lesson

Seven claimed refutations died in the adversarial stage. Two of those were the same wrong-tree
error: `git log -1 -- <file>` walks only HEAD's ancestry, so a file that reached master by
re-add reports the re-add as its authoring date.

**Two more survived both stages.** The auditor called *"1792 of 9842 spec tests (18.2%)"* wrong
because the repository's own ratchet counts 4867. Both numbers are real: 4867 is the brace form
`test N { ... }`, and the rest are BDD-style `test N / given / then`, which
`bootstrap/src/compiler.rs` parses as tests (`KwTest => parse_test_block`). 9842 is every spec
test and 18.2% is honest. Caught by reading the parser -- not by counting a third time.

**A second count is not an adjudicator.** Two counts that disagree are two populations until
something outside both says which one the sentence meant. Twice today an ad-hoc re-measurement
disagreed with a shipped tool and the ad-hoc command was wrong both times: a hand-rolled body
walker read 4031 where the tool reads 4054, and `*.tri` globbed repo-wide read 154 where
`.tri` **specs** hold 90.

## Two instruments that lie quietly

- **`git log --since=<bare date>` inherits the current time of day.** At 19:12,
  `--since=2026-09-04` means *since 19:12 today* and silently drops everything earlier -- it
  returned **0** commits against a day holding **79**, and `--until=2026-09-04` returns all 3016.
  A window that slides through the day, from the filter's own default. The repository's own nine
  uses are all relative offsets (`30.days`), so nothing here is affected; the defect was in the
  audit's commands. Give it a time, or use `--after`.
- **A reading that does not reproduce is not a measurement.** One run of a matcher printed 1791
  and 9841 where three later runs print **1792** and **9842** on an unchanged tree -- and 28
  files x 64 = 1792 confirms it structurally. Both candidate mechanisms were tested and refuted.
  A delta of one against 1792 is exactly the size a transient produces, and it was one sentence
  away from being published as overnight drift. **Re-run before reporting a delta.**

Refs #3172

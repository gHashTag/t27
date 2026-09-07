# NOW -- The extractor read a spelling, not a vocabulary (2026-09-07)

## The extractor read a spelling, not a vocabulary (Refs #3393)

- `cli-tri` red on master since 17:59. Green at `92a19608c`, red at `52db4aacb` --
  a neighbour's #3388, which rewrote `issue-gate.yml` to strip fenced blocks and quotes
  from the pull request body before grepping. The change is correct; the test that reads
  that file broke.
- Two independent breakages in one line, either sufficient on its own.
- The gate's line used to OPEN with its `grep`, so the first single-quoted span was the
  pattern. It now opens with `printf '%s\\n%s\\n'`, and the extractor took the FIRST
  span, found no reference in it, and abandoned the line rather than looking further along
  it. Taking the first span was never the rule -- it was the first span happening to be the
  only one.
- The line filter matched on `#[0-9]+`, which #3388 tightened to `#[1-9][0-9]*` to
  reject `#0`. The line then matched nothing, and the extractor reported that the gate
  states no pattern at all.
- Anchored on the gate's VOCABULARY now, not on how it spells a number. A number's spelling
  is the part of a pattern most likely to be tightened; the keyword list is what identifies
  it. Every quoted span on the line is scanned.
- 824 tests pass. Mutation: breaking the vocabulary inside the workflow reddens the test,
  so it still reads the file it claims to read.

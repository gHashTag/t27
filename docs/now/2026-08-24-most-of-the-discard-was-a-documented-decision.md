# NOW — most of the discard was a documented decision (2026-08-24)

Yesterday's number was 26,546 tokens of specification "never reaching codegen". Two thirds of it is a design decision the compiler documents in its own source, and my census undercounted the rest by half because I read my own tool's truncated output.

- **`compiler.rs` already says so**, in the function that does it: *"`forall`-quantified statements (837) are not runtime-checkable and fall back to the original skip, as does anything else this cannot model."* And, in codegen: *"Skipping an unbounded `forall` is a defensible language decision; reporting it as verification is not."* The project diagnosed this, decided it, and fixed the dishonest half — the artefact used to print `verified (no statements)` and now prints `NOT CHECKED -- body was not lowered`.

- **The honest split.** 26,533 discarded tokens: **16,986 (64%) inside `forall` blocks**, 9,547 (36%) everything else. `forall` opens 857 of the discarded lines, which corroborates the compiler comment's count of 837 statements.

- **The actionable number is 9,547, not 26,546.** By first dropped token on a line, outside `forall`: `:` 605, `var` 113, `assert` 110, `let` 94, `const` 77, `and` 72, `then` 65.

- **My own tool truncated the evidence.** `--spans` printed the first 40 discarded lines per file, and I ran a corpus census over that output. It reported `forall` on 415 lines; uncapped, it is 857. The truncation hit hardest on exactly the files a census is about — the ones with the most discarded lines. `--limit 0` now prints all of them, and the default says so when it truncates.

- **69 sealed specs discard text.** 3,650 of 85,245 lines across them (4%), worst case 25% in `specs/queen/brain_summaries.t27`. The seal's `spec_hash` covers the whole file, so editing discarded text does make the seal stale — the record is conservative, not permissive. What no seal says is that a quarter of a file's authored lines contribute nothing to the four `gen_hash` values it pins.

- **Seven over-generalisations in three days, all mine.** Three about this defect alone: it is not "invariant inside a test", not "the expression form", not "the braceless test block". And this one — "26,546 tokens of specification silently lost" — was the largest, because it was a real number attached to a wrong meaning. A number that is correct and misleading is harder to catch than one that is wrong.

Refs #2474, #2479

# NOW -- A capitalised Const in 24 quoted type strings (2026-09-06)

## A capitalised Const in quoted types (Refs #3362)

- Twelve specs write field types as quoted strings with a capitalised keyword:
  `tag : "[]Const u8"`. The compiler strips the quotes and emits the content, so rustc
  receives `Vec<Const u8>`.
- The correct `[]const` appears **374** times in the same corpus; the capitalised form
  **24**. That is a case typo in a keyword, not a semantic choice, which is why it is
  corrected here rather than referred: no reading makes `Const` mean anything else, and
  the corpus settles the spelling 374 to 24.
- Priced by hand first, on a sample of six: four compiled once corrected. Measured after,
  over the whole corpus: **346 to 352, zero regressions** -- two more than the sample
  predicted, because `aho_corasick` and `regex` also cleared.
- The matcher is narrowed to `[]Const`. A blanket `Const ` would have corrupted
  `KwConst = 1` in `compiler/lexer.t27`, a legitimate enum variant and the 25th
  occurrence. Verified intact: 3 mentions still there.
- `html` and `xml` keep failing, on `std.StringHashMap(...)` -- a Zig standard-library
  type also written as a string. Different defect, not touched here.

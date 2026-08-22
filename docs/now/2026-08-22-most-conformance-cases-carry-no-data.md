# NOW -- 412 of 512 conformance cases carry no data at all (2026-08-22)

## tri vectors debt classifies the corpus; the first classifier was the liar (Refs #2241)

- Measured across all 34 vector files: 512 cases, of which only 100 carry any
  field beyond id/description/note. 24 files are PROSE-ONLY -- an id and a
  sentence, no inputs, no expected values. No runner can execute them as
  written: they are documentation shaped like tests, and counting them as
  "vectors" is what let 0 executed read as adequate for months.
- The split is now the command's own output: 1 executed, 9 debt (data present,
  no runner -- blocked by #2410/#2413), 24 prose-only.
- INSTRUMENT CONTRADICTION, resolved before the number was written down: the
  first Rust counter split objects on ',' and was fooled by commas inside
  description strings, reporting 147 data-carrying against python's 100. The
  string-aware rewrite reproduces the python count on all 34 files. Two
  instruments disagreeing is a stop condition, not a rounding difference.

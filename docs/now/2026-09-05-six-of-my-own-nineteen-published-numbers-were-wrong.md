# NOW -- Six of my own nineteen published numbers were wrong (2026-09-05)

## Six of my own nineteen published numbers were wrong (Refs #3253)

- An audit re-measured every figure published this pass on master: five readers on disjoint claim sets using their own commands, third reading on each disagreement. 10 CONFIRMED, 3 stale-but-true, 6 WRONG.
- competitors.rs has NO production item below its test module: beta_competitor is at 696, inside the raw string const TWO which opens at 681 and closes at 705. A column-0 regex counted a fixture as code.
- '10 of 46 files / 130 items' reproduces under no definition: 13/160 by string split, 11/140 by attribute line, 9/133 rejecting string literals. Only gates.rs 79 survives, and the matcher is part of the number.
- 'Each push triggers 28 check-runs' was the MINIMUM of the sample; the range is 28-43, median 41. The correction strengthens the argument it was used for, which is why it was tempting not to check.
- 'Zero Admitted under coq/' - there is one, in coq/README.md. Zero in .v files, which is the population the gate reads and the one I meant.

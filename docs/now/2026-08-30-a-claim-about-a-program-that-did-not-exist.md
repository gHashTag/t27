# NOW -- A claim about a program that did not exist yet (2026-08-30)

## Testing my own sentence from one iteration earlier (Refs #2876)

- §317 said a variant-diff script "would have found all four". No such script existed when I wrote it
- built and measured: the naive diff flags 10 lists of 10 -- every list omits some constructed kind, because almost every list is a legitimate subset
- grouping kinds into families discriminates (3 of 10) and points at the exact `has_body` line on the commit before #2875 -- but all three of its hits on a CLEAN tree are correct code
- two of the four defects are not NodeKind lists at all: `compound_binop` maps operator strings, `expr_is_bool` is a match whose missing arm nobody enumerated
- score: enumeration-diff finds 1 of 4 with 3 false positives; `tri kinds drift` finds 1 of 4 with ZERO
- the lesson is not the script: a sentence in the conditional tense inside a document of measurements accumulates authority from the numbers around it
- either build it and write the number, or write "untested" beside it

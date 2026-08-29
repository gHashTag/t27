# NOW -- Three spec lines, and the idiom I validated against a broken line (2026-08-29)

## Three spec lines, and the idiom I validated against a broken line (Refs #2754)

- assert X = Y uses a single = where t27 needs ==; assert |x - y| < tol uses vertical bars the parser does not have
- I first wrote |...| because a line in the corpus uses it -- that line is itself one of the failing ones. abs(...) is the real idiom, and radix_economy already uses it four times in clauses that work
- 23644 -> 23624 tokens; constants 11 -> 10, radix_economy 5 -> 4, jones 4 -> 3 fallback events; all three still generate
- five of the eight single-= lines are NOT mechanical: three carry quantifier prose (#2774), one references an undefined e, one is followed by an English when-clause

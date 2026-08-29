# NOW -- Build the rule you rejected (2026-08-30)

## Two lessons from the invariant verdict (Refs #2856)

- three replacement rules were built and run before one was kept; two of them are wrong in ways not deducible from reading them
- "reject only if it contains a function call" breaks ten specs; "also require the name to be in const_defs" throws away 1742 checks that were compiling, because that map misses enum constants and does not hold `true`
- a rule about a corpus is a claim about the corpus: testing it costs one build, not testing it costs shipping the second row
- promoting the discards from comments to code exposed two defects the comment had been silencing: an operator with no right operand, and Zig digit separators C reads as a suffix
- a branch that swallows its input and emits a comment cannot be assessed by reading it -- its cost is invisible until something downstream consumes what it hid
- expect the first attempt to enable such a branch to look like a regression, and expect that regression to be a defect you did not know about

# NOW -- Cross-module types: priced, and the price was zero (2026-09-06)

## Cross-module types priced at zero (Refs #3351)

- Twelve specs fail on a type declared in a sibling spec, six of them on `Trit`. It looks
  like the obvious next feature and it is worth nothing.
- Of the twelve, eleven carry between 9 and 84 OTHER errors. Only `bigint.t27` fails on
  missing types alone.
- Resolution is ambiguous at every level: of 55 types imported by name, 18 are declared
  once, 8 in two to four places, and 29 nowhere at all. `Trit` has four declarations, and
  a module index does not help -- two files declare `module tritype` and both declare it.
- The measurement that settled it: pasting the real definition into the one unblocked
  spec's generated Rust by hand leaves three `mismatched types`. 5 errors before, 4 after.
- So the resolver moves the corpus by ZERO, and the blocker belongs to the corpus.
- The method is the lesson: **satisfy the dependency by hand before building the machinery
  that would satisfy it.** One command priced a hundred-line feature at nothing.
- A correction to my own earlier note: I wrote that `Trit` is declared in exactly one
  spec. That came from a grep for `type Trit` only. With the full matcher it is four.

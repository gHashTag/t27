# NOW -- `tri misread`, and the first thing it did was correct my count (2026-09-05)

The census #3225 asked for now exists. Its first corpus run corrected the number in
the issue that motivated it, from 14 to 22.

## The command (Refs #3225)

- `tri misread` reads the GENERATED OUTPUT, not the parser, because being read wrongly is invisible at the parser -- `tri unparsed`'s population is defined by "the compiler refused", so anything accepted lies outside it by construction
- four shapes, each a line that cannot mean anything in its target language: `pub f: ,`, `pub f: 0,`, `Vec<>`, and the C half `0 f;`
- **the positive control is not optional and it is not decoration.** The command runs a reproducer through the real compiler FIRST and refuses to report on the corpus unless every shape it claims to detect actually fired on a case built to contain it
- it caught a real defect on its first run: the reproducer carried only the literal-in-type-position case, so two detectors had no case, and the command refused rather than printing two comfortable zeros

## What it reads on the corpus (Refs #3225)

```
control: all 4 shape(s) fired on the reproducer
corpus:  650 spec(s) under specs
generated for 581 of 650 spec(s)

  22  rust: `pub f: ,`      field with no type
   1  rust: `pub f: 0,`     literal in type position
   0  rust: `Vec<>`         generic lost its parameter
   1  c:    `0 f;`          literal in type position
```

- the `Vec<>` row reading **0** is the point of the design: #3213 removed that defect from the corpus, and the control is what makes the zero a result rather than a silence
- all 22 are in the rustc-FAIL set, so none is a false positive

## The matcher that undercounted by eight (Refs #3225)

- the ad-hoc grep behind the 14 required `^[[:space:]]+pub [a-z_]+: ,$` -- a field-name class with **no digits and no capitals**
- the eight it missed are `Double`, `Null`, `Nil`, `Ready`, `IgnoreCase`, `Ascending`, `RGB`, `Trace` -- every one capitalised, and every one an enum-variant name, which is the same underlying cause
- the correction strengthens the finding: a typecheck rule refusing an empty field type now needs **22** ledger slots against a cap still at 152 of 152
- a one-off grep and a tested detector answer the same question with different populations, and only one of them has a test that says so

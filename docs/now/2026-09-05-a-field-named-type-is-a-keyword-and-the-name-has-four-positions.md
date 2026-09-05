# NOW -- A field named `type` is a Rust keyword, and the name has four positions (2026-09-05)

Seventh compiler fix of the pass, and the second in a row where the class had more
positions than the obvious two.

## The defect (Closes #3241)

- a spec field may be named `type`, `ref` or `match`; those are Rust keywords and the emitter wrote them bare, so the struct did not parse
- Rust's raw-identifier syntax `r#type` is the exact spelling and needs no judgement, unlike the mappings of #3222
- the name is written in **four** places: the field declaration, a field access, a function parameter, and a struct literal
- repairing the first two changed nothing beyond one spec; repairing all four changed nothing further, because the remaining specs carry other defects behind the keyword
- measured by name: **314 -> 315, +1, 0 regressions**

## The class is closed, which is the honest measure here (Closes #3241)

- `found keyword \`type\`` as a FIRST error across the corpus: **7 -> 1**
- the residue under "found keyword" is a different cause -- `enum` (5), `struct` (2), `mod` (1) -- keywords in TYPE position, from the list-valued declaration of #3225, not field names
- `crate`, `self`, `Self` and `super` are deliberately not escaped: `r#` is invalid for them, so escaping would swap one parse error for another; none occurs as a field name here and if one appears it stays bare and fails visibly

## A measurement flaw that has now misled me three times (Refs #3241)

- to find the failing source line I was taking the LAST numbered line of rustc's first error block
- rustc renders its SUGGESTION as a numbered line and suggestions come last, so I was reading back the corrected code as if it were the defect
- it produced a phantom `pub struct ListNode<T>` earlier in this pass, and here it made two specs look already-repaired when they were not
- the correct extraction is the FIRST numbered line after `-->`; the suggestion is what a fix looks like, and mistaking it for the input is the same shape as reading a tool's output as its subject

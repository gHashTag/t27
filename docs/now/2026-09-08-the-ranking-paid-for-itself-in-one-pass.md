# NOW -- The ranking paid for itself in one pass (2026-09-08)

## The ranking paid for itself in one pass (Closes #3467)

- The tool added last pass ranked the error classes for the first time and named a family nobody had looked at. One pass later it is **the largest single repair this campaign has made**: `Trit.pos` reaches C as `Trit.pos`, and C has no `Type.member`. Errors **14040 → 13340**, `unexpected type name 'Trit'` **642 → 12**, **15 files better and none worse**. `vsa/ops` 569 → 99.
- Only C is wrong: `gen-rust` emits `Trit::pos` and `gen-zig` emits `Trit.pos`, each correct for its language, while the constant `gen_c_enum` had been emitting all along is `TRIT_POS`. The declaration and the use had two spellings and nothing brought them together.
- **Half the family is a different defect and is filed as such.** `use of undeclared identifier 'POS'` (412) and `'NEG'` (319) are unchanged -- 731 errors over 422 lines, in `(a == NEG) ? ...` shapes. Those bare names appear **nowhere in the specs**: the compiler's own lowering introduces them without the type prefix. Same family by symptom, different repair, and saying so is cheaper than discovering it later.
- **A mutant survived on a guard that looked decorative.** Keying the rewrite on either side -- the member's name OR the base's -- passed every test, because no test had a member whose name was also an enum type. `struct S { Colour : i32 }` beside `enum Colour` is that case: `s.Colour` must stay a field access. Added, and the mutant dies.
- The measurement came before the code and earned it: printing the sites by shape showed 604 distinct lines all of one form, which is what made a single rewrite plausible before it was written.

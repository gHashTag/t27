# NOW -- The signature was right and the call was not (2026-09-08)

## The signature was right and the call was not (Refs #3402)

- #3403 rewrote a written `[]T` PARAMETER to `&mut [T]` and never rewrote the ARGUMENT. Adversarial review measured it: of **608** call-site arguments landing in a slice parameter position across the generated corpus, **0** gained a borrow. `tritwise_and(a, b, temp, len)` therefore read "expected `&mut [i32]`, found `[i32; 27]`" -- a correct signature reached by an uncorrected call.
- Now a call site borrows the argument that lands in a marked slot, and does NOT borrow one that is already a `&mut [T]` parameter of the caller: passing a borrow onward is a reborrow, and a second `&mut` is an error. Both shapes are in one test, because a rule with two branches needs a case for each.
- Measured, all 650 specs against master: total diagnostics **3773 -> 3745**; move/borrow **33 files / 143 -> 24 / 117**; rustc acceptance unchanged at **430**. `specs/isa/ternary_bitwise.t27` alone goes **12 -> 3** errors across the two changes. 41 files' output changed, 7 strictly better, 5 show a higher count of one class and all 5 are E0615 on `data.len`, previously verified as REVEALED against byte-identical lines.
- The corpus value of this half is 2 diagnostics. It is worth having anyway: passing a local buffer to a kernel is exactly the shape a ported hand-written kernel needs, and without it the parameter fix is unusable from any caller that does not already hold a borrow.
- A test caught a THIRD defect while being written: `var tmp : [4]i32 = undefined;` lowers to `let mut tmp: [i32; 4];` with no initialiser, so E0381 fires before the call is ever type-checked. The fixture was changed to `[_]i32{0,0,0,0}` so the test measures the rule it is for and not that one.
- Mutation-checked twice: removing the reborrow guard fails 3 tests, removing the borrow itself fails 2.

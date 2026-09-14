# NOW -- The oracle answered only for names it rewrites (2026-09-08)

## The oracle answered only for names it rewrites (Refs #3412, #3408)

- My unknown-type check asked the Rust emitter "do you know this name", and the emitter answers by REWRITING (`int` -> `i32`). Spellings already valid in Rust -- `usize`, `isize`, `char` -- come back unchanged, so the test read them as undeclared. **565 of the 1045 remaining warnings, every one on `usize`, every one false.** Second false-positive class in my own check in as many passes, and again found by measuring my own output.
- Fixed by naming the three. Corpus: `unknown type` **1045 -> 478**, all warnings **1570 -> 1003**, exit codes unchanged at 573/78. Across both passes my own false positives fall **1283 -> 478**, a 63% cut, with the exit code of `check` never changing for any of 651 specs.
- `int_value_bits(&base).is_some()` was added as a second oracle and then **REMOVED**. Mutation showed why: dropping the explicit list makes `char` warn and the test fail; dropping `int_value_bits` changes nothing, because the list already covers `usize` and `isize`. Second redundant guard removed rather than shipped in as many passes, and by the same rule -- if I cannot make a test tell the difference, it is decoration.
- What remains is real: `Result` 69, `Float` 60, `String` 53, `Int` 32, `Trit` 26, `Bool` 23. Those names are declared by nothing.
- Sized for the next pass: only **6 specs** still warn on `Trit`, and **4 of them carry zero `use` lines**. One import line in `specs/ar/restraint.t27` takes its emitted Rust from 30 rustc errors to 19.

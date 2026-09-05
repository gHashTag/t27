# NOW -- Zig's `undefined` has no Rust spelling, and mapping it forces a judgement I am not making alone (2026-09-05)

Fifth class of the pass, measured in full and deliberately NOT auto-merged. The yield
is +2 against 698 rewritten seals, and the fix embeds a decision about what an
uninitialised enum is.

## The defect and its cost (Refs #3222)

- `expr_to_rust` returns an identifier verbatim, so `let mut info: EncodingInfo = undefined;` reaches rustc as an unresolvable name and stops **21 specs**
- the C emitter maps it under "Map Zig-specific identifiers to C equivalents" to `{0}`; every t27 position is type-annotated, so `Default::default()` has what inference needs
- but `Default::default()` cascades: a struct deriving `Default` needs every field type to derive it, and an enum field is where the chain broke -- **7 specs regressed** on `ErrorCode: Default` before enums got the derive too

## The judgement, stated rather than dressed up (Refs #3222)

- an enum deriving `Default` must designate a variant, and the spec gives only declaration order, so the first declared variant is used
- this is **not** the C emitter's `{0}`: that selects the enumerator whose value is zero, and **123 of the 146** enums in this corpus give their first variant an explicit non-zero discriminant
- where the two disagree the backends would disagree about what an uninitialised enum is -- Rust's first DECLARED variant against C's value zero, which may name no variant at all
- I first wrote the code comment claiming the two coincide; they do not, in 84% of cases, and the corrected comment now says so

## Measured, 650 specs, by name (Refs #3222)

- master 275; `undefined` with `Default` on structs only: 270, **+2 and −7**; with `Default` on enums as well: **277, +2, 0 regressions**
- predicted **+0 to +4** before measuring, and the prediction held
- the predictor is the one #3219 established: this class is **2% stub-bodied** (4 of 208 functions) against 100% for the two classes before it, so a signature-level fix could not clear whole files here
- blast radius **698 seal files**, `gen_hash_rust` and `sealed_at` only

## Why it is offered rather than merged (Refs #3222)

- +2 accepted for 698 rewritten seals is a thin trade, and the alternatives are worse: `std::mem::zeroed()` matches `{0}` exactly but is `unsafe` and UB for types with invalid bit patterns, and leaving the identifier unmapped keeps 21 specs stopped
- declining is a reasonable answer and the branch is left ready either way

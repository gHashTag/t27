# NOW -- A declaration that states its type is not a type alias (2026-08-29)

## The #2830 trigger, found and closed (Refs #2830)

- `gen_c_const` treats a const with a single identifier initialiser as a TYPE ALIAS when `is_type_name` accepts the initialiser
- `is_type_name` is "a primitive, or starts with `[`, or STARTS WITH AN UPPERCASE LETTER"
- so `const ARG0 : u8 = R1;` emitted `typedef R1 ARG0;` -- the declared `u8` discarded, a value used as a type name -- 23 of them in `specs/isa/registers.t27` alone
- the branch never looked at the declared type, and a declaration that STATES its type cannot be a type alias: `pub const PackedTrit = u8;` carries no annotation, `const ARG0 : u8 = R1;` carries one
- corpus: 21 specs / 55 occurrences -> 13 specs / 13; real type aliases unchanged
- my own hypothesis for this trigger failed its prediction test two hours earlier (102 predicted, 20 actual) because it tested "bare identifier" without the uppercase condition; a fan-out audit read the branch instead of guessing at it

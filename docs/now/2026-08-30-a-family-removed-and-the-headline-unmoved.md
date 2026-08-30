# NOW -- A family removed and the headline unmoved (2026-08-30)

## A family removed and the headline unmoved (Closes #2929)

- `param_type_to_c` had arms for `?T`, `[]T`, `[T; N]` and `[N]T` and none for `*T`, so 52 headers carried `uint64_t set(*Bitmap bitmap);`
- in C the star belongs to the declarator: written first, `Bitmap` parses as the NAME with an implicit-int type and the next identifier is unexpected
- files with the family **52 -> 0**; `cc accepts` **174 -> 174**
- that is the measurement, not a disappointment: only 20 of 404 rejected files are blocked by a single family, so no one family is a lever
- what it moves is what comes NEXT -- `member reference type 'Bitmap *' is a pointer; did you mean to use '->'?` is now its own single-family file, invisible while the parameter never parsed
- the arm recurses: `*[]u8` is `uint8_t**`, and prepending the star to raw text gives `[]u8*`

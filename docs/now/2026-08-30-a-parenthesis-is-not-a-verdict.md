# NOW -- A parenthesis is not a verdict (2026-08-30)

## Four rules measured before one shipped (Refs #2847)

- an invariant was discarded as "not a C constant expression" when its rendered text CONTAINED A PARENTHESIS, and `gen_c_expr` parenthesises its own binary expressions
- 2078 invariants discarded across 179 specs; 1065 contain a function call and 1013 contain no call at all
- rule "no call": discarded 2078 -> 1066 and dropped TEN specs out of compiling. Measured, not shipped
- rule "no call AND every name in const_defs": discarded rose to 3820, checks FELL from 3674 to 1932 -- `const_defs` misses enum constants, and `true` is not in it
- what shipped: no call, no empty operand, and every identifier is a C word or a `#define`/enum member. A `static const` is NOT an integer constant expression however constant it looks, and `const_defs` holds both kinds
- promoting the discards exposed two defects the comment had been hiding: `(BOARD_NAME != )` with a missing operand, and `#define CLOCK_FREQ_HZ 100_000_000` -- Zig separators C reads as a suffix
- the separator defect was 97 lines in 35 specs and is fixed here too; without it the invariant change is a regression, with it both are gains

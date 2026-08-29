# NOW -- A statement may stand where a clause head is expected (2026-08-29)

## A statement may stand where a clause head is expected (Refs #2778)

- for and while are keywords, so they were neither a clause head nor a boundary and the whole block fell back: 30408 -> 25093 tokens, 86 -> 77 specs
- the C backend then emitted invariant bodies at module scope (while loop outside of a function) and printed @as(usize,x) as (usize)(x); both fixed, cc 158 -> 163, ALL FOUR 63 -> 66
- Zig 217 -> 214: three specs whose recovered assertions now RUN, and one of them fails on its own merits (#2778) -- the first such find of this line
- parse-complete --fallbacks: 863 of 1034 fallback events are forall in two spellings; the name I first gave one arm was wrong and is corrected

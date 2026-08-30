# NOW -- A declared width beats an inferred one, one level out (2026-08-31)

## A declared width beats an inferred one, one level out (Closes #2952)

- Zig needs a width on a shift with a runtime amount; with nothing else the emitter read the literal's magnitude, so var half: i32 = 1 << d became @as(u32, 1) << ...
- 58 sites in 47 specs carry the shape; 33 are declarations that state their type, all i32 -- the type was two lines above the expression that guessed
- zig test --test-no-exec 133 -> 165 (+32, 0 regressions); zig build-obj 282 -> 282 (+0), predicted, because the corpus column cannot see a body nothing references; cc unmoved at 268 as control
- M3 survived first: the leak test put a second DECLARATION after the first, and an untyped one overwrites the hint anyway -- what bites is a return, with a different shift amount so CSE does not fold the two

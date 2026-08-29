# NOW -- A declared width beats an inferred one (2026-08-30)

## The lexer consumed the suffix and threw it away (Refs #2860)

- the lexer recognises `u8/u16/u32/u64/usize/i8..` on a numeric literal, advances past it, and RECORDS NOTHING, so `1u64` and `1` reach the AST identically
- the Zig shift path then re-invents a width from the literal's MAGNITUDE: `1` fits in u32, therefore u32
- `(1u64 << mant_bits)` was emitted as `(@as(u32, 1) << @intCast(mant_bits))`; Zig accepts it because the u32 coerces to the u64 return type
- demonstrated: `@as(u32, 1) << 39` panics with `integer does not fit in destination type`; `@as(u64, 1) << 39` returns 549755813887
- the suffix now rides in the lexeme and is split back off at the two ExprLiteral sites into `extra_type`, so `value` is byte-identical and no emitter changes
- the first attempt patched the lexer and MISSED both literal sites -- indentation -- so `1u64` leaked straight into the C and Zig output. Caught by regenerating one file, not by reading the patch
- honest effect: ONE spec's output changes, one cast. The audit reported 84 sites; 84 is how many suffixed literals exist, not how many reach this path

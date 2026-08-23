# NOW -- Zig-subset campaign, wave 3: four more gaps, twenty-one specs total (2026-08-23)

## Zig-subset campaign, wave 3: four more gaps, twenty-one specs total (Closes #2426)

- Tagged unions (union(enum) captured verbatim, not lowered to a struct — its variants carry payload types and emitting one as the other would silently change the layout), a call on an arbitrary expression (func.?(a,b), where the comment being replaced said it should not happen), and the inline unroll hint before for/while.
- 541 generate to 562, ledger 171 to 150 across three waves. Five of the seven target specs now generate; main_zig_handwritten needs an error-union return type and cap_test is a deliberate refusal of closures rather than a grammar gap.

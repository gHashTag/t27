# NOW — parser: tuple return types no longer silently drop the function (2026-08-05)

Last updated: 2026-08-05

## parser: tuple return types no longer drop the function (Closes #1709)

- Branch: `feat/tuple-return-parse`

### Что легло
- The function return-type parser had no `(` branch, so `-> (T, U)` desynced and the whole function was silently dropped from the AST (1 FnDecl vs 2). Added a LParen branch capturing `(T, U, ...)`. Functions with tuple returns now survive with the correct signature. Part 1 of tuple support (#1702); body-lowering and `let (a,b)` destructuring are follow-ups. Full suite 1496/0; FROZEN_HASH re-sealed.

---


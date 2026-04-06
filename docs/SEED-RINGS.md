# SEED-RINGS: Incremental Compiler Bootstrap Pattern

The SEED-RINGS pattern builds a self-hosting compiler one capability ring
at a time, following Abdulaziz Ghuloum's "An Incremental Approach to
Compiler Construction" (2006).

## Rules

1. **One ring = one capability.** Each ring adds exactly one language
   feature (e.g., const literals, function headers, type checking).
2. **Sealed with 4 hashes.** Every ring is frozen by recording the
   SHA-256 of the stage-0 compiler in `bootstrap/stage0/FROZEN_HASH`.
3. **Reversible.** Any ring can be reverted by restoring the previous
   `FROZEN_HASH` and compiler source.
4. **Cumulative.** Ring N includes all capabilities of rings 0..N-1.
5. **No meta-meta.** The bootstrap compiler never compiles itself;
   it compiles the *next* stage only.

## Layers (Oak Metaphor)

| Layer    | Description                         | Rings   |
|----------|-------------------------------------|---------|
| SEED     | Bare lexer + parser, const literals | 0       |
| ROOT     | Types, enums, structs, functions    | 1-3     |
| TRUNK    | Control flow, expressions, codegen  | 4-7     |
| BRANCH   | Modules, imports, generics          | 8-12    |
| CANOPY   | Optimisations, self-hosting         | 13+     |

## Ring Anatomy (9 Steps)

Each ring follows these steps in order:

1. **Branch** - Create `ring/N-<name>` from the previous ring's commit.
2. **Spec** - Write a `.t27` spec file exercising the new capability.
3. **Lex** - Extend the lexer to tokenise any new syntax.
4. **Parse** - Extend the parser to build AST nodes for the new syntax.
5. **Lower** - (Optional) Transform AST into a simpler IR if needed.
6. **Gen** - Extend codegen to emit Zig/LLVM for the new construct.
7. **Test** - `tri parse spec.t27` must succeed; `cargo test` must pass; **`cargo build`** in `bootstrap/` must succeed (includes `build.rs` language guard).
8. **Freeze** - Record `sha256sum bootstrap/src/compiler.rs` in
   `bootstrap/stage0/FROZEN_HASH`.
9. **Seal** - Commit, push, and open a PR that closes the ring's issue.

## Golden canon vs refactor debt

**Where the gold is** (ring-sealed truth): specs that parse/gen under **`tri`**, the frozen compiler hash, seals under `.trinity/seals/`, and policy docs. **Everything else** on the critical path that is not t27+**tri** is **refactor heap** until removed.

See **`docs/GOLDEN-RINGS-CANON.md`** for the **micro-iteration checklist**, **GOLD vs REFACTOR-HEAP** tables, and links to numeric/language debt inventories.

## Ring 0: SEED-0 (Const Literals)

Ring 0 establishes the minimal viable compiler:

- Lexer: whitespace, `//` comments, `;` comments, identifiers, keywords,
  numbers (decimal, hex `0xFF`, binary `0b10`), strings, operators.
- Parser: `module` declarations, `pub const` with type annotations and
  literal values (including negative numbers like `-1`), `enum(T) { }`,
  `struct { }`, `fn` headers with brace-skip bodies, `test`/`invariant`/
  `bench` blocks with brace-skip bodies.
- Validation: `tri parse tests/ring0_trivial.t27` and
  `tri parse specs/base/types.t27` both succeed.

---

## Related: language discipline

Agents must not treat “any script” as the product. **SEED** is the minimal Rust bootstrap to parse `.t27`; all other languages on the critical path are **debt**. See **`docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md`** for the Queen Lotus cleanup inventory and **[trinity](https://github.com/gHashTag/trinity)** umbrella alignment.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*

# NOW -- a reference nobody checked (2026-09-21)

## 126 specs emitted JavaScript and TypeScript that threw before their first line ran

- **What was actually wrong.** `const_value` answered every bare identifier with
  `js_name(name)` under a comment reading *"a reference to a const declared
  above"*. The comment was the only thing checking. Three shapes in the corpus
  walked straight through it: a name the backend had itself **refused** to emit,
  a name declared **further down** the same file (a `const` is not hoisted into a
  usable state), and a **primitive type name** used where a value was expected.
- **Why nothing caught it.** The artifacts are syntactically perfect, so
  `node --check` passes all 1414. An unbound identifier is a `ReferenceError` at
  **import**, and it arrives for the whole module at once -- one bad name costs
  every declaration in the file. The Spec Explorer showed a green backend chip
  over a file that cannot be loaded.
- **Three instruments, each catching what the previous could not.** `node --check`
  found **3** duplicate `export const` (a SyntaxError that kills the module).
  `import` of every emitted artifact found **13** unbound identifiers. `tsc
  --noEmit --strict` over all of them found **24** lost omission records and **2**
  lost enum discriminants. Stopping at any one of the three would have shipped
  the next two.
- **`Bound` now carries three sets** -- emitted, refused, declared -- and
  `resolve` says which of the four cases a lookup fell into, with the line
  number, instead of producing a name and hoping. A name that is genuinely an
  import is still named as such; the backend does not pretend to know it is not.
- **An enum that declares one variant twice.** `compiler/ast.t27` has
  `TokenType.And` at **16** (the operator `and`) and again at **99** (the Gherkin
  keyword `And`). An object literal keeps the **last** of a repeated key, so the
  artifact silently answered 99 to both questions. The first now wins, the
  displaced one is announced, and the discriminant counter still advances past
  it -- `But` stays **100**. TypeScript says this out loud as TS1117; JavaScript
  does not, which is why it is announced in the artifact rather than left to a
  type-checker to notice.
- **`__NOT_EMITTED__` becomes a frozen list, not an object.** Two entries for the
  same declaration is legal in a list and TS1117 in an object -- an error in the
  artifact the TypeScript backend exists to make type-check.
- **The count travels with the code.** `generate_reported` returns how many
  declarations the artifact announced instead of printing, and the wasm bridge
  publishes it as `targets.js.notEmitted`. A catalog can now mark a spec
  **partial** without regexing `__NOT_EMITTED__` back out of the code, which
  would be a second, weaker implementation of something the backend already
  knows exactly. The five compiled backends get no such field: `null` reads as
  *no such question*, where `0` would read as *no omissions*.
- **Measured over the whole corpus, not sampled.** All 1414 specs emitted through
  every backend, every artifact loaded:

  | | before | after |
  |---|---|---|
  | `js` backend fails | 335 | **209** |
  | `ts` backend fails | 335 | **209** |
  | artifacts that import | -- | **1205 / 1205** |
  | `tsc --strict` errors | -- | **1** |

  209 is exactly the set that produces **no AST at all**, which no backend can
  serve -- `c`, `rust` and `zig` fail the same 209. The js/ts-only failing family
  is **zero**. The one remaining type error is `neg_invalid_annotation`, a
  fixture that exists to be wrong.
- **69 specs now say what they left out.** 244 declarations announced across
  them. These are not whole modules and not broken ones, and the catalog no
  longer has to call them either.
- **What this does NOT establish.** The 209 specs that never parse are untouched
  and unexplained here. Which of them are damaged files, which are legacy
  dialects and which are real parser gaps is a separate question, and
  deliberately not answered on the way past.
- `cargo test --bin t27c`: **1736 passed, 0 failed, 2 ignored**.

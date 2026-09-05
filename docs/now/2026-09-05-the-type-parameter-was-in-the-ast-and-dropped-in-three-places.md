# NOW -- The type parameter was in the AST, and the Rust emitter dropped it in three places (2026-09-05)

Fifth compiler fix of the pass, and the first where the class did not close until
every position it occupies was repaired.

## The defect (Closes #3234)

- the parser keeps the list: `ArrayView(T)` arrives as `params: ["T: "]`
- the Rust emitter dropped it while keeping every USE, so 19 specs generated a struct whose fields name a type nothing declares
- no sibling backend had the answer this time -- C has no generics and writes `struct ArrayView`, and the Zig emitter drops the parameter too -- but the mapping is forced rather than chosen: the spec declares a type parameter and `<T>` is how Rust spells one
- the names come from the AST, not from a guess about single capital letters, and a whole-word matcher keeps a struct named `Trit` from being matched by a parameter named `T`

## Each position alone was worth nothing (Closes #3234)

| step | accepts | gain |
|---|---|---|
| master | 275 | |
| `pub struct Name<T>` on the declaration | 275 | **+0** |
| `<T>` on free functions that use it | 276 | **+1** |
| `Name(T)` -> `Name<T>` at every type USE | **288** | **+13** |

- zero regressions at every step, each measured by spec name
- the declaration alone changes nothing because the module-level free functions still name `T`
- both together change almost nothing because the uses keep the call-style spelling, and rustc says so precisely: "parenthesized type parameters may only be used with a `Fn` trait", in 12 of the 19

## What this corrects in my own predictor (Refs #3234)

- predicted **+12 to +19** for the first two steps, because the class is 100% stub-bodied (94 of 94 functions); measured **+1**
- then predicted **+6 to +12** for the third; measured **+13**
- the stub-share predictor of #3219 is an UPPER BOUND that applies only once every site of the class is repaired -- it says what the class could yield, never what a partial repair will
- a partial repair of a class yields roughly nothing, which is the older rule restated: a class is not closed until every position it occupies is enumerated

## The Rust column across this pass (Refs #3234)

- 224 at the start, 237 (#3208), 242 (#3213), 252 (#3216), 275 (#3219), **288** here
- **+64 in total, 0 regressions at every step**

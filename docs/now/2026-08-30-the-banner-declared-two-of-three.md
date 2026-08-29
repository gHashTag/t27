# NOW -- The banner declared two omissions of three (2026-08-30)

## A declared omission is not a hidden one, and one category was not declared (Refs #2875)

- the Rust banner reads `NOT LOWERED BY THIS BACKEND: 7 test(s), 2 invariant(s).` and says nothing about functions emitted as `unimplemented!()`
- 817 such stubs across 559 files, and a file holding two dozen of them read as complete
- the banner is emitted BEFORE the body, so counting them needs a pre-pass over the AST
- the `has_body` predicate was inline in `gen_fn`; it is now a function both the banner and the emitter call, because two copies of that list is exactly how `StmtAssign` went missing from one of them
- cross-checked over the corpus: banner total 817, emitted stubs 817, zero files disagreeing

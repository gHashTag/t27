# NOW -- `null` and `.?` needed no idiom choice, and I had priced them as if they did (2026-09-05)

Eleventh compiler fix of the pass, and a correction to my own pricing from two
iterations ago.

## The defect (Closes #3260)

- the source language spells the empty optional `null` and its unwrap `.?`, and both reached Rust verbatim: `if (base != null) { base.? }`
- rustc: `cannot find value `null` in this scope` and `unexpected token: `?``
- the generated signature is already `base: Option<Vec<u8>>`, so `null` is `None` and `.?` is `.unwrap()` -- direct equivalents, no judgement
- `Option<T>` compares fine, so `base != None` is the literal rendering of what the spec wrote, and every occurrence measured sits behind an `if x != null` the spec put there itself

## Measured: the class closes, the column does not move (Closes #3260)

- accepts **329 -> 329**, zero regressions
- first errors on `?` or `null`: **8 -> 0**
- specs still emitting either: **16 -> 1**
- predicted +2 to +6 and measured **+0** -- the eight move to their next error, not to acceptance. Closure is the honest measure here
- the one remaining is `null = 20,` in `specs/lsp/schema.t27`, an enum VARIANT named `null`, which is a legal Rust identifier; the arm is anchored on expression position and correctly leaves it alone

## What this corrects in my own reading (Refs #3260)

- two iterations ago I priced this class as "needs an idiom decision, `if let` versus `unwrap`" and set it aside
- the decision only exists if you want IDIOMATIC Rust. The FAITHFUL rendering is mechanical
- reading the generated signature settled it in one command, and I had not read it -- I had reasoned about the shape of the expression instead
- this is the second time this pass that a class I priced as needing a decision turned out mechanical once I looked at the type rather than the syntax

## Anchored where it belongs (Refs #3260)

- inside `expr_to_rust`; `expr_to_string` carries an identical `ExprIdentifier` arm, is not a backend, and must keep returning the name
- the same trap as the keyword-field-name fix, where an assertion on the occurrence count caught that there were two

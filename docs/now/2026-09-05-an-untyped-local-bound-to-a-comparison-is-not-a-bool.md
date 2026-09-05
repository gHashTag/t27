# NOW -- An untyped local bound to a comparison is not a bool, and only the annotation differed (2026-09-05)

Twelfth compiler fix of the pass, and the second hole in the same function family:
#3249 fixed a module-level bool that was not in `bool_vars`; this is a function-local
one that was not either.

## The defect (Closes #3263)

- `collect_bool_locals` admits a local only when its declared type is literally `bool`
- a local with NO annotation, bound to a comparison, was not a bool to the emitter, so its use in condition position got the integer guard
- six lines reproduce it and the two locals sit one line apart: `let typed: bool = (x > 1);` gives `if typed {` and `let untyped = (x > 1);` gives `if (untyped) != 0 {`
- measured against master: **330 -> 333, +3, 0 regressions**, predicted +0 to +3 and landing at the top of the range

## The half that could not be asked (Closes #3263)

- `expr_is_bool` already answers this, but it is a `&self` method consulting `bool_vars` and `bool_fns`, and `collect_bool_locals` runs BEFORE those exist
- split out the syntactic half, which needs no state: a comparison, a logical operator, a `!`, or a `true`/`false` literal is bool whatever those sets end up holding
- the inference requires the annotation to be EMPTY rather than merely non-bool: a local annotated `u32` and initialised from a comparison is a spec defect and must keep failing visibly rather than being silently reclassified

## What the fan-out was worth here (Refs #3263)

- nine agents against a pinned binary; the operands lens sized this class at 3 and labelled it **mechanical**, and the label held
- its control was total: **0 of the 329 accepted files** contain an untyped local bound to a bool-valued expression, against 3 of the 251 failing
- the same lens independently sized the `.len()` class at 10, which is the class #3257 addresses -- an agreement I did not prompt for and did not need to check by hand

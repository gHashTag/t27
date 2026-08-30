# NOW -- A control on the artifact is not a repair in the emitter (2026-08-31)

## Four skill sections from instruments that read as findings (Refs #2990)

- ci-gates 422: a hand patch of a GENERATED file proved the `disable fork;` diagnosis with one line changed, and was quoted as though it were the fix; the named block it disables exists at one site, for one branch of one of three loop forms, and the emitter has no idea which loop encloses a `break`
- ci-gates 423: `false && A || B` disables only the left disjunct, so the mutant looked like a survivor and then the arm looked like dead code; both readings were the instrument, and the honest score is 5 of 5
- ci-gates 424: the accident inside that broken mutation was true -- `extra_kind == "float"` appeared exactly once in the compiler, on my own line; grep for something WRITING a value before reading it in a guard
- ci-gates 425: `etx_of_half_by_half_is_four` expects 4.0 and had been passing THROUGH the rounding defect its two siblings fail on; a prediction that fails in a specific direction locates the next defect
- filed from the same sweep, not fixed: #2987 the Icarus gate has had zero targets since specs/scratch was untracked, #2988 `break` lowers to `disable fork;` with no `fork` anywhere in the corpus, #2989 an early return inside a loop leaves the loop running and binary_search never terminates, #2992 `implies` appears 82 times in the corpus and 0 times in the compiler

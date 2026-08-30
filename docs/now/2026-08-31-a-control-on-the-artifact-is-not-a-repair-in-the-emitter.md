# A control on the artifact is not a repair in the emitter

- ci-gates 422: a hand patch of a GENERATED file proved the `disable fork;`
  diagnosis with one line changed, and was quoted as though it were the fix.
  The named block it disables exists at one site, for one branch of one of three
  loop forms; the emitter has no idea which loop encloses a `break`.
- ci-gates 423: `false && A || B` disables only the left disjunct. The mutant
  looked like a survivor, then the arm looked like dead code. Both readings were
  the instrument. Mutate a whole arm; score was 5 of 5, not 4 with a mystery.
- ci-gates 424: the accident inside that broken mutation was true --
  `extra_kind == "float"` appeared exactly once in the compiler, on my own line.
  Grep for something WRITING a value before reading it in a guard.
- ci-gates 425: `etx_of_half_by_half_is_four` expects 4.0 and had been passing
  THROUGH the rounding defect its two siblings fail on. A prediction that fails
  in a specific direction locates the next defect; one that is merely confirmed
  teaches nothing.

Refs #2990

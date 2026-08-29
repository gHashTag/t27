# NOW -- An alias is not a second opinion (2026-08-29)

## Two more reasons the ambiguity refusal was firing without cause (Refs #2764)

- `specs/math/sacred_physics.t27` declares `const PHI : f64 = constants::PHI;` -- a re-export, not a competing definition; it names the other candidate
- when every candidate but one is an alias pointing at that one's module there is a single definition, and the others say so
- splice the TARGET, never the alias: the alias's text is `constants::PHI`, and the flat output has no `constants` namespace
- `base/types.t27` writes `pub const ONE : i8 = 1;  // Trit = +1` and `base/ops.t27` writes it without the comment -- a comment is not part of a declaration
- refusals 29 -> 14 across 460 importing specs, specs with a refusal 14 -> 9, and the nine that remain are all real duplicate implementations
- `unknown type name` went 802 -> 803: the splice puts a bare `PHI_INV` where `sacred_physics::PHI_INV` stood, and the pre-existing #2830 defect turns `const PHI_TARGET = PHI_INV;` into `typedef PHI_INV PHI_TARGET;`
- that is a defect reached, not a defect added -- and #2830's count was measured on a filtered population: the corpus figure is 20 specs / 53 occurrences, not 8 / 35
- hypothesis "the trigger is a bare-identifier initializer" FAILED its prediction test: 102 specs predicted, 20 actual, counts matching in 2 of the 12 shared. Reported as a failure, not as a rule

# NOW -- the evening's emitter method becomes ci-gates section 13 (2026-08-23)

## measure the radius, then measure each arm (Refs #2325)

- Section 13 records what made 573 -> 186 possible: measuring the blast radius
  before writing code (100 of 438 structs), measuring each arm of a fix
  separately and believing the zero (the obvious arm moved nothing), verifying
  ordering instead of assuming it, the fact that fixing one flattening class
  can create another one layer out (caught by the smoke set in one run), that a
  control which does not fire is not a control, and holding the result with a
  per-module ratchet.

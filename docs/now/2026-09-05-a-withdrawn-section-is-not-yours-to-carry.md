# NOW -- a withdrawn section is not yours to carry (2026-09-05)

## a withdrawn section is not yours to carry (Refs #3195)

- tri skill renumber now refuses when the tail carries a section the base REMOVED on purpose, naming it. The rule from the previous pass lived in a report; it lives in the command now
- withdrawn_titles takes an injected predicate and is covered three ways without git, but inverting the git log -S emptiness test survived all of them. The control is a real repository built by the test: main adds a section, the branch takes it and appends its own, main withdraws it. Both git-touching mutants die there, and the test carries its own negative control
- the first fixture put the withdrawn section in the base region rather than the tail, so the rebuild dropped it correctly and the test failed because the TOOL was right. Reproducing a defect means reproducing its position, not only its ingredients

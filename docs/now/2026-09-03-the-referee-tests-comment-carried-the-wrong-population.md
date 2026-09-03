# NOW -- The referee test's comment carried a population that never shipped (2026-09-03)

## 37 was patch C's number; what landed moves 7 (Refs #2988)

- `a_loop_without_a_jump_is_unchanged` is the test that DEFINES the population a guard-flag change must not move, and its comment said *"the corpus delta is 37 of 581 generated .v files"*
- 37 is patch C's figure, for a change that guarded `return` as well -- the half that was deliberately cut before landing. What shipped moves **7**, re-measured twice by independent readers, and `5063edb19` re-sealed **19** seals over those 7 specs
- the sentence also listed `return` among the statements the fixture avoids; the shipped guard does not know about `return` at all
- corrected in place, with the reason, rather than left standing beside a newer number

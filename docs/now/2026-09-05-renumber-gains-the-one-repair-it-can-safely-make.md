# NOW -- renumber gains the one repair it can safely make (2026-09-05)

## renumber gains the one repair it can safely make (Refs #3195)

- of four refusals in tri skill renumber, three are right to refuse: when the rebuild would drop a section the safe action is unknown. The fourth is settled -- a section the base removed on purpose -- and --drop-withdrawn removes exactly those and names each, opt-in because deleting text nobody asked to delete is how a tool earns distrust
- the first version did what it promised and then died on titles_lost, the guard added three passes ago to stop a silent deletion: the section IS on disk and IS gone from the rebuild, and the operator asked for that. A guard has to know what the operator authorised, or the authorisation is not real
- two structural tests failed and both were right to: they pin the SHAPE around each guard, and turning a two-armed refusal into a three-armed opt-in changed it. A structural test that survives a restructuring of the thing it pins is not pinning it. Both rewritten to assert the new invariant; four mutants killed by unit tests and the scratch-repo control alike

# NOW -- The third module form, and a test that proved nothing (2026-08-29)

## Three lessons from re-measuring my own ruler (Refs #2833, #2835)

- ci-gates 234 says the corpus writes TWO indentation conventions; it writes three, and the third is 231 of 650 specs
- bracket depth zero reads a third of the corpus as empty and reports a smaller number, which after a fix reads as progress
- a survey of forms is a measurement and needs a denominator -- one grep per form would have said 392 / 231 / 27 before any ruler was written
- the obvious repair (smallest definition indent) readmits the body bindings the previous repair removed: four rulers, three wrong, each differently
- when repair N reintroduces what repair N-1 removed, the two failures are one unstated problem -- here, that "top level" is a parse question
- the kind-filter test passed with the filter removed: its fixture put the const where depth excludes it anyway
- a mutation check is not a formality to confirm what you expect; it is what separates a test of your code from a test of something adjacent

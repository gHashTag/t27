# NOW -- LAW 8 measured over twelve edge kinds (2026-09-04)

## LAW 8 measured over twelve edge kinds (Closes #3112)

- Correction to my own headline: the graph carries 12 edge kinds and three are documentation relations; documented-by is the INVERSE of a dependency, so counting it makes a spec depend on its own documentation
- Drop one kind at a time: all 91 edges give cycles 1 / backward 5; dropping documented-by gives 0 / 3; dropping references gives 0 / 5. The single cycle is a documented-by, a references and an import in series
- Over dependencies LAW 8 has 0 cycles and 3 backward edges -- affects_benchmark t2->t1, codegen t6->t2, and math/constants importing a docs node
- Both readings are printed and both are held, so nobody has to trust one file's opinion of which kinds are documentation; a planted import cycle fails the dependency ledger, a planted documentation cycle moves only the all-edges one

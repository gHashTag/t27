# NOW -- A corpus of already-broken files is not a control (2026-09-03)

## Two sections from getting #2997 wrong first, and from measuring a wobble (Refs #2997, #3006)

- ci-gates 429: repairing the range-in-a-bound defect I also renamed a `_` capture, copying C, and produced `register `__t27_i' unknown` in every file it touched -- while `iverilog` read 380 before and 380 after with an identical accepted set, because all 36 carriers were already inside the failing 201, on the very defect being repaired
- that is the inverse of "acceptance cannot see a defect that compiles" and worse: when the population you are fixing is entirely inside the failing set, the corpus is not a weak control, it is no control, and every aggregate you quote is reassuringly flat
- the rule: before quoting an unchanged aggregate as evidence of no regression, ask how many of the touched files are in the failing set; if it is all of them, say so in the same sentence and get the evidence from something that executes
- ci-gates 430: the first determinism harness read 1 / 3 / 2 differing files and the second read 4 / 2 / 4 on the same tree with the same binary -- neither wrong, because which files wobble is itself a draw; only the union of NAMES is stable, and it is four specs, two of them wobbling in all three backends
- three counts read as three independent emitter defects; four largely-shared names read as one shared path four specs reach -- a different investigation, and a tractable one

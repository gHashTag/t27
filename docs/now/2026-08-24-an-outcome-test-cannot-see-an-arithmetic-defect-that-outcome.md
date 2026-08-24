# NOW -- An outcome test cannot see an arithmetic defect that outcomes tolerate (2026-08-24)

## An outcome test cannot see an arithmetic defect that outcomes tolerate (Closes #2161)

- if t > hf: mant += 1 in _magadd survived the boundary operator. It is the round-half-to-even decision: with >= the pair becomes round-half-UP and the parity test on the next line is dead. Measured: _magadd(25600, 20480) gives 25600 clean and 25601 mutated [measured].
- The self-tests did not notice because they train a net and check ACCURACY. An optimiser absorbs a last-bit error in every addition without changing whether XOR reaches 4/4 — the mutant passes every training assertion in the file.
- Three assertions pin the decision, one per branch: tie with even s must not round up, tie with odd s must, strictly above half must. Negative controls: weakening > to >= fails the first, disabling the parity test fails the second.
- And a correction: I planned this tick around line 203, which I had listed yesterday as a survivor without checking. It is killed — the XOR self-test catches it 1/4. A finding recorded as a line number expires, and I wrote that rule myself.

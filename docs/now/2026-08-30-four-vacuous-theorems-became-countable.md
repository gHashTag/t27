# NOW -- Four vacuous theorems became countable (2026-08-30)

## Master was red and it was not the branch that found it (Refs #2882)

- `corpus_classifier_matches_lean_completeness` failed on a CLEAN `origin/master` checkout, before I touched anything
- none of the four specs contains the word `else`, so the parser change in the branch this surfaced from cannot reach them
- cause: #2882 taught the parser to read a hyphenated module name whole, so four specs that could not be classified became classifiable
- the classifier is right: `t27c gen-rust specs/github/auth.t27` exits 1 and emits 0 lines. The Lean theorem, written while the spec did not parse at all, is the stale side
- a SECOND ratchet then fired: `max_vacuous`, which moves down only. The four Lean models are genuinely empty -- `functions := []`, `globals := []`, `tests := []` -- so `model_empty: true` is a fact, not a copy
- those four were ALWAYS vacuous; they became COUNTABLE. Nothing new became vacuous
- raised 40 -> 44 with the reason written into the ledger, and with it said plainly that this is the second-best answer: the right repair is real Lean models for those four
- I set `model_empty` by copying the shape of an existing entry before checking. It happened to be true; checking came second, which is the wrong order

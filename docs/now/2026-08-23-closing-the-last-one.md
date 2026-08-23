# NOW — closing the last one revealed the count was wrong (2026-08-23)

`fuzz_trainer.py` was the last gate with no control. Closing it produced three findings, and the third makes the first two look small.

- **Two defects in two lines of a shared helper.** `skip()` hard-coded its own name, so `fuzz_trainer.py` announced *"SKIP verify_trainer_c"* while running something else. And it had no `--require`, while its sibling has one — **two of three trainer checks in one workflow job could silently pass on a missing compiler while the third refused.**
- **A crash at import.** `ROUNDS = int(sys.argv[1])` read position, not meaning, so `fuzz_trainer.py --require` died with a `ValueError` before `main()` and before any verdict.
- **A case that passed for the wrong reason.** The planted divergence hit `run_model`'s return instead of `run_c`'s. The counterexample case failed loudly; the **length** case passed, because a shortened model is also a length mismatch — satisfied by a divergence planted where it was never meant to be.

**Then the count moved 18 → 21.** Chasing a newly-visible gate produced a claim of my own: *"verify_trainer_c.py has no non-zero exit at all — a CI step named 'Prove the WHOLE trainer bit-exact' that cannot fail."*

**That claim is false.** Its last line is `sys.exit(0 if ok else 1)`. My grep looked for `sys.exit(` followed by a digit and could not see a ternary — and `is_gate_by_property` had the **identical** blind spot, so it classified three more CI gates as not-gates.

The campaign wrote `verdict_literals()` for exactly this, months ago. The selector reintroduced the blindness as a substring shortcut, and I reintroduced it a third time in a one-off grep while investigating.

Real state: **21 gates, 4 with no control in any form** — not 18 and 1.

**The rule:** when you write a quick grep for a property the codebase already has a parser for, you are choosing the version with the known bug.

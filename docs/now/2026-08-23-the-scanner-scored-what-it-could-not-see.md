# NOW — the scanner scored what it could not see (2026-08-23)

Third defect in `tri gates mutate` in two days, and the one that moves its numbers most. Found by asking why it reported a gate as having no failure path when the gate plainly has four.

- **It matched only a bare `return 1..4`.** Measured across the twelve gate scripts: **34 failure paths seen, 8 missed** — seven ternary returns and one `raise SystemExit(3)`. The denominator was short by a fifth of what the command claims to measure.

- **`pack_index_consistency_gate.py` reported "no failure path to break".** Every verdict in that gate is a ternary:

  ```python
  return 0 if not fails else 1
  return 1 if bad else 0
  raise SystemExit(3)
  ```

  A path the scanner cannot see is a path it scores as covered. That is the same substitution of a convenient rule for the thing it stands for that the command exists to catch — the third time this tool has committed the class it hunts.

- **Fixing the instrument raised the count, which is the honest direction.** Sites scanned 34 → 42; survivors 13 → 20; gates with survivors 8 → 9. Two gates gained sites that turned out to be already covered (`check_catalog_count` 3/3 → 4/4, `wp18_conformance_gate` 0/2 → 4/6). `check_elab_ratchet` gained two genuinely uncovered ones that nothing could see before.

- **A ternary is neutered whole, not per arm.** Either arm may be the failing one, and rewriting one leaves the other reachable — a mutant that changes nothing and then "survives" is a gap invented by the tool.

- **`yields_a_verdict()` reads a standalone `1..4` only.** `return t27c_failures` and `return code2` contain a matching character and are not verdicts; `raise SystemExit(main())` is a dispatch. This command has already invented one finding, and the cost of an invented site is higher than the cost of a missed one: a missed site stays an open question, an invented one is published as a defect in somebody's code.

- **Four unit tests, two of them negative:** a digit inside an identifier is not a verdict, and a bare name is not one either.

- **The pattern across all three defects in this tool.** It took the first control flag instead of the set; it held a 1:1 control map that could not express a shared control; it matched one syntactic form of a return. Every one is scope decided by what was convenient to write rather than by what the rule is for — §14 class H, in the auditor rather than in what it audits.

Refs #2468, #2470

# NOW -- the control certified a copy of itself (2026-08-23)

Refs #2325.

- `check_duplicate_agreement.py --self-check` built a literal
  `fake = {"x": {"aaaa": [...], "bbbb": [...]}}`, evaluated a comprehension
  written inside the control, and returned BEFORE the reporting block. It
  proved that the copy of the comparison living in the control worked.
- Measured against three mutants of the real logic -- verdict flag inverted,
  digest grouping key destroyed, a bare `return 0` planted ahead of the report:
  **old control 0 of 3 killed, new control 3 of 3.** CI runs this immediately
  before the gate itself, under a step that exists to make the gate falsifiable.
- The control now spawns the whole file as a subprocess against a planted tree
  with a genuine `tmul` divergence, so `scan()`, the grouping, the report block
  and `main()`'s own return value all execute. It asserts the message AND the
  exit code: a fixture that stops compiling makes the child exit 1 through the
  unrelated "no duplicated function found" guard, so an exit-code-only
  assertion would go wrongly green. Verified in that direction too -- breaking
  the fixture on purpose prints `split = False` and exits 1.
- Two sibling gates already did it this way (`check_json_parses.py`,
  `check_withdrawn_live.py`): plant a fixture, call the real scanner.

## a false measurement of my own, on the way

My first comparison ran the OLD script from `/tmp`, where `ROOT` resolves to
`/`, so `scan()` found no specs, `found` was empty, and the old control
returned 1 -- for the wrong reason. It looked like the old control killed all
three. Re-run from the real `tools/` directory it kills none. An instrument
placed outside the tree it measures answers a different question.

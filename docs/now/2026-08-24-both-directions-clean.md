# NOW — both directions clean (2026-08-24)

Two mutation operators now run over thirteen gates. The loud one has **zero** survivors; the silent one has one, and it is the branch declared uncovered with its reason.

- **Report modes were the last three.** Each sat behind a flag CI never passes, and each file said so in its own words. Covering them is one case apiece: run with the flag, demand exit 0 and the text that identifies the mode.

  ```
  check_specs_parse.py     --all                  "a report, not a gate"
  check_elab_ratchet.py    --allow-missing-tools  "reporting nothing, deliberately"
  check_specs_generate.py  --summary              a counts table
  ```

- **The distinction is worth keeping; leaving the exit code unmeasured is not part of it.** A report that prints its table and then reports failure breaks any script reading it, and nothing would have said so.

- **The opt-out one is the sharpest.** `check_gate_preconditions.py` proves the ratchet reds when its tools are missing. Nothing proved that `--allow-missing-tools` then returns **success**. An opt-out that fails anyway is worse than none: it teaches that the flag does not work, and the next person removes the guard instead of passing it.

- **Two gates report "no success path to break", and that is honest rather than clean.** `pack_index_consistency_gate.py` and `wp18_conformance_gate.py` hold no bare `return 0` at all — every verdict is a ternary or a `SystemExit`. The loud operator takes only a bare `return 0` on purpose: a ternary can yield 0 on one arm and a verdict on the other, so forcing the whole line to 1 would be the silent operator seen backwards and scored against the wrong control. **Those two are unmeasured in this direction, not covered**, and the report says which.

- **The one silent survivor is the declared one.** `check_elab_ratchet`'s no-baseline branch needs a smoke build that *succeeds* before it can fail the way it fails; planting a fake success would test the plant. Its line has now moved four times — `:390`, `:403`, `:495`, `:516` — and the declaration matched every time, because it names the branch by message.

Refs #2492

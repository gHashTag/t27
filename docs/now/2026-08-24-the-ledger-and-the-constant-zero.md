# NOW — the ledger and the constant zero (2026-08-24)

`check_specs_parse.py` compares `now > was`, where `was` is a spec's recorded discard debt. Every case in its control planted into a spec whose recorded debt is **zero**, so `now > was` and `now > 0` were the same expression.

- **Measured.** `if now > 0` planted in the gate: the control passes (exit 0), and the **live gate turns red** (exit 1). That mutation ignores the ledger entirely — every spec carrying debt would fail — and nothing in the control would have said which change did it.

- **The distinguisher is not more debt; it is debt UNDER the recorded figure.** A spec that owes 1,139 tokens, planted with about ten. Correct code is silent. `now > 0` raises a false alarm. So the new case asserts **silence**, and it is the only case in that control that does.

  ```
  now > 0     was: control passes   now: killed
  now >= was  killed both before and after
  now != was  killed both before and after
  no-op       silent -- the control does not cry wolf
  ```

- **The file had already reasoned about this half-way.** `_CONTROL_SPEC` is chosen as a REQUIRED spec with *zero* debt, deliberately, with a comment: a spec owing 1,139 tokens would swallow a ten-token plant and make the drift case vacuous. That reasoning is right for the drift case and is exactly what left the `was`-versus-zero question open. **Two cases with opposite requirements need two specs, not one rule.**

- **And the guard nobody had seen fire, now fired.** #2469 added a `try` around `json.loads(out)` in wp18's control so garbage stdout records `[FAIL]` instead of aborting with a traceback. It was correct by construction and unproven. Planting `print("not json at all")` in the gate: `[FAIL] TP0_whole_program_clean_exit0`, exit 1, **zero tracebacks**. Without it the control would have died there and left every later case unrun.

- **#2472 is closed.** Its four items: the wp18 drift verdict (refuted — three mutations of increasing narrowness, all killed), this ratchet gap (fixed), `check_seal_coverage`'s two uncovered paths (stated in the file), and the `json.loads` guard (now demonstrated).

- **The new-blindness check I owed.** Three files now delete a sibling's cached bytecode on every run. In CI the two that share a workflow are **sequential steps**, and ten concurrent pairs run locally produced zero failures. No race, measured rather than assumed.

Refs #2472

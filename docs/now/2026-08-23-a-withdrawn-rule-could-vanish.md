# NOW -- a withdrawal could be reversed by deleting one line (2026-08-23)

Refs #2325.

- `check_withdrawn_live.py` asked one question -- is a retracted number stated
  in a live document -- and guarded only against the registry being emptied
  ENTIRELY. Deleting a single row was invisible.
- Measured by removing each of the 7 rows in turn: **6 of 7 vanished with the
  gate printing OK and exiting 0**. The seventh, 323 MHz, is pinned only
  because `self_check()` happens to hardcode that string. Coverage genuinely
  lapses -- a planted "41.2 GOPS" scores 1 hit with the rule and 0 without.
- The reverse direction now runs before the forward one. No new data file: the
  baseline already records WHICH pattern excused each accepted occurrence, so a
  pattern present there and absent from the registry is a deleted rule whose
  exemptions stayed behind.
- After: **0 of 7 delete silently.** Four controls held -- `--update-baseline`
  writes the same 20 entries byte for byte, `--self-check` still passes, a new
  rule matching nothing keeps the tree green (a rule with zero hits is the
  healthy end state, so "every rule must have a hit" would be the wrong gate),
  and a planted live "41.2 GOPS" is still caught by name.

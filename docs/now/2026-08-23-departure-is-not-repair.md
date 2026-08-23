# NOW -- a spec leaving the index was scored as a repair (2026-08-23)

Refs #2325.

- `check_specs_generate.py` built `bad` only from `git ls-files "*.t27"`, and
  then computed `fixed = known - bad`. That folds two different events into one
  congratulation: a spec that started compiling, and a spec that stopped being
  TRACKED while still failing on disk.
- Reproduced: untracking a debt spec printed
  `NOTE 1 spec(s) in the baseline now generate` and exited 0.
- It has already happened at scale. Commit `2255e4c32` removed 58 ledger lines
  with 455 deletions and 0 modifications to the specs themselves; its own
  message concedes "those 58 would read as fixed".
- Departure is now its own class: `DEPARTED` is reported by name and fails.
  `fixed` is `(known & tracked) - bad`. Control fires; clean tree still exits 0
  with 712 specs / 541 generate / 171 known-broken.
- Not fixed here, filed instead: `generates()` returns True on `rc == 0` alone
  and never opens the artefact, so a 0-byte spec counts as generating. Two
  tracked 0-byte specs exist. That is a bigger change than this one.

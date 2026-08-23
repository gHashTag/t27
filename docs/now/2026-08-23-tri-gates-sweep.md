# NOW -- `tri gates sweep`, and the four gates it found (2026-08-23)

Refs #2325.

- Running every gate and its negative control was a hand-typed loop six times
  today. On the sixth I typed it wrong -- zsh does not word-split a variable,
  so `set -- $c` passed the whole string as one argument and five gates
  reported exit 2. I read that as a regression for a minute. Now a command.
- What it found on the first run: **4 of 12 gate scripts have no negative
  control at all** -- `check_catalog_count.py`, `check_catalog_integrity.py`,
  `check_elab_ratchet.py`, `check_vector_data.py`. Three of those four are
  files I wrote or heavily edited today.
- It distinguishes a control that lives in a SEPARATE file
  (`wp18_conformance_gate.py` is checked by `wp18_selftest_gate.py`) from one
  that is missing, and excludes control scripts from the gate list -- counting
  either wrong would invent a finding out of the naming convention. Two unit
  tests pin exactly that.
- **A defect in the command itself, found before shipping.** Gates that need
  `--ssot/--vectors` exit 2 from argparse without running, and the first
  version printed `2` in the run column -- reporting a refusal to parse
  arguments as a gate verdict, which is the exit-status-mistaken-for-the-
  property mistake this command exists to surface. It prints `args` now, and
  says what that means.

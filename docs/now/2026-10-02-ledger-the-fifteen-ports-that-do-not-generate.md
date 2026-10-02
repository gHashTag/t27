# NOW -- Ledger the fifteen ports that do not generate (2026-10-02)

## Record fifteen broken ports so the gate catches the next one (Refs #5497)

- `check_specs_generate.py` failed on master: 15 specs under specs/port/ do not generate with any backend. Each was merged while this step was already red.
- Making them parse is not a repair: on #5473 seven were made to parse and their generated C/Zig still had dozens of errors per file. Their ports are to be redone (#5549, one row per spec with its error and the PR that added it).
- Until then they join `tools/specs_generate_baseline.txt` by hand with the compiler's own first line, which is how the tool means the ledger to grow (`--update-baseline` refuses to grow it). Ledger 14 -> 29; the gate passes (1212 specs, 1183 generate) and its self-check passes.

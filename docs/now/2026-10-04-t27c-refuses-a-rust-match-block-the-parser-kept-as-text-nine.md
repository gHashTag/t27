# NOW -- t27c refuses a Rust match block the parser kept as text; nine become switch (2026-10-04)

## t27c refuses a Rust match block the parser kept as text; nine become switch (Closes #5949)

- typecheck refuses every match block the parser captured as text (check_captured_match), naming its line and fn; parse still accepts it, and no generated output moves from the compiler change
- 9 of the corpus's 31 captured blocks are rewritten as t27 switch or if, in auth/config, ar/restraint, ar/coa_planning, igla/training/roadmap, igla/evaluation/multi_lang_harness and ar/composition
- git/diff's 21 blocks match on Ok/Err of shell commands t27 cannot express, so it is ledgered red at typecheck: suite cap 112 -> 113, the reason recorded in _why_entries_rose

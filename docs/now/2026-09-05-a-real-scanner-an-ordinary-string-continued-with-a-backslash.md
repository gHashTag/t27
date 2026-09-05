# NOW -- A real scanner: an ordinary string continued with a backslash is not code (2026-09-05)

## A real scanner: an ordinary string continued with a backslash is not code (Refs #3255)

- test_module_lines used per-line rules that could not express an ordinary string continued with a trailing backslash. types_dup.rs holds const THREE_SPELLINGS: &str = backslash-continued, whose fixture has a column-0 brace at line 1115; that ended the module and handed 29 test-only literals to the mutator.
- Replaced the per-line rules with code_mask, a single character pass over comments, ordinary strings with escapes, raw strings with hash-counted closes, and char literals as distinct from lifetimes.
- Measured against master's binary: types_dup.rs offered 71 to 40, skipped 11 to 42, sites inside its own test module 29 to 0. competitors.rs holds at 45 offered and 0 in-module. Nine other files unchanged.
- Five mutants killed: the ordinary-string rule off, escapes ignored, block comments not nested, the raw close ignoring hash count, and the caller ignoring the mask. Two controls had to be sharpened first - the originals did not distinguish the mutants.
- The prev_is_word guard is now a decoration again and is removed: it was load-bearing in the per-line scanner that no longer exists. Necessity is a property of the scanner's shape, not the check.

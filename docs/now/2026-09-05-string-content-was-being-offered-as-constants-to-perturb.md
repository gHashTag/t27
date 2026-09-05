# NOW -- String content was being offered as constants to perturb (2026-09-05)

## String content was being offered as constants to perturb (Refs #3255)

- masked() is language-agnostic and treats # as a comment opener, so r#" read as r plus a comment to end of line and the raw string's CONTENTS came back as code. A probe file was reported as 4 literal(s), offering 12345 and 6789 from inside an r#"..."# fixture.
- test_module_lines ended a #[cfg(test)] module at a line that is exactly }, and competitors.rs has such a line inside the raw string const TWO. Everything after it was reported as production: 36 offered sites sat below that file's own #[cfg(test)].
- Fixed both. competitors.rs: offered 114 to 45, skipped 14 to 77, sites inside its own test module 36 to 0. fpga.rs skipped 637 to 617. Nine other files unchanged. Baseline measured by building master's own mutate.rs and gates.rs.
- Reported not fixed: types_dup.rs uses an ORDINARY string continued with a backslash, whose fixture has a column-0 brace at 1115; 29 test-only sites still offered. A quote-parity rule was written and reverted after measurement - it took fpga.rs to 993 offered and 0 skipped.
- A prev_is_word guard is a decoration in masked(), where the ordinary-string rule reaches those bytes first, and load-bearing in raw_string_opens, which has no such rule. Only measurement told them apart.

# NOW -- The largest first error is not the largest lever (2026-08-30)

## The largest first error is not the largest lever (Closes #2925)

- C generates for 578 specs and cc accepts 174; the 404-file gap ranked by FIRST error put `default_input`/`valid_input` on top with 74
- measured before building: of the 166 files carrying that error, **0** would compile if it were the only fix -- every one has other independent families
- ranking by first error ranks by POSITION IN THE FILE, not by blocking power; only **20 of 404** files are blocked by exactly one family, median is 4-5
- the real find came from the 4 single-family files: `const EXP_OFFSET: u32 = 1792...173` (185 digits) typechecked clean and three backends emitted it verbatim
- ten constants across five specs; only cc ever said anything, and it is the fourth backend
- two branches of my own fix were DEAD with comments explaining why they were needed: `parse::<u128>` returns Err rather than overflowing, and the lexer already strips separators -- both removed after their mutations changed no test

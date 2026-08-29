# NOW -- The fix I set out to do was not work, and the tool said so (2026-08-30)

## The fix I set out to do was not work, and the tool said so (Refs #2864)

- Took the 45 located items as 'work without research'. Probed the first family -- t *= 2.0, float parameters, f32 returns -- and every one COMPILES in isolation. The items were not defects.
- Cause: locate did not filter by stage. report learned that split last pass and locate did not, so 8 of its 40 answers were TYPECHECK failures, where 'the item whose presence causes the failure' is a category error. A type error already names its line AND its reason.
- Fixed: locate now answers only for parse failures and prints what it excluded -- 10 typecheck, 4 lex. 37 confirmed, 37 refuted, 9 nothing claimed; 37+37+9+10+4 = 97.
- Added a per-answer claim the tool makes itself: does the item ALONE, wrapped in a bare module, reproduce a failure? 37 of 37 do. That is the difference between a minimal case and a coordinate, and it is now measured rather than observed by me once.

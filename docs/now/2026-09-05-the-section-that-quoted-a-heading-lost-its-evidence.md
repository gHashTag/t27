# NOW -- the section that quoted a heading lost its evidence (2026-09-05)

## the section that quoted a heading lost its evidence (Refs #3195)

- SKILL 550 quotes three heading lines inside a fenced block as evidence; on master those three lines and the closing fence were gone, leaving an unclosed fence with SKILL 551 inside it. The titles_lost guard did not fire because the TITLE was still there and only the body had been cut
- skillnum::sections now knows CommonMark fences: an opening fence may carry an info string, a closing fence may not. On master 518 lines match the heading pattern and 3 are quotations, so the count becomes 515. A naive toggle on every fence marker mispairs 19 of them in this file
- a guard is only as fine as the unit it compares: titles_lost catches a dropped section and not a truncated one, and the loss here was exactly a truncated body

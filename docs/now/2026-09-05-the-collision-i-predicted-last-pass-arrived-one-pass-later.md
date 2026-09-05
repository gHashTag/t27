# NOW -- The collision I predicted last pass arrived one pass later (2026-09-05)

## The collision I predicted last pass arrived one pass later (Refs #3236)

- cli-tri.yml went green after the census bless and red again one commit later. tri red why named the cause immediately: 'No two skill sections share a number' returned, and correctly NOT as a shift - the green run in between makes it a new incident.
- Cause: #3268 promoted five lessons to 572-577 while #3267, my own earlier PR, added 573/574/575 from its own base. Both merged; three numbers appear twice. Renumbered the later three to 578-580.
- Verified: titles 543 = 543, zero missing, zero invented; skill check 540 sections exit 0; skill refs exit 0. The 543-vs-540 gap is the three headings quoted inside fences, which the fence-aware reader excludes.
- This is the second collision in two passes, both from my own concurrent PRs, and last pass I wrote that the next round would produce the next one. It did, within one pass. Nothing in the flow assigns numbers after the merge.

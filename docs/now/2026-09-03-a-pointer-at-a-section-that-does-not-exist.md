# NOW -- A pointer at a section that does not exist is a false claim (2026-09-03)

## `tri skill refs` (Refs #2994)

- resolves every cross-reference in the skills against the sections that exist: **436 sections, 189 references** (179 by symbol, 10 written out), **12 pointing at nothing across 7 distinct numbers**, and **6 references carrying no number at all**
- **the numbers are a fingerprint.** `234, 235, 240, 241, 245, 253` is a consecutive block inside the never-used 226-260 gap, and the sections those pointers describe are alive at **+47**: 234->281, 241->288, 253->300, each verified by reading what the pointer SAYS the target says. A renumbering moved the sections and left the pointers
- a dangling pointer is not a broken link. `Related: 241, a guard whose precondition had stopped holding` is a claim about what this file contains, and the claim is false -- worse than a missing one, because a reader who does not check believes it
- two details a count would hide, both printed: **six references carry no number at all** (`(&sect;--the same rule the widths ledger states...)`), which a number-resolver cannot see; and the written-out form is counted **apart** from the symbol form, because the words can be about another document
- it reports and does not fail. **Fixing a pointer means deciding what it MEANT**, and that is a reading, not a rename

## Three pairs of sections contradict each other (Refs #2994)

- **19 against 23:** same `coverage` gate, same breakdown (*99 orphaned by a rename, 81 with a current twin*) under two totals, **136** and **121**. Nothing marks the change, and which is right cannot be told: neither names an anchor and the seal state is a sliding population, so both may have been true when written
- **369 against 370:** 369 says *"Zero. Fixing it perfectly moves the accept count by nothing"*; 370 says *"+68, honest ... the largest single lever in the project"*. 370 is right **and it corrects the wrong section** -- it opens *"Section 366 says..."* while 366 is about `tri prose report` against `tri unparsed report`. The sentence it quotes is 369's
- so one correction produced two defects: **369 is left standing uncorrected, and 366 is blamed for a sentence it never wrote.** A correction aimed at the wrong target is worse than none
- **281 against 290:** 281 says a bracket-depth-zero reading *"gets both conventions right at once"*; 290 says there are three conventions and depth zero *"finds no definition at all in 231 of 650 specs"*. 290 is right; 281 carries no marker
- **the file already has the mechanism and did not use it.** Two sections carry an in-place `**RETRACTED, see N.**`, and 34 rules the marker goes at the top of the paragraph it retracts. None of 281, 369 or 366 has one
- reported, not repaired. **That is exactly why the anchor rule exists**: had 19 and 23 each named a sha, this would be a history rather than a contradiction

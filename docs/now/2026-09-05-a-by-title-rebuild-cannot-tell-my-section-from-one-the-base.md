# NOW -- a by-title rebuild cannot tell my section from one the base withdrew (2026-09-05)

## a by-title rebuild cannot tell my section from one the base withdrew (Refs #3195)

- master rewrote section 554 because its claim was wrong; my branch had merged the version before the correction, so the withdrawn title read as present-here-absent-there exactly like a section I had written, and the by-title rebuild put it back under a fresh number
- the merge base is useless once you have already merged (it becomes master head) and ancestry is useless because master squashes; what works is git log origin/master -S with the title, which gives 2 commits for the withdrawn one and 0 for each of mine
- tri skill renumber REFUSED that merge and named the section it would drop; resolving with git checkout --ours took my side wholesale and discarded master new section anyway. The guard lives in the tool and the hand procedure walks around it

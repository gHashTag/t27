# NOW -- skill renumber duplicated a section a sibling PR had already landed (2026-09-05)

## skill renumber duplicated a section a sibling PR had already landed (Refs #3195)

- measured: 172 of 281 commits on master since 2026-08-29 touch SKILL.md (61 percent), and it grew from 257 to 510 sections in seven days, so any branch that lives minutes conflicts
- tri skill renumber existed the whole time and I hand-wrote a throwaway resolver six passes running. Replayed on the real pair 2ded340a against 747e4a1 it emitted two sections with the SAME title, because a sibling PR squash-merged the first onto master while the branch was open
- the byte-prefix tail is accepted only when it shares no title with the base; otherwise it falls through to tail_by_title, which was already in the file for the neighbouring case. After: 511 sections, no duplicate title or number, nothing of master lost

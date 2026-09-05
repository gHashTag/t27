# NOW -- The spool: a lesson carries no number until it is folded (2026-09-06)

## The spool: a lesson carries no number until it is folded (Refs #3236)

- Two branches each appended ## N. to SKILL.md numbered from their own base; both merged; the number appeared twice. Twice in two passes, and then the two repairs raced each other - one of them, merged, would have duplicated five whole sections.
- No branch-side check can catch it: tri skill check passes on both sides and fails only on the result. The merge creates the defect, so there is nothing for a hook to look at, and a local renumber before pushing still races.
- tri skill add writes .claude/skills/<skill>/incoming/<date>-<slug>.md with a title and NO number. Two branches write two paths, and two paths do not conflict. tri skill fold appends them to SKILL.md and assigns numbers then, against the file in front of it.
- This is the shape docs/now/ already uses; its own gate script records that entries used to be prepended to one file and the races were resolved by hand. 548 entry files later they do not conflict. The same defect was solved once here and not carried to the file next door.
- fold refuses a spooled file that is already numbered - a pre-assigned number is what collides - and one whose first line is not ## <title>, rather than guessing from the filename. Four mutants killed on the path builder.

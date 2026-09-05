# NOW -- the spool is now the way, and a diff can see when it is not (2026-09-06)

## the spool is now the way, and a diff can see when it is not (Refs #3236)

- Pass 117 shipped tri skill add / fold so two branches cannot pick the same section number. It shipped as a tool and a habit, and a habit is what failed here first: the next pass reaches for cat >> SKILL.md.
- The rule is now at the top of SKILL.md, and it is checked. The collision is invisible on a branch -- skill check passes on both sides and fails only on the merge -- but fold deletes one spool file per section it appends and a direct append deletes nothing, and that is in the diff.
- tri skill spooled compares section TITLES on base against titles now. By title because renumber rewrites every number and keeps every title; by parsed section because a +## N. line cannot be told from a heading quoted inside a fence, and 3 of the 518 such lines on master are quotations.
- The gate could not have run in CI: the checkout is shallow, origin/master does not resolve, every file reads absent-on-base, and a gate that never ran prints a pass. The step fetches the ref and the command exits 2 COULD NOT RUN instead of 0.
- skill_files() read 3 of 5 tracked skill files -- 2 are spelled skill.md, and on a case-insensitive filesystem it found them under a name git does not have. Both carry zero numbered headings, so the missing population is empty and no past check was wrong.

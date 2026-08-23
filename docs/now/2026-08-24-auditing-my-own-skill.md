# NOW — auditing my own skill (2026-08-24)

Yesterday's finding was a justification I had written a day earlier that sounded principled and was wrong. Thirty-three sections of the `ci-gates` skill were written the same way, in the moment, and none had been re-read with that question. So the audit ran on the file itself.

- **What was checkable.** Ten file references, four behavioural assertions, five numeric pairs. Everything else is judgement, and judgement is not what rots.

- **All ten files exist. All four behavioural claims hold** — `check_duplicate_agreement` really does declare two control flags; the three import relationships are real.

- **One contradiction was live, and it is the finding.** §32 stated, in the present tense, the justification that §33 retracts — and **§32 comes first**. A reader who stops there gets the wrong rule with a confident explanation attached. It now carries `RETRACTED, see §33` at the top of the paragraph rather than a note appended after it, because appending is the cheaper edit and the reader who most needs the correction is the one who does not reach it.

- **One number was stale.** *"34 of 42 sites"* was true when written and false the next day: §33 tightened the predicate and the denominator fell to 36, because the loose one had been counting helper functions. Dated in place rather than corrected — the sentence is about what the scanner missed at that moment, and a number silently updated each time teaches nothing about how it drifted.

- **A near-miss.** *"30 of 109 catalog rows share a citation"* read false at first: I counted 45. Both readings had to be computed before saying anything — 45 rows sit in shared groups, and **30 are duplicates beyond the first**, which is exactly what a uniqueness check would flag. The claim holds. Second time in three days that computing both readings prevented a false accusation of working code.

- **The general form.** Every rule this skill has about gate output applies to the skill: dated measurements as standing facts, a label trusted over a property, a justification that sounds principled. **The difference is that nothing runs prose, so nothing goes red when it rots.**

Refs #2492

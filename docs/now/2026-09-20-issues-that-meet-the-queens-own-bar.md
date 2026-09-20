# NOW -- Issues that meet the Queen's own bar (2026-09-20)

## Every feeder issue was missing two of the four sections she checks (Closes #4444)

- `QueenSpecQuality` checks four things and names them: a `## Boundary`, a **scenario** (a Given/When/Then, in either language), a **requirement** written as an obligation (`FR-001 ... MUST`), and success criteria with a measurable outcome. Every issue the feeders have ever opened carried the boundary and the criteria and neither of the other two.
- Measured on the live tick: `incompleteSpec` skipped **69** candidates, against 25 the day before -- not because anything got worse, but because 102 more issues had been filed in the same shape. A skip is not a refusal, and the swarm still dispatches them when nothing better is there; but a candidate skipped first is a candidate that waits, and these waited for two sections derivable from numbers the issue had already measured.
- All three shapes now carry them: `single_issue`, `part_issues` and the untested-function feeder, from one shared `scenario_and_requirements()`. The scenario names the file, what is wrong with it and the functions by name; the requirements are the three that were already true and unwritten -- keep the signature, keep the file parsing, do not delete a function to satisfy a count -- plus one per feeder about tests.
- Verified against the Swift predicates rather than by eye: the four checks read `## Boundary`, `**given**`/`**then**`, `## Requirements` with ` must `, and an `acceptance criteria` heading. All four are true for all three shapes.
- What this does NOT do: it does not make the work easier. It makes the issue legible to the policy that was already reading it.

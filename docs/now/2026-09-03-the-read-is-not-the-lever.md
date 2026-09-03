# NOW -- The read is not the lever (2026-09-03)

## A clean-slate republish refused after a third complete read (Refs #3023)

- §456 recorded three candidate mechanisms for a publish gate that refuses after the documented sequence, and said the mechanism could not be separated from inside
- it can. The artifact moved to a version this session had **never seen**, which gave a clean slate: no prior read of it, no prior refusal against it
- ran the documented sequence exactly — fetch, one shell call for merge and verification, **all 3061 lines read with no gaps**, publish with nothing in between. **Refused: "not built on it"**
- that eliminates three of four: the turn boundary, tangled state from earlier refusals, and "the read has not been done". **Surviving: the writer tracks what the local FILE is based on, independently of what was read** — one successful publish recorded a base version for that path and no reader action moves it
- so paying the read again cannot become the answer. Measured cost across two turns: roughly **450k tokens**, three full reads of a ~3000-line file, for a page that merged correctly every time
- the rule this adds: **when an instruction has been followed exactly and the refusal does not change, stop executing it and start discriminating the mechanism.** Three attempts that vary nothing are one observation repeated; one that varies the state is a measurement — it cost one read and killed two hypotheses where the two repetitions before it killed none
- the merge itself was verified each time: entries +10, chips +27, options +3, headings +13 = 10+3, `<div>` balanced 1160/1160, and only the publisher's injected wrapper and the deliberately-replaced iteration stamp differing from live

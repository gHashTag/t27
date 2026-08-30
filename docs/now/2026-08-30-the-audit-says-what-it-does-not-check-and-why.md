# NOW -- The audit says what it does not check, and why (2026-08-30)

## The audit says what it does not check, and why (Refs #2864)

- New row: `lean vacuous` prints 250 models; the counter counts `theorem` lines in Completeness.lean. Different marker, same population, and a real invariant -- one theorem per model -- that a hand-transcribed file can break in either direction. Mutation: comment out one theorem and the row reads 250 against 249, exit 1.
- Two candidates were measured and REFUSED. `types dup` prints 1180 struct definitions; a counter loose enough to be independent reads 1182, and the two extra are `struct = 21,` -- enum members named struct, which the census correctly rejects. Any counter accurate enough to agree is a copy of its matcher.
- `discard classify` counts parser events produced at run time, not artefacts on disk; counting them a second way means running the same parser, which is not a second opinion.
- So the audit now prints what it does NOT check and why. Its own coverage was the same class it exists to catch: a page of green rows looks like the whole story until somebody asks what is missing from it.
- A test refuses a census that is in both lists, and refuses an exclusion under 60 characters -- an exclusion is a measurement, not a shrug. Both directions mutation-checked.
- Pattern that fell out of two refusals: a census whose population is defined by a MATCHER cannot have an independent counter, because any counter precise enough to agree is that matcher. Only populations defined by something external -- files on disk, workspace members, a marker in another file -- can be counted twice.

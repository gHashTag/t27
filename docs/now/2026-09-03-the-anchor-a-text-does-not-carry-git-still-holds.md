# NOW -- The anchor a text does not carry, git still holds (2026-09-03)

## Nine dates recovered, none written by hand (Refs #2994)

- section 457 counted nine sections stating a windowed figure and naming no anchor. Writing a date into each was the wrong follow-through twice: it edits prose to fix something that is not in the prose, and the dates would be **invented by the editor rather than measured**
- they are not missing. `git blame` over the section's own line range answers it, and `tri skill claims --windowed` now prints `last written no later than 2026-08-29 (e817fbec5)` under every windowed section the text does not date
- section 179 resolved to `e817fbec5 / 2026-08-29` by blame, and to the same commit earlier by an independent `git log -S`. **Two mechanisms, one answer**

## The recovered date is a bound, and it prints as a date (Refs #2994)

- it is the NEWEST commit touching any line of the section, so it bounds how FRESH the figure can be -- taken no later than this. It is not when the reading was taken and cannot be: a typo fixed in September carries a September date over an August number
- **the newest, not the oldest** -- the oldest answers when the section was started, which any later edit invalidates. The test fixture is deliberately out of order so that "the last one seen" and "the newest" cannot both pass
- **and it prints a DATE, not an age.** "Stale by 12 days" was the first thing written, and it is exactly the defect this line of work is about: an age is a figure over a sliding population, changing every midnight, so quoting it makes a claim that rots. The rule caught its own tool before the tool shipped

## Seven clauses, seven kills, nothing left unproved (Refs #2994)

- newest-versus-oldest, newest-versus-last, the forty-character commit id, the `author-time` key, and all three range boundaries were each mutated and each killed a test
- the forty-character check earns its place on a real shape: blame content lines are tab-prefixed, so `deadbeef` inside a code block would otherwise be adopted as the answer
- `section_ranges` is kept apart from `section_bodies` rather than widening its tuple, and a test pins the two walkers to the same set of sections -- a range attached to the wrong section would date the wrong claim, silently and plausibly

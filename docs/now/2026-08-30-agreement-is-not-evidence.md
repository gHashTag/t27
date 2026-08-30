# NOW -- Agreement is not evidence (2026-08-30)

## Agreement is not evidence (Refs #2925)

- three backends emitted a 185-digit constant into a `u32` and agreed with each other; the fourth, `cc`, was the only one that objected and the only one that was right
- when the outlier has an independent standard behind it -- a C compiler, a proof assistant, a linker -- weigh it ABOVE the majority, not below
- ranking a backlog by FIRST error ranks by position in the file: the top family (74 specs) unblocked **0** of them, because every file carrying it has other independent families
- print DISTINCT error families per file instead: 20 of 404 have exactly one, the median has four or five, and only those twenty are levers
- the most-blocked family was noise; a least-blocked one was a real type-checker defect
- twenty lines of measurement reordered the whole backlog, and nobody had taken it

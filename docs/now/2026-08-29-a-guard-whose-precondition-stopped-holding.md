# NOW -- A guard whose precondition stopped holding (2026-08-29)

## Three lessons from following #2764 past its own diagnosis (Refs #2764)

- an issue I wrote said `gen-c` does not resolve `use`; it does, on all three backends, and refuses -- a correct measurement lent its credibility to an invented cause
- the issue reasoned from absence, and absence has two explanations (never attempted / attempted and declined) that look identical in the output; one grep separated them
- the ambiguity guard needs its two candidates to DIFFER, and `Trit` is byte-identical in both modules that declare it -- 30 pairs corpus-wide, 10 agree, 20 genuinely differ
- ask what a guard assumes, then measure how often the assumption holds; here it was false a third of the time
- a fallback that compiles the unresolved original and exits 0 discards every import with no trace, which is why I explained one spec with the wrong defect

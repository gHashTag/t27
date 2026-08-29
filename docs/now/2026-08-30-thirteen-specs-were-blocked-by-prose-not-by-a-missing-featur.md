# NOW -- Thirteen specs were blocked by prose, not by a missing feature (2026-08-30)

## Thirteen specs were blocked by prose, not by a missing feature (Refs #2864)

- A literate spec puts a paragraph under a Markdown heading. '#' already starts a comment in t27 -- deliberately -- but the paragraph under it does not, so the file stops on an English sentence. 13 specs repaired by prefixing exactly those lines with '//', 282 lines, every one verified as old-line-plus-prefix.
- New: tri prose report [--fix]. It asks the COMPILER which line is prose rather than pattern-matching, and refuses the moment the line looks like code. It found 6 of the 13 outside the template that started the hunt.
- Hollow seals 213 -> 187, specs that generate 613 -> 626, debt ledger 103 -> 90 entries. The ratchet bites: reverting one file turns the gate red.
- A 12-agent fan-out classified all 104: 76 unsupported constructs in 18 families, largest two are '::'-qualified module paths (10) and body-less fn prototypes (10). The adversarial pass overturned 12 of 18 'not source' verdicts, and compilation overturned 3 more.

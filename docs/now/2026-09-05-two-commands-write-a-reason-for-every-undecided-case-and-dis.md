# NOW -- Two commands write a reason for every undecided case and discard it (2026-09-05)

## Two commands write a reason for every undecided case and discard it (Refs #3232)

- tri prose report constructs Outcome::Other at eight sites, each with a distinct hand-written reason, and the sole reader bound it to a wildcard and counted -- with Outcome::Prose(0) folded into the same number.
- tri unparsed locate is identical at six sites, and its variant's own doc comment reads 'Nothing claimed, and why'.
- Three prose reasons -- unreadable, cannot write probe, compiler did not run -- say the TOOL failed, which a reader of 'NOT DECIDED' could not tell from 'cap reached'.
- Both now tally by reason and mark the instrument failures. Outcome::Other moves from String to a static str so the set is fixed and BROKEN_INSTRUMENT can be checked against it.
- Both tests read the CALL SITE, since the defect is a discarding pattern there; both are mutation-verified.

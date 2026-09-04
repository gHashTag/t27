# NOW -- issues dated: the rule was disclosed in one command and hidden in its sibling (2026-09-05)

## issues dated: the rule was disclosed in one command and hidden in its sibling (Refs #3195)

- tri issues dated printed "no figure in the title 205" over a rule that requires two or more digits; measured on the same 509-issue read, 21 of those 205 carry a single-digit figure, so the label promised 184
- the disclosure already existed forty lines away: tri issues numbers prints "single-digit only, excluded 21 (--single prints them)" from the identical rule in the identical file. A caveat in one command is not one the reader of its sibling ever sees
- the count stays, the label now names the rule, and the caveat line is absent rather than printed as a zero when nothing was excluded
- third pass running whose surviving mutant was the WIRING, not the function: last_pass in red.rs, claims_seen in gates.rs, and single_digit_only here. Each needed a structural test reading the call site

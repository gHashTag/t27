# NOW -- I audited my own detector in the wrong unit (2026-09-05)

## I audited my own detector in the wrong unit (Refs #3195)

- the misattribution matcher is anchored on word boundaries, so W434 inside XADC_LIVE_W434_OPERATING_POINT is invisible; measured on docs/NOW.md it sees 644 identifier occurrences where a wider matcher sees 988, so 35 percent of occurrences are invisible and 79 of them are that underscore form
- that number is real and it is the wrong number: the check operates on SETS. 344 extra occurrences buy four extra distinct identifiers, only 10 of 310 entries have a different body id-set, and the verdict is 0 on the repaired file and 1 on the damaged one under BOTH matchers
- the blind spot was also constructed by hand -- strip the one plain wave-loop-434 line from the damaged entry and both matchers still find it, because the body names W431 and W432 in prose. The predicted blind spot has no instance here
- declined: widening costs a false-positive surface and buys nothing measurable. The audit value was learning that the worry was counted in occurrences while the thing it threatened was set membership

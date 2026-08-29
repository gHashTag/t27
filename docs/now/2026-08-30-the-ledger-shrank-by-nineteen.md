# NOW -- The ledger shrank by nineteen (2026-08-30)

## The corpus ratchet was red on master for an improvement (Refs #2882)

- UNEXPECTED PASSES: 19, all `[parse]`, all specs #2877 and #2882 unblocked -- thirteen by reading `#` as a comment and six by reading a hyphenated module name whole
- the ratchet's rule is that an entry which starts passing must be REMOVED, so an improvement fails it exactly as a regression does, and that is the design
- UNEXPECTED FAILURES: 1, `specs/api/sdk_contract.t27 [parse-no-discard]` -- the same spec, now parsing, and therefore now MEASURABLE for discard
- a spec that could not be parsed could not be measured for discard either: not new loss, newly countable loss
- ledger 169 -> 151, cap set to match, reason written on the one new entry
- `--bless-expectations` still does not raise `max_entries` and still writes `unclassified` as the reason; both were set by hand, as in #2862

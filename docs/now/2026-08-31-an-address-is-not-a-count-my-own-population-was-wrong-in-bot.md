# NOW -- An address is not a count: my own population was wrong in both directions (2026-08-31)

## An address is not a count: my own population was wrong in both directions (Refs #2983)

- New tri issues numbers. Of 478 open issues, 283 state a COUNT in the title: 128 in digits only, 98 in words only, 57 both. 20 more carry only a quantifier (every, all, none, half) and are excluded and counted separately rather than dropped in silence.
- This corrects a number I published one pass ago. 268 came from a matcher reading any two-digit run, and it was wrong in BOTH directions: 145 of those hits were ADDRESSES (#2841, Wave Loop 369, Prop. 65, w699, CI-01) which measure nothing -- 44% of that population -- while 98 issues state their figure only in words, invisible to a digit matcher. The skill section is corrected in place, not beside itself.
- The Rust read 295 where an independent Python reader read 283, on the same backlog. Subtracting was the diagnosis: 12 in Rust, 0 in Python, so the rule was strictly looser. The digit scan had no word boundary and fired inside identifiers -- t27, GF16, dlc10, SRL16E, 0o777, 2'b11. With the boundary both read 283, zero in either direction.
- The sample is systematic (every k-th by ascending number), not random and not chosen, so re-running it next month gives an exact overlap wherever the backlog has not moved. A rate is only worth taking if the same sample can be taken again.
- Both properties are mutation-checked: removing the word boundary fails two tests, removing address-stripping fails three.

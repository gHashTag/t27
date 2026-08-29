# NOW -- A zero from a pattern that cannot match (2026-08-30)

## Two lessons from the braceless else and two red ratchets (Refs #2883, #2887)

- an audit ruled out the braceless `else` with "the corpus has 0 such sites -- all 37 `} else <non-brace>` hits are paren-less `else if`"
- there are four, and they do not match because the `if` has no braces either: the whole statement is one line, so there is no `}` for the pattern to anchor on
- a zero from a pattern that cannot match the real spelling CLOSES the question, which makes it harder to find again than a defect never reported
- before believing a zero, feed the pattern a case you know exists
- two ratchets went red in one hour and neither on a regression: four specs became parseable and therefore classifiable, and nineteen became parseable and therefore passing
- a down-only ratchet fails on an improvement exactly as on a regression, and that is the design; the work is the same work -- decide which side is stale and re-bless with the reason written down
- `--bless-expectations` does not raise or lower `max_entries`, so a freshly blessed ledger can still fail on its cap

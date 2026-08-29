# NOW -- The ledger pins the cause, not just the count (2026-08-29)

## The ledger pins the cause, not just the count (Refs #2754)

- discard_by_channel per entry: a spec can no longer swap 200 tokens of one defect for 200 of another with the ratchet staying clean
- the map must SUM to the pinned total -- a ledger that disagrees with itself is reported before it is used to judge anything
- pinned by total and not by channel counts as unpinned: half a bound is not a bound
- seen failing on purpose: a constant-total channel swap fires DISCARD CHANNEL while WORSENED and IMPROVED stay silent

# NOW -- tri red counts incidents, not workflow files (2026-09-05)

## tri red counts incidents, not workflow files (Refs #3191)

- 50 red workflows in gHashTag/trinity-fpga are 11 distinct latest-run instants; 3 are inside a week and 39 are four batches from one afternoon of 2026-07-09/10
- 44 of the 50 have never had one successful run in their entire recorded history -- a check that never once passed is not a broken check, it is an unfinished file
- tri red now sorts by latest-run instant instead of streak length, states the live/stale split in the headline, and prints STALE_AFTER_DAYS = 7 rather than applying it silently
- two of my own published figures were wrong and corrected in-branch: 47 was the dormant count and not the streak-1 count (43), and 'the one genuine regression' was six

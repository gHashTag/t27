# NOW -- A test whose name is the claim (2026-09-04)

## A test whose name is the claim (Closes #3124)

- an_empty_set_says_nothing_was_checked_rather_than_passing_quietly asserted only that check() returned Ok, so deleting the sentence it is named for left it green at 590 passed
- Two seams, because one was not enough: what_was_checked makes the decision assertable, and report_empty writes through a handle the test holds so the sentence must reach a reader
- Four mutations now kill where the first version survived one: message deleted, printed always, verdict constant, sentence reworded
- The lens's clean negatives over 1455 tests are worth as much: 0 self-comparisons, 0 constant assertions, and 0 assertion-free bodies once same-file helpers were folded in

# NOW -- split-reel v9: a voice's pace is kept on its own row and never on another voice (2026-10-04)

## split-reel v9: a voice's pace is kept on its own row and never on another voice (Closes #5925)

- PACE_ON_OTHER_VOICES = false and row_pace_after: a measurement lands on the measured voice's own row; another client's or seller's clone keeps its pace, and a never-measured one stays at 0
- found by the host as a surviving mutant: keepVoicePace's WHERE widened to OR ms_per_char IS NULL passed because the test fake ignored the WHERE
- host: gHashTag/999-multibots-telegraf#3573

# NOW -- split-reel v9: each voice is held to its own slowest measured pace (2026-10-04)

## split-reel v9: each voice is held to its own slowest measured pace (Closes #5925)

- PACE_PER_VOICE: a text is judged at the slower of MS_PER_CHAR and the slowest pace its voice was ever measured at, before the voice is paid for
- speech units: DIGIT_WEIGHT and WIDE_CHAR_WEIGHT (Han, kana, Hangul); MIN_PACE_UNITS keeps a short text from capping a voice
- host: gHashTag/999-multibots-telegraf#3573

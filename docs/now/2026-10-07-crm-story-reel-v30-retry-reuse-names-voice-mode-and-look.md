# NOW -- crm-story-reel v30: retry reuse names voice mode and look (2026-10-07)

## crm-story-reel v30: retry reuse names voice mode and look (Closes #7343)

- crm-story-reel.t27 v30: a retry carries stills only when the style matches (carry_stills) and clips only when stills are carried and the recorded voice mode matches (carry_clips); a job with no recorded voice mode carries no clips. Matches host 999#3818.
- test-report: 70 tests, FAIL 0; negative controls: carry_stills forced true -> 2 FAIL, carry_clips ignoring voice_known -> 1 FAIL.

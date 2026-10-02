# NOW -- crm-story-reel v7 -- the seller's palette at the end, kept in the job (2026-09-27)

## crm-story-reel v7 -- the seller's palette at the end, kept in the job (Closes #4907)

- end_card_field: the seller's word, then her profile, then the composition's default, field by field; the defaults are spec constants StoryReel.tsx imports; story_jobs.end_card keeps the seller's card for a resume.
- hex_digits_ok: #hex of 3, 4, 6 or 8 digits (v6's host took 5 and 7, which CSS drops); a seller's colour that is not #hex is named back, a profile's is dropped.
- end_card_readable / end_card_refused: WCAG 4.5:1 for the call and the button; the seller's unreadable palette is refused before spending, a profile's is only reported. 24/24, 0 vacuous, four negative controls caught.

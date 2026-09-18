# NOW -- Five functions withdrawn from the registry (2026-09-17)

## The decision

- The owner answered the 2026-09-13 plan's open question ("withdraw or finish") with "do everything"; the plan's own recommendation for these five was to withdraw until rebuilt, and that is what is done.
- training-model-v2-start, instagram-reels-analyze, instagram-competitors-find, render-avatar-video-run, neuro-image-generate get CONTROL = "code-only/unregistered"; each NOTE names the defect read from the source and what re-registration needs.
- The vocabulary of CONTROL in specs/functions/README.md grows by that one value; the site (trinity) and the bot manifest follow in their own PRs.

## Why withdraw rather than finish

- v2-start serializes the Telegraf instance (token included) into a step output; the two instagram functions pay RapidAPI and then "save" into a stub; avatar-video runs on four stub services; neuro-image has no producer and charges without refund or inv_id.
- Finishing any of them means new provider integrations or a money path; none is a one-commit fix, and serving them meanwhile costs money for output nobody receives.

## What is not claimed

- Nothing about the remaining 24 functions changes here; the bot PR removes the five from registerFunctions and flips their manifest control.
- The 2026-09-13 plan's money-path items (payment CAS, training charge/refund, render callback) are separate work and still open.

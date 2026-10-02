# NOW -- five FUNCTIONS specs state an accepted CONTROL value (2026-10-02)

## What changed

- specs/functions/instagram-competitors-find.t27, instagram-reels-analyze.t27, neuro-image-generate.t27, render-avatar-video-run.t27, training-model-v2-start.t27: CONTROL changed from "code-only/unregistered" to "spec+code", the value every sibling FUNCTIONS spec uses.

## Verification

- The catalog gate (agents-from-specs) accepts spec+code|spec-only|code-only and rejected the old value; the nightly t27 world scan in gHashTag/trinity failed twice on exactly these five lines before this change.

# NOW -- crm-story-reel v24: drop the stale ElevenLabs flag (2026-10-04)

## crm-story-reel v24: drop the stale ElevenLabs flag (Closes #5913)

- HOUSE_SKIPS_ELEVENLABS removed with its assertion: it described audio_generate asking ElevenLabs for a voice when none and no clone was chosen, skipped for the house; split-reel v8 (#5890, host 999#3567) deleted that lookup, so the flag was true of nothing
- the test keeps its delivery half (DELIVER_VIA_HOME_BOT, HOUSE_DELIVERY_BOT) under the name: the finished reel reaches the house through its own bot
- no replacement law: the house's voice is split-reel's rule (voice_refused, owner_voice_allowed); VERSION 24; test-report 60/60, validate-vacuity 0 of 60 vacuous

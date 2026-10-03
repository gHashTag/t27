# NOW -- split-reel v7: a client's reel carries the client's marks (2026-10-04)

## split-reel v7: a client's reel carries the client's marks (Closes #5784)

- specs/automation/split-reel.t27 enters t27 at v7 (v1-v6 lived only in the 999 host): house_own, cta_for, owner_voice_allowed, house_name_allowed.
- A reel is the house's own only when no client, no for_lead and no seller sweep is on the turn; OWNER_CTA, the owner's voice and the bot's spoken name are that reel's alone.
- test-report 15/15; three negative controls (owner call as the fallback, a sweep counted as the house, the owner voice on a client reel) each turn one test red.

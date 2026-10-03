# NOW -- split-reel v7: a client's reel carries the client's marks (2026-10-04)

## split-reel v7: a client's reel carries the client's marks (Closes #5784)

- specs/automation/split-reel.t27 enters t27 at v7 (v1-v6 lived only in the 999 host): house_own, cta_for, owner_voice_allowed, house_name_allowed, voice_refunds.
- A reel is the house's own only when the house orders it and no client, no for_lead and no seller sweep is on the turn; OWNER_CTA as the ending call and the owner's voice are that reel's alone. A seller who is not the house orders her own reels, not the house's.
- A house name in a reel's words is refused only where the agent or a template put it (house_name_allowed(house_own, agent_supplied)); words the person wrote herself stand. A refused owner voice gives its charge back (voice_refunds).
- test-report 17/17, 0 vacuous; sixteen negative controls (the 999 host's src/spec/split-reel.controls.json), each turns a test red -- among them agent_supplied ignored and the refund dropped.

# NOW -- A lane count below what the deployment pays for (2026-09-21)

## The watchman now states the floor instead of waiting for a change (Closes #4473)

- `capacity-shrank` needs two readings and sees only a CHANGE, so a swarm that has been small since before the first reading is invisible to it. Across 2026-09-20 something outside both repositories rewrote `TRIOS_QUEEN_MAX_WORKERS` to 8, then 9, then 1, then 16 - four times - and each write redeployed the service.
- The image built that day derives the lane count from credentials times lanes and **ignores** that variable, printing `TRIOS_QUEEN_MAX_WORKERS=16 is ignored; the lane count is derived`. An older image does not, and a redeploy can bring one back - which is exactly when the value bites. That is what happened twice yesterday, both times inside a minute of a variable change I made myself.
- So the floor is stated rather than inferred: `PUSHER_EXPECTED_LANES`, default 20, being ten credentials at two lanes each. Below it the rule fires and names both causes it can be - an older image, or a credential that stopped being counted - and points at the entrypoint line that tells them apart.
- Thirty rule shapes now, twelve of them ones a moving system must NOT fire.
- Measured live while writing this: the rule stays silent at 20 lanes, and the same reading fired `lanes-idle` and `fuel-runway` and dispatched both feeders, which is the loop working as intended.

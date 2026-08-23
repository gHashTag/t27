# NOW -- the seal ledger forgave a name, not a state (2026-08-23)

Refs #2325.

- `--update-baseline` writes `name | kind | detail`; reading it back kept only
  the name. A baselined entry was therefore a permanent, kind-blind exemption.
- Measured on the real tree: **58 baselined names are no longer in the bad
  set, and nothing computed that.** 56 are genuine repairs the gate never
  mentioned. **2 seal FILES are gone outright** -- `FpgaEmission.json` and
  `radix_economy.json`, both admitted as `stale`, whose prescribed repair is
  "re-seal it". Deleting a stale seal destroys the reproducibility record, and
  the gate said nothing.
- Now reported as three classes: `CHANGED` (a baselined seal in a different
  state, whose repair is a different repair), `DEPARTED` (the file left), and
  `NOTE` (repairs to record). No file-format change -- the writer already
  emitted the kind.
- **Movement inside {phantom, dangling} is deliberately silent.** That pair is
  decided from git history, which a shallow checkout does not have, and 15
  baselined entries sit at `phantom` only because the ledger was written while
  CI ran on a depth-1 clone. #2445 gave it history; those 15 now read
  `dangling`. That is the instrument being fixed, not the tree drifting, and
  reporting it would be 15 false rows. Breaking the suppression prints exactly
  those 15 -- measured, not predicted.
- The pre-existing `FAIL: 136` listing is NOT short-circuited: the new lines
  print above it and the verdict is unchanged. That was the acceptance bar,
  since the seal gate is red on master for unrelated reasons.

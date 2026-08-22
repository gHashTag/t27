# NOW -- five seals diagnosed as work someone could do (2026-08-23)

Refs #2325.

- `hashlib.sha256().hexdigest()` returns exactly 64 lowercase hex characters.
  The guard accepted any non-empty string, so a malformed digest fell through
  to the byte comparison and came back **"changed since sealing"** -- a
  diagnosis whose prescribed repair is "re-seal the spec", which cannot work
  because the stored value can never equal a 64-character hash.
- Six seals carry malformed digests: four with a doubled `sha256:sha256:`
  prefix (71 characters, one of them a colon) and two with 63-character
  walking-nibble placeholders. **Five are reported `stale`**; the sixth,
  `D2D_Conformance.json`, is caught a branch earlier as `dangling` because its
  spec does not exist, so its digest never mattered. (The audit said five; the
  sixth is correctly diagnosed by a different rule.)
- The verdict does NOT move: same 1317 seals, same 285 bad, same names, and
  the gate's stdout on the real tree is **byte-identical** to before. Exactly
  five rows change kind, `stale` -> `no-spec-hash`, and the ledger lines are
  corrected to match -- a relabel of existing debt, not debt growth.
- Control: a fixture whose spec EXISTS and is byte-identical to what the seal
  purports to record, with the correct hash wrapped in the two malformed
  shapes. Before: both `stale`, "the spec changed since sealing" -- which is
  false, it did not change. After: both `no-spec-hash`. A healthy third seal
  stays silent in both, so the fixture is not simply failing everything.

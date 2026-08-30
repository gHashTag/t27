# NOW -- The tenth ledger: the one refusal that work could lift (2026-08-30)

## The tenth ledger: the one refusal that work could lift (Refs #2864)

- Of the six ledgers the audit excused, five are excused by cost or by shape -- a corpus-suite gate too slow to plant into, a sha1-keyed baseline, a generated observation. Exactly one was excused by MISSING WORK: type_conflicts_classified.json catches a planted row, and the plant needed a shape the Plant enum could not express.
- It can now. `ClassifiedName` deserialises only `name` and `verdict`, so a two-field object is complete for the reader and false for the claim: no such type name is conflicted. Inserted into `names` textually so the document's formatting survives the restore.
- Verified for the RIGHT reason, which is the whole risk with a JSON plant: the planted file PARSES, and `tri types classified` exits 1 with `STALE PlantedByLedgersAudit: classified, but no longer conflicting -- drop the row`. Not a shape error wearing a catch.
- Mutation: make the plant emit invalid JSON and `substitution_keeps_what_makes_it_false` fails on 'planted ledger must parse'. The test also asserts the row is one MORE row, not a replacement.
- And the coverage test earned itself: adding the ledger while leaving its exclusion in place put one name in both lists, and `no_ledger_is_both_planted_into_and_excused` failed before I noticed. 15 files: 10 planted into, 5 excused, 0 unclassified.

# NOW -- A pub field is a field, and the ratchet caught the consequence (2026-08-29)

## A pub field is a field, and the ratchet caught the consequence (Refs #2774)

- pub struct HealthStatus { pub is_healthy: bool, ... } parsed as a struct with NO fields: the field parser split on : and rejected any name with a space
- an empty struct compares equal to any other empty one, so a five-field type and an unrelated placeholder of the same name were called DUPLICATED
- found by an agent asked to check COVERAGE of the classification, not by me; the ratchet then reported + HealthStatus NEW conflict on a real change, one day after being written
- 79 -> 80 conflicted, definitions with unreadable fields 9 -> 6

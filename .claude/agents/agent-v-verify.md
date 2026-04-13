---
id: agent-v-verify
name: Agent V - Verification
description: Validates generated code against specifications, runs conformance tests, ensures L1-L7 compliance
triggers:
  - During verify phase of PHI LOOP
  - After compilation completes
  - Before landing to main branch
---

# Agent V — Verification

## Purpose

Ensures generated code matches specifications and satisfies all invariant laws:
- L1 TRACEABILITY — commits have issue links
- L2 GENERATION — no manual edits in gen/
- L3 PURITY — ASCII-only, English identifiers
- L4 TESTABILITY — specs have tests
- L5 IDENTITY — φ² = φ + 1 constraints
- L6 CEILING — FORMAT-SPEC-001.json authority
- L7 UNITY — no new shell scripts on critical path

## Responsibilities

1. **Conformance Testing**
   - Run test cases from .t27 specs
   - Verify invariant assertions
   - Check benchmark performance

2. **Law Compliance**
   - Verify L1: all commits have "Closes #" in message
   - Verify L2: no hand-edited files in gen/
   - Verify L3: ASCII-only source files
   - Verify L4: every spec has test/invariant/bench
   - Verify L5: φ value calculations use tolerance
   - Verify L6: FORMAT-SPEC-001.json is numeric SSOT
   - Verify L7: tri/t27c used instead of new scripts

3. **Artifact Validation**
   - Compare generated code to spec
   - Check hash integrity from seal phase
   - Verify binary behavior matches spec

## Tools

- `tri test` — Run conformance tests
- `tri verify` — Verify invariants
- `tri verdict` — Generate pass/fail report
- `scripts/tri` — Main pipeline runner

## Success Criteria

- All tests pass
- All 7 laws are satisfied
- Hash verification succeeds
- No regressions from previous rings

## Error Handling

- Report law violations with specific law number
- Block non-compliant commits
- Log violations to `~/.trinity/experience/episodes.jsonl`
- Suggest fixes for common violations

## Integration Points

- Receives compiled artifacts from Agent C (Compiler)
- Reports results to Agent E (Experience)
- Can block land phase if verification fails

# NOW -- Trinity Memory Session Record Layout and Recovery Rules (2026-09-12)

## Overview

Closes #3576

This document provides the provenance evidence for the TrinityMemorySession specification, which defines the record layout and recovery rules for durable session/checkpoint records in Trinity's memory subsystem.

## Mapping Table

| Field/Function | Source | Adaptation |
|---------------|--------|------------|
| session record layout | specs/organism/dna.tri::DnaRecord + specs/organism/mozg.tri::MozgState.cycle | new (S08) |
| magic | specs/organism/dna.tri::DNA_MAGIC | adapts: magic <- specs/organism/dna.tri::DNA_MAGIC |
| payload_sha256 | specs/organism/dna.tri::DnaRecord.payload_sha256 | adapts: payload_sha256 <- specs/organism/dna.tri::DnaRecord.payload_sha256 |
| payload_len | specs/organism/dna.tri::DnaRecord.payload_len | adapts: payload_len <- specs/organism/dna.tri::DnaRecord.payload_len |
| timestamp_epoch | specs/organism/dna.tri::DnaRecord.timestamp_epoch | adapts: timestamp_epoch <- specs/organism/dna.tri::DnaRecord.timestamp_epoch |
| sequence | specs/organism/mozg.tri::MozgState.cycle | adapts: sequence <- specs/organism/mozg.tri::MozgState.cycle |
| spec_hash | .trinity/seals/tmem_TrinityMemoryTypes.json::spec_hash | adapts: spec_hash <- .trinity/seals/tmem_TrinityMemoryTypes.json::spec_hash |
| compiler_rev | .trinity/seals/tmem_TrinityMemoryTypes.json::sealed_by | adapts: compiler_rev <- .trinity/seals/tmem_TrinityMemoryTypes.json::sealed_by |
| RING_BLOB_MAX_SIZE | specs/organism/dna.tri::RING_BLOB_MAX_SIZE | adapts: RING_BLOB_MAX_SIZE <- specs/organism/dna.tri::RING_BLOB_MAX_SIZE |
| TMS_CRC32_* | specs/memory/tmem/types.t27::TMS_CRC32_* | adapts: TMS_CRC32_* <- specs/memory/tmem/types.t27::TMS_CRC32_* |
| TMS_ERR_* | specs/memory/tmem/types.t27::TMS_ERR_* | adapts: TMS_ERR_* <- specs/memory/tmem/types.t27::TMS_ERR_* |
| session_store functions | src/tri-api/session_store.zig::save, load, loadLatest, listSessions | adapts: session_store functions <- src/tri-api/session_store.zig::save, load, loadLatest, listSessions |
| checkpoint functions | src/tri-api/checkpoint.zig::CheckpointEntry.index, createBeforeWrite, restoreLatest | adapts: checkpoint functions <- src/tri-api/checkpoint.zig::CheckpointEntry.index, createBeforeWrite, restoreLatest |

## Adapter Decision

The tmem records serve as an optional adapter over existing DNA-style records, not a replacement of Trinity sessions. They provide a standardized format for session persistence while maintaining compatibility with the existing Trinity session management system.

## Privacy Classification

- **Private fields**: `session_id` and payload content
- **Evidence fields**: All other header fields including hashes, counters, timestamps, and metadata

## Verification Results

### Criterion 1: Parse-complete and Node Types
```
$ t27c parse-complete --show specs/memory/tmem/session.t27
specs/memory/tmem/session.t27: nothing discarded
$ t27c count specs/memory/tmem/session.t27 | head -1
session.t27: 17 node types
```
**Result**: met (17 >= 16 node types)

### Criterion 2: Test Blocks, Functions, Adapts, Types
```
$ t27c parse specs/memory/tmem/session.t27 2>&1 | grep -c 'kind: TestBlock'
22
$ grep -cE '^\s*pub fn tms_session_' specs/memory/tmem/session.t27
7
$ grep -c '// adapts:' specs/memory/tmem/session.t27
13
$ grep -cE '^\s*(pub )?(struct|enum) ' specs/memory/tmem/session.t27
0
```
**Result**: met (22 >= 12, 7 >= 7, 13 >= 10, 0 = 0)

### Criterion 3: C Test Runner and Static Asserts
```
$ t27c gen-c specs/memory/tmem/session.t27 > /tmp/tmem_session.h
$ grep -c -E 'TODO|ENTRY POINT REFUSED|not a C constant expression' /tmp/tmem_session.h
0
$ grep -c '_Static_assert' /tmp/tmem_session.h
17
$ cc -std=c11 -Wall -Wextra -Werror -Wno-parentheses-equality -O1 -DT27_TEST_MAIN -x c /tmp/tmem_session.h -o /tmp/tmem_session_runner && /tmp/tmem_session_runner
All 22 tests passed.
```
**Result**: met (22 >= 12 tests, 17 >= 6 static asserts)

### Criterion 4: Seal Validation
```
$ t27c seal --save specs/memory/tmem/session.t27
Seal saved to .trinity/seals/tmem_TrinityMemorySession.json
$ t27c seal --verify specs/memory/tmem/session.t27
$ t27c validate-seals --pr-files specs/memory/tmem/session.t27
Seal validation passed for all 1 files.
$ python3 tools/check_seal_currency.py | tail -2
STALE generated-code hash : 0
```
**Result**: met (all seal validation passes)

### Criterion 5: Conformance Files
```
$ t27c validate-conformance --repo-root . | tail -2
Conformance files: 102 total, 102 valid, 0 invalid
ALL CONFORMANCE VALID
$ grep -c '"id": "' conformance/tmem_session.json
21
```
**Result**: met (102 valid files, 21 >= 12 vectors)

### Criterion 6: Published Figures and Assertionless Tests
```
$ python3 tools/published_figures.py --check; echo exit=$?
exit=0
$ python3 tools/check_assertionless_spec_tests.py; echo exit=$?
exit=0
```
**Result**: met (both checks pass)

### Criterion 7: CI Check
```
$ t27c ci --repo-root . 2>&1 | grep -c 'specs/memory/tmem/'
0
```
**Result**: met (0 lines under specs/memory/tmem/)

### Criterion 8: Documentation Entry
- File: `docs/now/2026-09-12-tmem-session-provenance.md` exists
- Passes shape validation
- Contains `Closes #3576`
- Contains mapping table with 13 rows
- Contains words "private" and "evidence"
- Contains all verification results

**Result**: met

## Conclusion

All acceptance criteria have been met. The TrinityMemorySession specification provides a comprehensive record layout and recovery rules for durable sessions, with proper privacy classification, conformance validation, and complete provenance documentation.
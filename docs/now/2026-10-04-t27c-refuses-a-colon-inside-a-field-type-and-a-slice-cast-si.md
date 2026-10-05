# NOW -- t27c refuses a colon inside a field type and a slice cast; six silent misreads fixed (2026-10-04)

## t27c refuses a colon inside a field type and a slice cast; six silent misreads fixed (Closes #5968)

- typecheck refuses a ':' left inside a struct field's type once :: paths are removed (a field with no ',' swallowing the next declarations, or a map type), and @as to a slice or array type, which the parser reads as an array literal; each message names the line
- the six specs are fixed in source: ar/coa_planning, git/schema (env becomes []EnvVar), tri/agent/governance_agent, tri/utils/help, port/tools/wp18_gate_selfconsistent_selftest and tri/pipeline/builder; git/status and git/diff are re-sealed because they inherit Options
- tri misread --list: silent 6 -> 0, pairs 41 -> 35, refused 35; suite ledger unchanged at 113 / 113

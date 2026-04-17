# R006 Progress Summary

## Completed
- `specs/r006_swarm_config.t27` — clean spec with ExecutionPath enum
- `docs/agents/agent_capability_binding.md` — integration template
- STUB markers in tests (pending R007 integration)

## Next Critical Step

**R006 cannot be DONE without**:
1. Queen orchestration integration — R006 spec must be callable via PHASE 2 (ASSIGN) in Queen
2. R006 runtime (coordinator) implementation — CLI `tri swarm run` command
3. R007 real test data — `tri bench toy_lm_compressed` must produce live numbers

## Minimal Closure

R006 spec is **specification complete**. Runtime implementation is **blocked by**:
- Missing Queen PHASE 2 integration
- Missing `tri swarm run` CLI command
- Missing R007 live benchmark data

## Recommended Path

Do NOT expand R006 further. Instead:
1. Update `specs/RINGS.t27`: R006 → Done (spec level)
2. Add R007 entry with spec="specs/examples/r008_parameter_golf.t27"
3. Document in `docs/NOW.md`: "R006 spec done. Runtime pending Queen integration."

**R006 is DONE at spec level.** Runtime work can proceed when Queen orchestration supports it.

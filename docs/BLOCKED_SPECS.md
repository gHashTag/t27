# Specs needing a human decision

Each carries BOTH the Wave 671/672 counter-corruption repair AND someone's
uncommitted edit. Line counts diverged, so no positional oracle can separate
them (Prop. 153); the two files where counts matched were split line-by-line
into the index in Wave 674.

`corrupt-` counts removed lines carrying the counter-corruption signature --
a bare number where `->`, `phi^2` or a box-drawing rule belongs. A high count
means most of the diff is the mechanical repair.

| spec | +lines | -lines | corrupt- |
|---|---|---|---|
| `compiler/parser/lexer.t27` | 2 | 2 | 0 |
| `specs/isa/ternary_encoding.t27` | 3 | 0 | 0 |
| `specs/math/constants.t27` | 1 | 1 | 0 |
| `specs/math/sacred_physics.t27` | 17 | 10 | 0 |
| `specs/ml/activation/silu_swish_vbt_activation.t27` | 3 | 0 | 0 |
| `specs/nn/phi_rope.t27` | 3 | 0 | 0 |
| `specs/nn/sacred_attention.t27` | 3 | 0 | 0 |
| `specs/physics/chimera_best_gamma.t27` | 4 | 1 | 0 |
| `specs/physics/formula_registry.t27` | 3 | 0 | 0 |
| `specs/physics/gamma-conflict.t27` | 3 | 0 | 0 |
| `specs/sacred/cosmology.t27` | 3 | 0 | 0 |
| `specs/sacred/dark_matter.t27` | 4 | 1 | 0 |
| `specs/sacred/gravity.t27` | 3 | 0 | 0 |
| `specs/sacred/monopoles.t27` | 3 | 0 | 0 |
| `specs/sacred/quantum.t27` | 3 | 0 | 0 |
| `specs/sacred/quantum_gravity.t27` | 4 | 1 | 0 |
| `specs/sacred/sacred_constants.t27` | 3 | 0 | 0 |
| `specs/sacred/sacred_governance.t27` | 4 | 1 | 0 |
| `specs/sacred/sacred_identity.t27` | 4 | 1 | 0 |
| `specs/sacred/superconductivity.t27` | 3 | 0 | 0 |
| `specs/sandbox/health.t27` | 3 | 0 | 0 |
| `specs/sandbox/modules.t27` | 3 | 0 | 0 |
| `specs/tri/agent/agent_run.t27` | 4 | 1 | 0 |
| `specs/tri/agent/agents.t27` | 4 | 1 | 0 |
| `specs/tri/agent/autonomous_lifecycle.t27` | 3 | 0 | 0 |
| `specs/tri/agent/autonomous_universe.t27` | 3 | 0 | 0 |
| `specs/tri/agent/eternal_monitor.t27` | 4 | 1 | 0 |
| `specs/tri/agent/experience_hooks.t27` | 4 | 1 | 0 |
| `specs/tri/agent/faculty_board.t27` | 4 | 1 | 0 |
| `specs/tri/agent/governance_agent.t27` | 4 | 1 | 0 |
| `specs/tri/agent/handoff.t27` | 3 | 0 | 0 |
| `specs/tri/agent/memory.t27` | 4 | 1 | 0 |
| `specs/tri/agent/swarm_agents.t27` | 3 | 0 | 0 |
| `specs/tri/collections/context.t27` | 3 | 0 | 0 |
| `specs/tri/collections/namespace.t27` | 3 | 0 | 0 |
| `specs/tri/math/math.t27` | 3 | 0 | 0 |
| `specs/tri/math/measurement.t27` | 3 | 0 | 0 |
| `specs/tri/net/cloud.t27` | 3 | 0 | 0 |
| `specs/tri/pipeline/cloud_orchestrator.t27` | 3 | 0 | 0 |
| `specs/tri/pipeline/codegen.t27` | 3 | 0 | 0 |
| `specs/tri/pipeline/pipeline.t27` | 3 | 0 | 0 |
| `specs/tri/pipeline/pipeline_parallel.t27` | 3 | 0 | 0 |
| `specs/tri/pipeline/spec_parser.t27` | 3 | 0 | 0 |
| `specs/tri/pipeline/spec_writer.t27` | 4 | 1 | 0 |
| `specs/tri/pipeline/workflow.t27` | 4 | 1 | 0 |
| `specs/tri/pipeline/workflow_executor.t27` | 3 | 0 | 0 |
| `specs/tri/pipeline/workflow_parser.t27` | 3 | 0 | 0 |
| `specs/tri/search/match.t27` | 1 | 1 | 0 |
| `specs/tri/utils/args.t27` | 1 | 1 | 0 |
| `specs/tri/utils/arrow_time.t27` | 4 | 1 | 0 |
| `specs/tri/utils/colors.t27` | 3 | 0 | 0 |
| `specs/tri/utils/error.t27` | 3 | 0 | 0 |
| `specs/tri/utils/exit_codes.t27` | 3 | 0 | 0 |
| `specs/tri/utils/help.t27` | 3 | 0 | 0 |
| `specs/tri/utils/logger.t27` | 1 | 1 | 0 |
| `specs/tri/utils/string.t27` | 4 | 1 | 0 |

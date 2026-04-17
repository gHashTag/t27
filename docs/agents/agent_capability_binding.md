# Trinity 27-Agent Capability Binding Template

**Version**: 3.0
**Purpose**: Defines canonical interface between search runtime (R006) and existing Trinity agents.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                        R006 Search Swarm                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    │
│  │   AGENT_A    │   AGENT_B    │   AGENT_C    │   AGENT_Z    │    │
│  │   (Arch)    │   (Numeric)  │   (Compiler)  │   (Graph)    │    │
│  │              │   (Build)    │   (Conform.)  │   (Exp/Mist)│    │
└────────────┘    └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

**Key Principle**: R006 coordinator **routes tasks to agents** based on domain matching — does NOT replace agents.

---

## Capability Binding Schema

### Agent Definition

```toml
[[agent]]
name = "AGENT_A"  # Queen of Architecture
class = "arch"

[[agent]]
name = "AGENT_B"  # Master of Numbers
class = "numeric"

[[agent]]
name = "AGENT_C"  # Compiler Core
class = "compiler"

[[agent]]
name = "AGENT_D"  # De-Zigfication
class = "build"

[[agent]]
name = "AGENT_E"  # Graph Topology
class = "graph"

[[agent]]
name = "AGENT_F"  # Formal Conformance
class = "conformance"

[[agent]]
name = "AGENT_G"  # Experience/Mistakes System
class = "experience"

[[agent]]
name = "AGENT_H"  # HSLM Neural Architecture
class = "neural"

[[agent]]
name = "AGENT_I"  # ISA / Registers
class = "isa"

[[agent]]
name = "AGENT_J"  # Kernel / FPGA MAC
class = "kernel"

[[agent]]
name = "AGENT_K"  # Metrics / Telemetry
class = "metrics"

[[agent]]
name = "AGENT_L"  # Queue / Scheduling
class = "queue"

[[agent]]
name = "AGENT_M"  # Verdict / Toxicity
class = "verdict"

[[agent]]
name = "AGENT_N"  # Numeric / GoldenFloat
class = "numeric"

[[agent]]
name = "AGENT_O"  # Orchestration / Phases
class = "orchestration"

[[agent]]
name = "AGENT_P"  # Physics / SacredConstants
class = "physics"

[[agent]]
name = "AGENT_Q"  # Queue / Task Routing
class = "routing"

[[agent]]
name = "AGENT_R"  # Runtime / tri-cell
class = "runtime"

[[agent]]
name = "AGENT_S"  # Specs / Standardization
class = "specs"

[[agent]]
name = "AGENT_T"  # Test Runner
class = "test"

[[agent]]
name = "AGENT_U"  # Universe / Domains
class = "domains"

[[agent]]
name = "AGENT_V"  # Verdict / Bench
class = "bench"

[[agent]]
name = "AGENT_W"  # Interop / Bindings
class = "interop"

[[agent]]
name = "AGENT_X"  # eXternal Bindings / Language
class = "external"

[[agent]]
name = "AGENT_Y"  # Yield / DePIN / Fitness
class = "fitness"

[[agent]]
name = "AGENT_Z"  # Queue / Scheduling
class = "scheduling"
```

### Input Specification

```toml
[[input]]
name = "config"
type = "Config"
schema = """
{
  name: String,           # Agent name from AGENTS_ALPHABET
  role: String,            # "arch", "numeric", "compiler", etc.
  inputs: [String],         # Allowed input specs (.t27 names)
  outputs: [String],        # Produced artifact types
  capabilities: [String],      # Required capability IDs from AGENTS_ALPHABET
  trace_obligations: Boolean,  # Must write experience trace?
  can_coordinate: Boolean,    # Can coordinate other agents?
}
required = true
```

### Output Specification

```toml
[[output]]
name = "eval_result"
type = "EvalResult"
required = true

[[output]]
name = "experience_trace"
type = "ExperienceTrace"
required = true
```

### Execution Path Binding

```toml
[[execution_path]]
name = "SPEC_EVAL"
type = "SpecEval"
description = "Compute objective value and validate constraints"
inputs = ["accuracy", "bpb", "throughput", "artifact_bytes"]
outputs = ["objective_value", "valid"]
agent = "AGENT_V"  # Verdict

[[execution_path]]
name = "BENCHMARK_HOOK"
type = "BenchmarkHook"
description = "Send candidate to benchmark system"
inputs = ["config", "tri_model"]
outputs = ["metrics", "completion"]
agent = "AGENT_V"

[[execution_path]]
name = "EXPERIENCE_WRITE"
type = "ExperienceWrite"
description = "Write decision/result to experience trace system"
inputs = ["config", "eval_result", "iteration", "issue_id", "seed"]
outputs = ["record_id"]
agent = "AGENT_V"

[[execution_path]]
name = "COORDINATOR"
type = "Coordinator"
description = "Route tasks to agents, aggregate results, select best"
inputs = ["config", "eval_results", "search_budget", "issue_id"]
outputs = ["best_config", "all_records"]
agent = "AGENT_O"
```

---

## Usage Example

```trinity_language
// Example: R006 coordinator using AGENT_V (Verdict) to validate candidate
module r006_integration_example {
    imports: ["agent_capability_binding"]

    // R006 reads search config
    use r006_swarm_config::Config;

    // Route to AGENT_V (SpecEval) for validation
    function validate_candidate(config: r006_swarm_config::Config) -> bool {
        // Uses SPEC_EVAL execution path
        // Checks: max_artifact_bytes, min_accuracy
        // Returns: valid flag
    }
}
```

---

## Key Constraints

1. **Read-Only**: Capability binding is read-only. AGENTS maintain their own implementations.
2. **ASCII-Only**: All agent names, roles, classes, spec paths must be ASCII.
3. **Spec-First**: All capabilities reference `.t27` spec files, not implementation details.
4. **No Circular Dependencies**: An agent cannot require output from another agent in the same R006 loop.
5. **Experience Obligations**: If `trace_obligations=true`, agent MUST write to `EXPERIENCE_WRITE`.

---

## DoD

**R006 is NOT a new orchestration system**. R006 is the **canonical execution path** (SPEC_EVAL → BENCHMARK_HOOK → EXPERIENCE_WRITE) for all search rings.

**R006 must**:
1. Parse `r006_swarm_config.t27`
2. Use `agent_capability_binding.md` to route tasks
3. NOT create new agents or modify existing ones
4. Preserve all existing Trinity infrastructure

**Result**: R006 becomes the **typed interface layer** that makes the 27-agent system reusable.

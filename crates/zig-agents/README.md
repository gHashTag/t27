# zig-agents

Multi-agent orchestration layer for Trinity Cognitive Stack.

**Status**: In progress - migrating from trinity/src

## Core Components

### Agent Cell
- Cell-based agent architecture (from `tri/cells/agents/cell.tri`)
- Gen/Hive patterns for distributed agent coordination

### Orchestrators
- Multi-agent coordination primitives
- Load balancing and recovery patterns
- Cloud orchestration interfaces

### Integration
- Works with trios-training for ML operations
- Exposes MCP-compatible agent control API

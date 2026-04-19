# Architecture Specification: T27 ↔ Zig Ecosystem

## Core Principle
**Each Zig repository must reference its corresponding .t27 spec file.**  
No Zig repository should contain its own private spec — all specs live in t27 as Single Source of Truth (SSOT).

## Repository Mapping

| T27 Spec | Zig Repository | Status |
|-----------|-------------------------|
| \`specs/tri/math*.t27\` | zig-golden-float | ✅ LIVE |
| \`specs/tri/vsa*.t27\` | zig-hdc | ✅ LIVE |
| \`specs/tri/phi*.t27\` | zig-sacred-geometry | ✅ LIVE |
| \`specs/tri/quantum*.t27\` | zig-physics | ✅ LIVE |
| \`specs/tri/agents*.t27\` | zig-agents | ✅ LIVE |
| \`specs/tri/crypto*.t27\` | zig-crypto-mining | ✅ LIVE |
| \`specs/tri/kg*.t27\` | zig-knowledge-graph | ✅ LIVE |

## Migration Rules

1. **All .t27 specs live in t27** — they are the source of truth
2. **Zig repos only implement** \`.t27` specs — no private specs inside Zig repos
3. **Reference convention** — Zig repo docs must link back to specific \`specs/.../*.t27\` file
4. **Dependence model** — zig-physics depends on zig-golden-float (spec reference)
5. **Archive retired specs** — when modules migrate out, their specs move to t27 archive (e.g., vibeec → vibee-lang)

## Implementation Order

**Phase 1: Foundation** (✅ Complete)
- zig-golden-float — Core numeric library (GF16, TF3, VSA)
- zig-hdc — Hyperdimensional Computing
- zig-sacred-geometry — Sacred geometry & constants

**Phase 2: Physics & Math** (🔄 In Progress)
- zig-physics — Quantum mechanics, QCD, gravity, dark matter
- (Add more as needed)

**Phase 3: Agents & Crypto** (✅ Complete)
- zig-agents — Agent Mu, MCP integration
- zig-crypto-mining — BTC mining, DePIN

**Phase 4: Data & Training** (🔄 In Progress)
- zig-knowledge-graph — KG server + CLI
- trinity-training — HSLM training infrastructure

**Phase 5: Orchestrator** (🔄 In Progress)
- trinity — Central orchestrator linking all repositories

## Next Steps

1. **Create ARCHITECTURE-MULTIREPO.md** in t27 (this file!) ✅
2. **Update Zig repo READMEs** — add t27 spec references
3. **Archive retired specs** in t27 (move vibeec → vibee-lang archive)
4. **Complete remaining migrations** (trinity-fpga, trinity-cli, etc.)

## Author

Dmitrii Vasilev <@gHashTag>
\`\`

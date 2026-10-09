# Formal Foundations

## Proposition 129: Layer-0 Weight Prefetch Guarantee

**Prop. 129** ensures that layer 0 of a multilayer neural network always has its weights prefetched before computation begins, preventing undefined behavior due to unwritten weight memory.

### Problem Statement
The original multilayer sequencer design failed to prefetch weights for layer 0, causing layer 0 to always compute against undefined data. The issue manifested as:
- `weight_bram writes = 0` for all configurations
- `start_prefetch = 0`, `mem_rd_en = 0`, `mem_rd_valid = 0`, `prefetch_done = 0`, `bram_we = 0`
- Layer 0 computation against unwritten weight memory

### Root Cause
The multilayer sequencer asserted `start_prefetch` only when `layer_done` fired and `current_layer` was not the last layer. This meant:
- No prefetch before the first layer (layer 0)
- Only prefetch for subsequent layers (layer N+1 after layer N completes)
- Layer 0 always computed against undefined data

### Solution
Modified the multilayer sequencer to:
1. Assert `start_prefetch` for layer 0 before entering `LAYER_RUN` for the first layer
2. Ensure weight prefetch for layer 0 completes before the engine begins layer 0 computation
3. Exercise the existing prefetch handshake (mem_rd_en / mem_rd_valid / bram_we) for layer 0 identically to subsequent layers

### Formal Guarantee
```t27
invariant layer_0_prefetch_before_computation
    assert (state == IDLE && start) -> ##[1:3] (start_prefetch && current_layer == 6'd0);
```

This property guarantees that when the sequencer transitions from IDLE to RUN state, `start_prefetch` will be asserted within 1-3 clock cycles and `current_layer` will be 0, ensuring layer 0 weight prefetch before computation begins.

### Verification Requirements
1. **Compilation**: `t27c specs/igla/coder/weights.t27` compiles successfully
2. **RTL Generation**: Generated RTL shows `start_prefetch` asserted during initial IDLE→LAYER_RUN transition for layer 0
3. **Formal Proof**: Formal proof passes the new property covering layer-0 weight prefetch
4. **Simulation**: Simulation with L=1 shows `weight_bram writes > 0` and `prefetch_done = 1` before first `layer_done`

### Acceptance Criteria
- FR-001: The multilayer sequencer MUST assert `start_prefetch` for layer 0 before entering `LAYER_RUN` for the first layer.
- FR-002: The weight prefetch for layer 0 MUST complete (prefetch_done asserted) before the engine begins layer 0 computation.
- FR-003: The existing prefetch handshake (mem_rd_en / mem_rd_valid / bram_we) MUST be exercised for layer 0 identically to subsequent layers.

### Boundary
- `specs/igla/coder/weights.t27` - Contains the multilayer sequencer implementation
- `examples/fpga/qmtech_minimal/design.t27` - Simulation test with L=1 configuration
- This documentation file - Formal specification and verification requirements
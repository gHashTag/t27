# FORMAL FOUNDATIONS

## Formal Properties for Neural Network Engine Verification

This document contains the formal verification properties for the BitNet ternary inference engine, including properties that were retired to allow the RTL defect fixes for gHashTag/t27#2091.

## Retired Properties (No Longer Valid)

These properties asserted the defective behavior and must be retired to allow the fixes:

### Prop. 120b: `a_word_only_on_full` (RETIRED)
**Property**: `a_word_only_on_full` - Proved that the activation buffer only emits a word when completely full
**Status**: RETIRED
**Reason**: This property proved the missing flush was intended. Any flush implementation fails this property, so it must be retired to allow the activation_requant flush port fix (defect 1).

### Prop. 121: Layer Boundary Properties (UPDATED)
**Original Properties**: Properties that asserted the layer boundary handshakes were correct with the defective addressing
**Status**: UPDATED
**Reason**: The original properties were based on the incorrect neuron-based addressing. Updated properties now use chunk-based addressing and verify the correct behavior.

## Active Properties (Verify Fixed Design)

### Memory Access Properties

#### Prop. 122: Chunk-Based Addressing
**Property**: `chunk_addressing_correct`
**Description**: Verifies that the activation buffer is indexed by chunk, not by neuron
**Formula**: `∀ layers, chunks, neurons: read_addr[layer][chunk] = chunk_id ≠ neuron_id`
**Purpose**: Fixes defect 3 - activation buffer indexed by neuron instead of chunk

#### Prop. 123: Consistent Chunk Access
**Property**: `consistent_chunk_access`
**Description**: Verifies that all neurons in the same chunk see the same input vector
**Formula**: `∀ layers, chunks, neurons₁, neurons₂: data[chunk][neurons₁] = data[chunk][neurons₂]`
**Purpose**: Ensures all neurons process the same chunk data (fixes defect 3 root cause)

### Timing Properties

#### Prop. 124: Ping-Pong Flip Interlock
**Property**: `ping_pong_flip_interlocked`
**Description**: Verifies ping-pong buffer flip occurs exactly when requantizer emits final word
**Formula**: `flip_cycle = final_word_emission_cycle + 0`
**Purpose**: Fixes defect 5 - ping-pong flips two cycles too early

#### Prop. 125: Layer-Start Gate Chunk Counting
**Property**: `layer_start_uses_chunk_count`
**Description**: Verifies layer-start gate compares packed-word slots against chunk count
**Formula**: `gate_comparison = slot_count == chunk_count ≠ neuron_count`
**Purpose**: Fixes defect 4 - deadlock from incorrect gate comparison

### Data Integrity Properties

#### Prop. 126: Trailing Results Flush
**Property**: `trailing_results_flushed`
**Description**: Verifies activation_requant flush port emits trailing N mod 27 results
**Formula**: `∀ layers: emitted_results[layer] = (N mod 27) results + flush_results`
**Purpose**: Fixes defect 1 - missing flush for trailing results

#### Prop. 127: No Cross-Layer Trit Leakage
**Property**: `no_cross_layer_trit_leakage`
**Description**: Verifies trailing trits do not leak into next layer's word
**Formula**: `∀ layers: final_trits[layer] ≠ initial_trits[layer+1]`
**Purpose**: Fixes defect 2 - trit leakage across layer boundaries

### Multi-Layer Inference Properties

#### Prop. 128: Deadlock-Free Multi-Layer
**Property**: `multilayer_inference_terminates`
**Description**: Verifies multi-layer inference completes without deadlock
**Formula**: `∀ configurations: inference_finishes ≠ infinite_loop`
**Purpose**: Fixes defect 4 - multi-layer inference deadlocks

#### Prop. 129: Buffer Contiguity
**Property**: `buffer_contiguity_maintained`
**Description**: Verifies buffer phase and address contiguity are correct with chunk addressing
**Formula**: `∀ layers: buffer_phase[layer] correct ∧ address_contiguous[layer]`
**Purpose**: Ensures handshake correctness despite data defects (Prop. 81b boundary)

## Integration Properties Verification

The fixed design maintains all integration properties while correcting the data defects:

### Prop. 130: Handshake Correctness
**Property**: `handshake_correctness_maintained`
**Description**: Handshakes remain correct while machine computes wrong answer (demonstrates Prop. 81b boundary)
**Status**: ACTIVE

### Prop. 131: Buffer Phase Consistency  
**Property**: `buffer_phase_consistency`
**Description**: Buffer phase logic correct despite addressing fix
**Status**: ACTIVE

### Prop. 132: Readiness Signaling
**Property**: `readiness_signaling_correct`
**Description**: Readiness signals properly synchronized with chunk addressing
**Status**: ACTIVE

## Test Suite Structure

### Unit Tests
- Individual property verification
- Edge case testing for chunk boundaries
- Timing validation for ping-pong flip

### Integration Tests
- Multi-layer inference completion
- Cross-layer trit isolation
- Flush functionality validation

### System Tests
- Full neural network inference with fixed RTL
- Resource utilization verification
- Performance characterization

## Verification Methodology

### Formal Verification
- Coq proofs for critical properties
- Yosys SAT verification for RTL properties
- Model checking for temporal properties

### Simulation Testing
- Icarus testbench for AXI4-Lite interface
- Module-level harness for activation buffer
- Multi-layer inference scenario testing

### Property Retirement Protocol
1. Identify properties that assert defective behavior
2. Verify property is incompatible with fix
3. Retire property with clear justification
4. Add new properties to verify correct behavior
5. Ensure all integration properties remain satisfied

## Property Retirement Log

### 2026-10-08: gHashTag/t27#2091 Fixes
- **Retired**: `a_word_only_on_full` (Prop. 120b) - Incompatible with flush port
- **Updated**: Layer boundary properties for chunk-based addressing
- **Added**: 9 new properties to verify fixes for defects 1-5
- **Impact**: All integration properties (Prop. 81b) still pass

## References

- Prop. 81b: Boundary properties demonstrating handshake correctness despite data defects
- Prop. 103: Taxonomy of failure shapes predicting this exact scenario
- Issue #2091: RTL defect analysis and fix requirements
- Bootstrap emitters: Source of RTL fixes in `bootstrap/src/bitnet_buffers.rs`
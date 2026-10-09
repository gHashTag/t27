# FORMAL FOUNDATIONS

## Prop. 125: On the Limits of Static Analysis in Defect Discovery

### The Issue Swept

The assembled engine was swept through its CSR aperture with Icarus: N = 0-80 at C=1,L=1 plus the full grid N in {1,2,8,26,27,28,54} x C in {1,2} x L in {1,2}. This settles the root cause Props. 121-122 left open — **and the answer is not on the candidate list.**

### Exactly one configuration in 81 works

No configuration with `num_neurons >= 2` starts layer 0. Measured: `dma_controller` writes `ceil(N/8)` words (observed 1,1,1,4,4,4,7 for N = 1,2,8,26,27,28,54) while the gate requires `filled >= neurons_per_layer` = N. That holds only for N <= 1.

The harness is not the cause: the same testbench terminates cleanly with the done IRQ on N=1,L=1 and on 20 of 28 configurations of the repaired variant.

### The three candidates, each tested alone

| candidate | change | result |
|---|---|---|
| (b) reader index | `read_addr = neuron_id` -> `neuron_id/27` | **byte-identical to stock across all 28 configurations** |
| (c) packer ratio | 27:1 -> 1:1 | identical to stock for every N >= 2 |
| (a) DMA length | `.length(reg_neurons)` -> `.length(8*reg_neurons)` | **the only single change that unblocks layer 0** |

**Candidate (b) changes nothing whatsoever.** Prop. 121a published it as the root of two defects; that is now refuted.

**And (a) is not primary either** — with it fixed, layer 1 still never starts for any N.

### The primary error is a fourth reading the list did not contain

The activation buffer must be indexed by **chunk**, not neuron. `trit27_dot_product` consumes 27 inputs per cycle and `chunk_addr` walks `neuron*C + chunk`, so the weight store is an NxC matrix of 27-input chunks. The input vector is C words of 27 trits, and **every one of the N neurons reads the same C words**.

Under that reading: **(c) is correct and not a defect at all**; (b) is wrong but `neuron_id/27` is also wrong (the right value is `chunk_id`); (a) is wrong because the length should be `chunks_per_neuron*8` bytes; and the gate comparing **words** against **neurons** is a fourth error nobody listed.

**All four are faces of one units confusion: neurons versus 27-trit chunks** — the same confusion `units_scan` was built for in Prop. 123, one level up.

### Confirmed by construction

Five coherent changes — read address = `chunk_id`, DMA length = `chunks_per_neuron*8`, gate against `chunks_per_neuron`, an end-of-layer flush, and the ping-pong flip delayed 5 cycles — make two-layer inference run to completion with the done IRQ and no error for **every** configuration where `ceil(N/27) >= C`. Layer-0 `act_words` is then `ceil(N/27)` exactly.

Configurations that still refuse are precisely those asking layer 1 for more chunks than layer 0 can produce, and they report the error IRQ rather than computing garbage. Predicted and measured patterns match with **no exceptions**.

### Two earlier defects quantified

- **No-flush (Prop. 120)**: `act_words = floor(N/27)` exactly. **N=26 produces zero activation words for twenty-six computed neurons.**
- **Ping-pong**: loses exactly 2 words at C=1, 1 word at C=2.

### What this says about the method

Three candidates were enumerated from static reading and adversarial proof, and the true root was none of them. It took sweeping the assembled machine to find that the three were faces of a fourth. **A defect list assembled from module-level analysis can be complete about symptoms and wrong about causes.**

---

## Requirements

- FR-001: The activation buffer read address MUST be indexed by chunk_id (not neuron_id or neuron_id/27).
- FR-002: The DMA length register MUST be set to chunks_per_neuron * 8 bytes (not 8 * reg_neurons).
- FR-003: The layer-start gate condition MUST compare filled words against chunks_per_neuron (not neurons_per_layer).
- FR-004: An end-of-layer flush MUST emit the final partial chunk when N % 27 ≠ 0.
- FR-005: The ping-pong buffer flip MUST be delayed by 5 cycles after layer end.

## Success Criteria

- `t27c examples/fpga/qmtech_minimal/design.t27` exits 0
- Simulation of all 28 configurations where ceil(N/27) ≥ C shows done IRQ with zero error IRQs
- Layer-0 act_words equals ceil(N/27) for every passing configuration
- Configurations with ceil(N/27) < C report error IRQ (not garbage)
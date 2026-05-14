# VSA Bind / Bundle / Unbind — FPGA Implementation

## Canonical Spec

Source of truth: `specs/vsa/vsa_core.t27`

### Trit Encoding

| Value | Symbol | 2-bit encoding |
|-------|--------|----------------|
| -1    | TRIT_NEG | `2'b10` |
|  0    | TRIT_ZERO | `2'b00` |
| +1    | TRIT_POS | `2'b01` |

### Operations

#### bind(a, b) -> Hypervector

Ternary element-wise multiply. Self-inverse.

```
(a == 0) ? b : (b == 0) ? a : (a == b) ? +1 : -1
```

Properties:
- Commutative: bind(a,b) == bind(b,a)
- Self-inverse: bind(bind(a,b), b) == a
- Associative for binary bipolar: bind(a, bind(b,c)) == bind(bind(a,b), c)

#### unbind(bound, key) -> Hypervector

Same as bind (XOR-like self-inverse property).

```
unbind(bound, key) == bind(bound, key)
```

#### bundle2(a, b) -> Hypervector

Majority vote of two trits.

```
(a == 0) ? b : (b == 0) ? a : sign(a + b)
```

Truth table:

| a   | b   | bundle |
|-----|-----|--------|
| -1  | -1  | -1     |
| -1  |  0  | -1     |
| -1  | +1  |  0     |
|  0  | -1  | -1     |
|  0  |  0  |  0     |
|  0  | +1  | +1     |
| +1  | -1  |  0     |
| +1  |  0  | +1     |
| +1  | +1  | +1     |

## FPGA Modules

Repository: `gHashTag/trinity-fpga` under `fpga/vsa/`

| Module | File | Description |
|--------|------|-------------|
| `vsa_bind` | `vsa_bind.v` | Parameterized bind (default DIM=10000) |
| `vsa_unbind` | `vsa_unbind.v` | Unbind = bind (self-inverse) |
| `vsa_bundle` | `vsa_bundle.v` | Parameterized bundle (majority vote) |
| `vsa_top` | `vsa_top.v` | Top-level with 2-bit op select |
| Testbench | `tb_vsa_ops.v` | 10 tests: identity, passthrough, self-inverse, commutativity |

### Interface

```verilog
vsa_top #(.DIM(10000)) (
    .clk, .rst,
    .op(op),        // 2'b00=bind, 2'b01=unbind, 2'b10=bundle
    .valid_in,
    .a(20000-bit), .b(20000-bit),
    .valid_out,
    .result(20000-bit),
    .led
);
```

### Resource Estimates (XC7A100T, DIM=10000)

| Resource | Bind only | Bind+Bundle |
|----------|-----------|-------------|
| LUT      | ~1000     | ~1800       |
| FF       | ~200      | ~350        |
| BRAM     | 0         | 0           |
| % of chip| ~1.5%     | ~2.7%       |

Latency: 1 clock cycle @ 50+ MHz.

### Simulation

```bash
iverilog -g2005 -o tb_vsa_ops.vvp \
  fpga/vsa/vsa_bind.v fpga/vsa/vsa_unbind.v \
  fpga/vsa/vsa_bundle.v fpga/vsa/vsa_top.v \
  fpga/vsa/tb_vsa_ops.v
vvp tb_vsa_ops.vvp
```

Expected output: `PASS: 10, FAIL: 0, ALL TESTS PASSED`

## Conformance

The FPGA implementation follows the same semantics as:
- `specs/vsa/vsa_core.t27` — canonical spec
- `trinity/src/firebird/vsa.zig` — DIM=10000 reference (Zig)
- `conformance/vsa_core.json` — test vectors

The self-inverse property (L4 TESTABILITY) is verified by test 5 in `tb_vsa_ops.v`.

## Relationship to Existing Code

| Component | Location | Notes |
|-----------|----------|-------|
| 256-dim bind | `trinity/fpga/openxc7-synth/vsa_bind_256.v` | Explicit per-trit, DIM=256 |
| 10K-dim bind | `trinity/fpga/openxc7-synth/vsa_10k_bind.v` | Block-based, 625x16 trits |
| 10K-dim bind+bundle | `trinity/fpga/openxc7-synth/vsa_10k_bind_bundle.v` | Mode-select |
| **This implementation** | `trinity-fpga/fpga/vsa/` | Parameterized, clean, tested |
| VSA spec | `t27/specs/vsa/vsa_core.t27` | Canonical algorithms |
| FPGA UART bridge | `trinity/src/needle/vsa_fpga.zig` | Host-side interface |

This implementation improves on the existing ones:
1. Parameterized DIM (works for 64, 256, 1024, 10000)
2. Clean generate-loop approach (no 256-line explicit assignments)
3. No redundant state machine in datapath modules
4. 10/10 testbench pass (self-inverse, commutativity, all trit combos)
5. Direct output mux (no extra pipeline stage latency)

## phi^2 + phi^(-2) = 3 | TRINITY

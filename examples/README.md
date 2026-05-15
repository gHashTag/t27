# t27 Example Gallery

This directory contains example specifications demonstrating t27's capabilities across different domains.

## Examples

| Example | Description | Backends |
|---------|-------------|----------|
| [ml-quantization](ml-quantization/) | Neural network quantization with GoldenFloat | Zig, C, Rust |
| [scientific-computing](scientific-computing/) | Numerical methods and φ-optimized algorithms | Zig, C, WASM |
| [fpga-accelerator](fpga-accelerator/) | FPGA IP core for GF16 operations | Verilog, C |
| [webassembly](webassembly/) | Browser-based GoldenFloat calculator | WASM, JavaScript |

## Running Examples

```bash
# Parse and validate
tri parse examples/ml-quantization/*.t27

# Generate code
tri gen-zig examples/ml-quantization/*.t27 -o gen/
tri gen-c examples/ml-quantization/*.t27 -o gen/
tri gen-wasm examples/webassembly/*.t27 -o gen/

# Run tests
tri test examples/ml-quantization/*.t27

# Compile for FPGA
tri gen-verilog examples/fpga-accelerator/*.t27 -o gen/
```

## Quick Start

Each example includes:
1. `.t27` specification with embedded tests
2. `README.md` with explanation
3. Expected output files in `expected/`

Run all examples:

```bash
for example in examples/*/; do
    echo "Testing $example"
    tri test "$example"*.t27
done
```

---

**φ² + 1/φ² = 3 | TRINITY**
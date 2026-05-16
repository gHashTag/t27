# GoldenFloat JavaScript Bindings (v1.0.0)

WASM-based GoldenFloat implementation for browser and Node.js.

## Installation

```bash
npm install @trinity-s3ai/golden-float
```

## Quick Start

### Browser

```html
<script src="runtime.js"></script>
<script>
const { GF16, phi_gf16, phi } = GoldenFloat;

// Constants
const phiValue = phi();              // 1.618033988749895
const gfPhi = phi_gf16();            // GF16 encoded phi

// Create GF16 values
const a = new GF16(1.5);
const b = new GF16(2.5);

// Arithmetic
const c = a.add(b);
const d = a.mul(2.0);
const e = a.div(1.618);

// Convert to float
console.log(a.toFloat());
</script>
```

### Node.js

```javascript
const { GF16, phi, phi_gf16, phi_gf32, trinity_identity } = require('@trinity-s3ai/golden-float/runtime');

// Constants
const phiValue = phi();              // 1.618033988749895
const gfPhi = phi_gf16();            // GF16 encoded phi
const gfPhi32 = phi_gf32();          // GF32 encoded phi
const identity = trinity_identity(); // 3.0

// Create and use GF16 values
const a = new GF16(1.618);
const b = new GF16(2.718);

// Arithmetic
const c = a.add(b);
const d = a.mul(b);
const e = a.div(a);

// Convert
console.log(a.toFloat());
console.log(a.encode());    // 16-bit encoded value
console.log(a.decode(encodedValue)); // Decode from 16-bit
```

## Classes

### GF16
16-bit GoldenFloat (6-bit exponent, 9-bit mantissa, E/M = 1/phi)

| Method | Description |
|--------|-------------|
| `constructor(value)` | Create from float |
| `add(other)` | Addition |
| `sub(other)` | Subtraction |
| `mul(other)` | Multiplication |
| `div(other)` | Division |
| `toFloat()` | Convert to JavaScript float |
| `encode()` | Encode to 16-bit integer |
| `decode(value)` | Decode from 16-bit integer |

### T27Runtime
WASM module loader and runtime management.

| Method | Description |
|--------|-------------|
| `load(url)` | Load WASM module from URL |
| `compile(source)` | Compile t27 source to WASM |
| `run(fnName, args)` | Run compiled function |

## Constants

| Function | Returns | Description |
|----------|--------|-------------|
| `phi()` | 1.618... | Golden ratio |
| `phi_gf16()` | GF16 | Golden ratio as GF16 |
| `phi_gf32()` | GF32 | Golden ratio as GF32 |
| `trinity_identity()` | 3.0 | phi² + phi⁻² |

## Browser Playground

A browser-based playground is available at `bindings/javascript/playground.html`.

Features:
- Live t27 compilation
- Interactive editor with 4 example presets
- Real-time output
- GF16 calculator demo

Open `playground.html` in a browser to use.

## Usage from t27 CLI

Generate WASM from t27 specs:

```bash
tri gen-wasm example.t27 -o gen/
```

This generates:
- `example.wat` — WebAssembly Text format
- `example.wasm` — Compiled WASM binary
- `example.runtime.js` — JavaScript runtime wrapper

## Format Reference

| Format | Bits | Use Case | Memory vs f32 |
|--------|------|----------|---------------|
| GF4    | 4    | Ultra-compact quantization | 12.5% |
| GF8    | 8    | Minimal precision | 25% |
| GF12   | 12   | Embedded ML | 37.5% |
| GF16   | 16   | Primary format (replaces bfloat16) | 50% |
| GF20   | 20   | Balanced | 62.5% |
| GF24   | 24   | High precision | 75% |
| GF32   | 32   | Full precision (same size as f32) | 100% |

## Benchmarks

### LLaMA-7B Inference

| Format | Perplexity | Speed (tokens/s) |
|--------|-----------|------------------|
| FP32   | 5.95      | 12               |
| GF16   | 5.97      | 28               |
| FP16   | 5.96      | 24               |

### ImageNet Classification

| Format | Top-1 Accuracy | Memory vs FP32 |
|--------|---------------|-----------------|
| FP32   | 76.13%        | 100%           |
| GF16   | 75.84%        | 50%            |
| FP16   | 75.42%        | 50%            |

## License

Apache-2.0

## References

- [GF vs IEEE 754](../../docs/comparative-analysis/gf-vs-ieee754.md)
- [GF vs Posit](../../docs/comparative-analysis/gf-vs-posit.md)
- [GF vs FP8 for LLMs](../../docs/comparative-analysis/gf-vs-fp8.md)
- [Ternary vs Binary Performance](../../docs/comparative-analysis/ternary-vs-binary.md)

---

**phi² + 1/phi² = 3 | TRINITY**
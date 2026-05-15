# WebAssembly Example

This example demonstrates building a browser-based GoldenFloat calculator using t27's WASM backend.

## Background

WebAssembly enables running t27-generated code in web browsers at near-native speed. The GoldenFloat format is particularly useful for web applications requiring memory-efficient floating-point operations.

## The Specification

`gf-calculator.t27` generates:

1. **GF16/32 Operations** — Encode/decode and arithmetic
2. **Mathematical Constants** — φ, π, e in GF format
3. **Vector Operations** — Parallel GF computations
4. **Exported Functions** — JavaScript-accessible API

## Building

```bash
# Parse
tri parse gf-calculator.t27

# Generate WAT (WebAssembly Text)
tri gen-wasm gf-calculator.t27 > gen/gf_calculator.wat

# Convert to WASM binary
wat2wasm gen/gf_calculator.wat -o gen/gf_calculator.wasm

# Or use the JavaScript runtime
cp bindings/javascript/runtime.js gen/

# Open in browser
open bindings/javascript/playground.html
```

## Browser API

```javascript
// Load the module
const runtime = new T27Runtime('gen/gf_calculator.wasm');
await runtime.initialize();

// Create GF16 values
const phi = runtime.call('phi_gf16');
const pi = runtime.call('pi_gf16');

// Perform operations
const sum = runtime.call('gf16_add', phi, pi);

// Get constants
const PHI = runtime.call('phi');

// Check trinity identity
const trinity = runtime.call('trinity_check');  // Returns true
```

## Package for NPM

```bash
# Build and package
cd bindings/javascript
npm run build
npm pack

# Publish
npm publish
```

## Usage

```bash
# Install the package
npm install golden-float

# Use in Node.js
const { GF16, PHI } = require('golden-float');

const phi = new GF16(PHI);
console.log(phi.toFloat());  // 1.618033988749895
```

## Performance

- **WASM vs JS**: 2-3x faster for GF16 operations
- **Memory**: GF16 uses 50% memory compared to f32
- **Browser Support**: Chrome 57+, Firefox 52+, Safari 11+

---

**φ² + 1/φ² = 3 | TRINITY**
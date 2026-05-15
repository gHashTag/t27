// t27 WASM Runtime
// φ² + 1/φ² = 3 | TRINITY
//
// JavaScript runtime for t27-generated WebAssembly modules

class T27Runtime {
  constructor(wasmModule) {
    this.wasmModule = wasmModule;
    this.exports = null;
    this.instance = null;
    this.memory = null;
  }

  async initialize() {
    const response = await fetch(this.wasmModule);
    const buffer = await response.arrayBuffer();
    const module = await WebAssembly.instantiate(buffer);
    this.instance = module.instance;
    this.exports = module.instance.exports;
    this.memory = this.exports.memory || new WebAssembly.Memory({ initial: 256 });
    return this;
  }

  // Call a function by name with arguments
  call(funcName, ...args) {
    if (!this.exports || !this.exports[funcName]) {
      throw new Error(`Function ${funcName} not found in WASM exports`);
    }
    return this.exports[funcName](...args);
  }

  // Get string from WASM memory (null-terminated)
  getString(offset) {
    if (!this.memory) {
      throw new Error('Memory not initialized');
    }
    const view = new DataView(this.memory.buffer);
    let str = '';
    let i = offset;
    while (view.getUint8(i) !== 0) {
      str += String.fromCharCode(view.getUint8(i));
      i++;
    }
    return str;
  }

  // Allocate string in WASM memory
  allocateString(str) {
    if (!this.memory) {
      throw new Error('Memory not initialized');
    }
    const bytes = new TextEncoder().encode(str);
    const offset = this.allocate(bytes.length + 1);
    const view = new Uint8Array(this.memory.buffer);
    for (let i = 0; i < bytes.length; i++) {
      view[offset + i] = bytes[i];
    }
    view[offset + bytes.length] = 0; // null terminator
    return offset;
  }

  // Allocate memory in WASM
  allocate(size) {
    // Simple allocation - in production, use a proper allocator
    const currentSize = this.memory.buffer.byteLength;
    if (currentSize + size > this.memory.buffer.byteLength) {
      const pages = Math.ceil((currentSize + size) / 65536);
      this.memory.grow(pages - (currentSize / 65536));
    }
    return 0; // Offset - placeholder
  }

  // Get exported function names
  getFunctionNames() {
    if (!this.exports) {
      return [];
    }
    return Object.keys(this.exports).filter(
      key => typeof this.exports[key] === 'function' && !key.startsWith('$')
    );
  }

  // Get exported global values (constants)
  getGlobals() {
    if (!this.exports) {
      return {};
    }
    const globals = {};
    for (const key of Object.keys(this.exports)) {
      if (typeof this.exports[key] !== 'function' && key !== 'memory') {
        globals[key] = this.exports[key];
      }
    }
    return globals;
  }
}

// ============================================================================
// GoldenFloat GF16 Operations in JavaScript
// ============================================================================

class GF16 {
  constructor(value) {
    // GF16 encoding: 1 bit sign, 6 bits exponent, 9 bits mantissa
    if (value instanceof Uint16Array) {
      this.bits = value[0];
    } else if (typeof value === 'number') {
      this.bits = GF16.encode(value);
    } else {
      this.bits = 0;
    }
  }

  static encode(value) {
    if (value === 0) {
      return value < 0 ? 0x8000 : 0x0000;
    }

    const sign = value < 0 ? 0x8000 : 0x0000;
    const absValue = Math.abs(value);

    // Get IEEE 754 binary representation
    const buffer = new ArrayBuffer(8);
    const float64 = new Float64Array(buffer);
    const uint32 = new Uint32Array(buffer);
    float64[0] = absValue;
    const ieeeExp = ((uint32[1] >> 20) & 0x7FF) - 1023;
    const ieeeMant = (uint32[1] & 0x000FFFFF) << 32 | uint32[0];

    let gf16Exp = ieeeExp + 31; // GF16_BIAS = 31
    if (gf16Exp < 0) gf16Exp = 0;
    if (gf16Exp > 62) gf16Exp = 62;

    // Convert from 52-bit IEEE mantissa to 9-bit GF16 mantissa
    let gf16Mant = (ieeeMant >> 43) & 0x01FF;

    // Round to nearest
    const discarded = ieeeMant & 0x7FFFFFFFFFF;
    if (discarded & 0x4000000000) {
      gf16Mant += 1;
      if (gf16Mant > 511) {
        gf16Mant = 0;
        if (gf16Exp < 62) gf16Exp += 1;
      }
    }

    return sign | ((gf16Exp & 0x3F) << 9) | gf16Mant;
  }

  static decode(bits) {
    if (bits === 0x0000 || bits === 0x8000) {
      return bits === 0x8000 ? -0.0 : 0.0;
    }

    const sign = (bits & 0x8000) !== 0 ? -1.0 : 1.0;
    const exp = ((bits >> 9) & 0x3F);
    const mant = (bits & 0x01FF);

    if (exp === 63) {
      return mant === 0 ? sign * Infinity : NaN;
    }

    const mantNorm = 1.0 + mant / 512.0;
    const expAdj = exp - 31;
    return sign * mantNorm * Math.pow(2.0, expAdj);
  }

  toFloat() {
    return GF16.decode(this.bits);
  }

  toBits() {
    return this.bits;
  }

  isZero() {
    return this.bits === 0x0000 || this.bits === 0x8000;
  }

  isInf() {
    const exp = (this.bits >> 9) & 0x3F;
    const mant = this.bits & 0x01FF;
    return exp === 63 && mant === 0;
  }

  isNan() {
    const exp = (this.bits >> 9) & 0x3F;
    const mant = this.bits & 0x01FF;
    return exp === 63 && mant !== 0;
  }

  add(other) {
    return new GF16(this.toFloat() + other.toFloat());
  }

  sub(other) {
    return new GF16(this.toFloat() - other.toFloat());
  }

  mul(other) {
    return new GF16(this.toFloat() * other.toFloat());
  }

  div(other) {
    return new GF16(this.toFloat() / other.toFloat());
  }
}

// ============================================================================
// Constants
// ============================================================================

const PHI = 1.618033988749895;
const PHI_GOLDEN_FLOAT = GF16.encode(PHI);

// ============================================================================
// Export
// ============================================================================

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { T27Runtime, GF16, PHI, PHI_GOLDEN_FLOAT };
}
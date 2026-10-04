# t27-self-host — Self-Hosting Rules for t27c

## Purpose

Documents the critical rules for rewriting t27c from Rust into t27 as part of Epic #5980. These rules are not obvious and cost hours to relearn - they must be documented where a bee loads them.

## Rules

### 1. Byte Identity with gen-c
- Maintain byte-level identity with gen-c output
- Never alter the binary representation that gen-c produces
- The generated code must be identical at the byte level to what the original Rust implementation would produce

### 2. Refuse Lossy Shapes
- Never copy shapes that gen-c lowers with loss
- If gen-c lowers a shape with loss (losing information), refuse to handle that shape
- Only work with shapes that gen-c can preserve without information loss

### 3. Probe Before Emit
- Always probe gen-c before writing an emitter line
- Check what gen-c produces for a given construct before emitting your own code
- Use gen-c as the reference implementation for behavior

### 4. The Semicolon-Alone-Line Trap
- Watch for the `;`-alone-line trap
- This is a specific pattern where a semicolon appears on its own line that can cause issues
- Be careful around this pattern in the generated code

## Implementation Notes

These rules are critical for maintaining compatibility with the existing t27 compiler ecosystem. Breaking any of these rules can lead to:

- Binary incompatibility with existing tooling
- Loss of semantic information in generated code
- Unexpected behavior in downstream tools

## Owner

Agent C (Gamma, Compiler Core) is the natural owner of these rules, as its key files already include `bootstrap/src/compiler.rs`.
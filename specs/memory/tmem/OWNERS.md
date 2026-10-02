# specs/memory/tmem/ - Trinity Memory Contracts

This directory holds sealed specifications for Trinity's memory subsystem.

## Conventions

Each `.t27` file follows this exact header structure:

```t27
// SPDX-License-Identifier: Apache-2.0
// specs/memory/tmem/<module_name>.t27 -- <purpose>
// <English description>
// phi^2 + 1/phi^2 = 3 | TRINITY

module TrinityMemory<Module> {
    // ...
}
```

## Pinned Compiler Pitfalls

1. **Non-constant invariants become comments**: An `invariant` block that contains anything other than compile-time constant expressions will be emitted as a comment in the generated C, not as `_Static_assert`. Use only simple arithmetic comparisons of constants.

2. **Assertions starting with `(` are folded**: An `assert` statement that begins with `(` (i.e., a parenthesized expression) will be folded into a function call. Always bind such expressions to a `var` first: `var x = (complex expression); try eq(x, expected);`

## Module List

- `types.t27` - Memory types, error codes, and CRC parameters
- `bridge.t27` - Bridge between Trinity and memory systems
- `tensorpack.t27` - Tensor packaging and compression
- `stream_compute.t27` - Stream processing computation
- `conformance.t27` - Conformance test vectors
- `edge_demo.t27` - Edge computing demonstration
- `session.t27` - Session record layout and recovery rules

## Owner

The owner of this directory is recorded in `specs/memory/OWNERS.md`.
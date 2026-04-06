# VS Code T27 Language Support

Syntax highlighting for [T27](https://github.com/trinity-ssai/t27) (TRI-27), the ternary language for Trinity S³AI.

## Installation

### Development Install

```bash
cd vscode-trinity-swe
code --install-extension .
```

### From VSIX

```bash
npm install -g vsce
vsce package
code --install-extension vscode-trinity-swe-0.1.0.vsix
```

## Features

- Syntax highlighting for `.t27` files
- Support for T27 keywords: `module`, `fn`, `struct`, `enum`, `const`, `var`, `if`, `else`, `switch`, `for`, `return`, `test`, `invariant`, `bench`
- Ternary types: `Trit`, `PackedTrit`, `TernaryWord`, `Ternary`
- Builtins: `@as`, `@intCast`, `@intFromEnum`, `@compileAssert`, etc.
- Array types: `[N]Type` and `[_]Type`
- Switch expressions: `.case => result`
- Comments: `;` and `//`
- Phi constants: `PHI`, `PHI_INV`, `PHI_SQ`, `TRINITY`

## License

MIT

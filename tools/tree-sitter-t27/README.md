# tree-sitter-t27

T27 (TRI-27) grammar for [tree-sitter](https://tree-sitter.github.io/tree-sitter/), the parser generator used by Neovim, Helix, Zed, and other modern editors.

## Installation

### For Neovim (nvim-treesitter)

Add to your `init.lua`:

```lua
require'nvim-treesitter.configs'.setup {
  ensure_installed = { "t27" },
  parsers = {
    t27 = {
      install_info = {
        url = "https://github.com/trinity-ssai/t27",
        files = { "tools/tree-sitter-t27/src/parser.c" },
      },
    },
  },
}
```

### For Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "t27"
scope = "source.t27"
file-types = ["t27"]
roots = []
comment-token = ";"
injection-regex = "t27"

[language.auto-pairs]
'(' = ')'
'{' = '}'
'[' = ']'
'"' = '"'

[language.indent]
tab-width = 4
unit = "    "
```

## Development

```bash
# Generate parser.c from grammar.js
npm run generate

# Run tests
npm run test

# Parse a file manually
npm run parse -- examples/sample.t27
```

## License

MIT

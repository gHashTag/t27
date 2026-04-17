//! Experimental commands banner
//!
//! Displays warning about experimental commands that are stub implementations.

pub const BANNER: &str = r#"
⚠️  EXPERIMENTAL COMMANDS ⚠️

The following commands are STUB implementations pending full integration
with t27c in future rings (Ring-018/019):

  pipeline <spec.tri>              Run .tri → .trib → execute (E2E pipeline)
  bench    <spec.tri>              Run benchmarks from .tri spec
  parse    <spec.tri>              Validate .tri syntax

These commands will print placeholder messages and exit with code 1.
Use stable commands instead:
  tri gen    <spec.tri>              Generate code from spec
  tri test   <spec.tri>              Run tests
  tri status                           Show PHI LOOP status
  tri experience <subcommand>           Save/list/diff experiences

"#;

//! Front-end glue: exactly the t27c path, unmodified.
//!
//! `use` resolution splices imported declarations into the source the same way
//! every t27c `gen-*` and `typecheck` path does, with the same safety contract:
//! if the spliced source does not parse, the original source is used.

use crate::compiler::{typecheck_ast, Compiler, Node};
use std::path::Path;

pub struct Parsed {
    pub ast: Node,
    /// True when the `use`-spliced source was used.
    pub spliced: bool,
    pub notes: Vec<String>,
}

/// Parse `raw` (the contents of `path`), resolving `use` declarations.
pub fn parse(path: &Path, raw: &str) -> Result<Parsed, String> {
    let has_use = raw.contains("use ");
    if has_use {
        let spliced = crate::use_resolve::resolve(path, raw);
        if spliced != raw {
            match Compiler::parse_ast(&spliced) {
                Ok(ast) => {
                    return Ok(Parsed {
                        ast,
                        spliced: true,
                        notes: Vec::new(),
                    })
                }
                Err(splice_err) => {
                    let ast = Compiler::parse_ast(raw).map_err(|_| splice_err)?;
                    return Ok(Parsed {
                        ast,
                        spliced: false,
                        notes: vec![
                            "spliced source did not parse; compiling the unresolved original"
                                .to_string(),
                        ],
                    });
                }
            }
        }
    }
    let ast = Compiler::parse_ast(raw)?;
    Ok(Parsed {
        ast,
        spliced: false,
        notes: Vec::new(),
    })
}

/// Run t27c's own type checker. Returns its error messages when it fails.
pub fn typecheck(ast: &Node) -> Result<usize, Vec<String>> {
    let r = typecheck_ast(ast);
    if r.ok {
        Ok(r.warnings as usize)
    } else {
        Err(r.errors)
    }
}

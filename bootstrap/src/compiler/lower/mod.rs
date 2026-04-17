//! Lowering module
//!
//! Lowers AST to target IR formats (.tri IR, trib bytecode).
//! Part of modular compiler architecture (Ring-018).

pub mod tri_ir;
pub mod trib;

// Re-export lowering functions
pub use tri_ir::lower_to_tri_ir;
pub use trib::lower_to_trib;

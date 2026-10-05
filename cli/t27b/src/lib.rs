//! t27b: t27's own native backend (MVP).
//!
//! Pipeline: .t27 source -> t27c front-end (lexer, parser, typecheck_ast; the
//! frozen bootstrap/src/compiler.rs, mounted unmodified) -> AST ->
//! t27b IR (lower) -> AArch64 instruction selection and encoding (codegen, a64)
//! -> either an in-process JIT (jit) or a Mach-O MH_OBJECT file (macho).
//!
//! No LLVM, no zig, no clang, no rustc at run time; the only crate dependency is
//! serde, which compiler.rs needs for its derives.

// The frozen front-end. bootstrap/build.rs pins this file against
// bootstrap/stage0/FROZEN_HASH; it is included here byte-for-byte, never edited.
#[path = "../../../bootstrap/src/compiler.rs"]
#[allow(dead_code, unused, clippy::all)]
pub mod compiler;

// compiler.rs refers to crate::use_resolve (imported enums/structs for the
// Verilog path); the real module is mounted so the reference resolves and so
// `use` splicing behaves exactly as in t27c.
#[path = "../../../bootstrap/src/use_resolve.rs"]
#[allow(dead_code, unused, clippy::all)]
pub mod use_resolve;

pub mod a64;
pub mod blockers;
pub mod codegen;
pub mod eval;
pub mod front;
pub mod ir;
pub mod jit;
pub mod macho;
pub mod lower;
pub mod timing;

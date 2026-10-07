//! Hand-written Mach-O 64-bit relocatable object (MH_OBJECT) for arm64.
//!
//! Layout:
//!
//! ```text
//! mach_header_64                      32 bytes
//! LC_SEGMENT_64 + 2 section_64       232 bytes   (__TEXT,__text and __TEXT,__const)
//! LC_BUILD_VERSION                    24 bytes   (macOS 11.0)
//! LC_SYMTAB                           24 bytes
//! LC_DYSYMTAB                         80 bytes
//! padding to 16
//! __text                              the linked functions
//! __const                             read-only data blobs, 8-byte aligned
//! relocations                         two per data reference (PAGE21, PAGEOFF12)
//! nlist_64 symbols                    one local symbol per blob, then one
//!                                     external N_SECT symbol per fn
//! string table
//! ```
//!
//! Intra-module `bl` offsets are resolved at emit time, so calls need no
//! relocations. A data reference is an `adrp` + `add` pair; its target is
//! named by the blob's local symbol (`l_t27b_data.N`) so the linker computes
//! the page and page offset itself, exactly as for clang's own `adrp`/`add`
//! to a `l_.str` literal. The header does not set MH_SUBSECTIONS_VIA_SYMBOLS:
//! the linker must keep `__text` as one block.
//!
//! The layout and the bytes of the header, the load commands, the
//! relocations and the symbols are specs/tri/t27b/macho.t27 (#6198); this
//! file collects the names and copies text, data and strings into place.

use crate::codegen::{self, Linked};

#[path = "../../../gen/rust/tri/t27b/macho.rs"]
#[allow(dead_code, unused_parens, unused_mut, unused_assignments, unused_variables, non_snake_case, non_upper_case_globals, clippy::all)]
mod spec; // t27c gen-rust of specs/tri/t27b/macho.t27

/// Serialise a linked image (no prefix) with one global symbol per function.
/// `names[i]` is the t27 name of function i; the symbol is `_` + name. An
/// empty name exports nothing (a function only other functions call).
/// `data` is `Program::data`, referenced by `linked.data_refs`.
pub fn object(linked: &Linked, names: &[String], data: &[Vec<u8>]) -> Vec<u8> {
    let text: Vec<u8> = linked.code.iter().flat_map(|w| w.to_le_bytes()).collect();
    let mut syms: Vec<(String, u64)> = Vec::new();
    for (i, off) in linked.offsets.iter().enumerate() {
        if let (Some(o), false) = (off, names[i].is_empty()) {
            syms.push((format!("_{}", names[i]), (*o as u64) * 4));
        }
    }
    syms.sort();
    let (blob_offs, const_size) = codegen::data_layout(data);
    // Only referenced blobs need a symbol; the others are still emitted (the
    // layout is the interpreter's), just unnamed.
    let mut used: Vec<u32> = linked.data_refs.iter().map(|&(_, k)| k).collect();
    used.sort_unstable();
    used.dedup();

    let text_size = text.len();
    let const_addr = spec::const_addr(text_size);
    let nlocal = used.len();
    let nsyms = nlocal + syms.len();
    let mut strtab: Vec<u8> = vec![0];
    let mut strx = Vec::with_capacity(nsyms);
    for k in &used {
        strx.push(strtab.len() as u32);
        strtab.extend_from_slice(format!("l_t27b_data.{}", k).as_bytes());
        strtab.push(0);
    }
    for (n, _) in &syms {
        strx.push(strtab.len() as u32);
        strtab.extend_from_slice(n.as_bytes());
        strtab.push(0);
    }
    while strtab.len() % 8 != 0 {
        strtab.push(0);
    }

    let nrefs = linked.data_refs.len();
    let mut w = vec![0u8; spec::object_size(text_size, const_size, nrefs, nsyms, strtab.len())];
    spec::put_commands(&mut w, text_size, const_size, nrefs, nlocal, syms.len(), strtab.len());
    let text_off = spec::text_off(const_size);
    w[text_off..text_off + text_size].copy_from_slice(&text);
    if const_size > 0 {
        for (b, &o) in data.iter().zip(&blob_offs) {
            let at = text_off + const_addr + o;
            w[at..at + b.len()].copy_from_slice(b);
        }
    }
    let mut at = spec::reloff(text_size, const_size);
    for &(p, k) in &linked.data_refs {
        let sym = used.binary_search(&k).expect("referenced blob has a symbol") as u32;
        at = spec::put_reloc_pair(&mut w, at, p, sym);
    }
    for (j, k) in used.iter().enumerate() {
        at = spec::put_nlist(&mut w, at, strx[j], spec::N_SECT, spec::SECT_CONST, (const_addr + blob_offs[*k as usize]) as u64);
    }
    for (k, (_, value)) in syms.iter().enumerate() {
        at = spec::put_nlist(&mut w, at, strx[nlocal + k], spec::N_SECT | spec::N_EXT, spec::SECT_TEXT, *value);
    }
    w[at..].copy_from_slice(&strtab);
    w
}

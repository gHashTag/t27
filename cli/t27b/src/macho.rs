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

use crate::codegen::{self, Linked};

const MH_MAGIC_64: u32 = 0xfeed_facf;
const CPU_TYPE_ARM64: u32 = 0x0100_000c;
const CPU_SUBTYPE_ARM64_ALL: u32 = 0;
const MH_OBJECT: u32 = 0x1;
const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x2;
const LC_DYSYMTAB: u32 = 0xb;
const LC_BUILD_VERSION: u32 = 0x32;
const PLATFORM_MACOS: u32 = 1;
const MINOS_11_0: u32 = 0x000b_0000;
const S_ATTR_PURE_INSTRUCTIONS: u32 = 0x8000_0000;
const S_ATTR_SOME_INSTRUCTIONS: u32 = 0x0000_0400;
const S_REGULAR: u32 = 0x0;
const N_SECT: u8 = 0xe;
const N_EXT: u8 = 0x1;
const ARM64_RELOC_PAGE21: u32 = 3;
const ARM64_RELOC_PAGEOFF12: u32 = 4;

struct W(Vec<u8>);

impl W {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn name16(&mut self, s: &str) {
        let mut b = [0u8; 16];
        b[..s.len()].copy_from_slice(s.as_bytes());
        self.0.extend_from_slice(&b);
    }
    fn pad_to(&mut self, align: usize) {
        while self.0.len() % align != 0 {
            self.0.push(0);
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn section(&mut self, sect: &str, seg: &str, addr: u64, size: u64, off: u32, align: u32, reloff: u32, nreloc: u32, flags: u32) {
        self.name16(sect);
        self.name16(seg);
        self.u64(addr);
        self.u64(size);
        self.u32(off);
        self.u32(align);
        self.u32(reloff);
        self.u32(nreloc);
        self.u32(flags);
        self.u32(0);
        self.u32(0);
        self.u32(0);
    }
}

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

    let nsects: u32 = if const_size > 0 { 2 } else { 1 };
    let sizeofcmds: u32 = (72 + 80 * nsects) + 24 + 24 + 80;
    let text_off = ((32 + sizeofcmds as usize) + 15) & !15;
    let text_size = text.len();
    // __const follows __text in both the file and the address space.
    let const_addr = (text_size + 15) & !15;
    let vm_size = if const_size > 0 { const_addr + const_size } else { text_size };
    let reloff = (text_off + vm_size + 7) & !7;
    let nreloc = 2 * linked.data_refs.len();
    let symoff = reloff + 8 * nreloc;
    let nlocal = used.len();
    let nsyms = nlocal + syms.len();
    let stroff = symoff + 16 * nsyms;
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

    let mut w = W(Vec::with_capacity(stroff + strtab.len()));
    // mach_header_64
    w.u32(MH_MAGIC_64);
    w.u32(CPU_TYPE_ARM64);
    w.u32(CPU_SUBTYPE_ARM64_ALL);
    w.u32(MH_OBJECT);
    w.u32(4); // ncmds
    w.u32(sizeofcmds);
    w.u32(0); // flags
    w.u32(0); // reserved
    // LC_SEGMENT_64 (unnamed, as in every MH_OBJECT)
    w.u32(LC_SEGMENT_64);
    w.u32(72 + 80 * nsects);
    w.name16("");
    w.u64(0); // vmaddr
    w.u64(vm_size as u64); // vmsize
    w.u64(text_off as u64); // fileoff
    w.u64(vm_size as u64); // filesize
    w.u32(7); // maxprot rwx
    w.u32(7); // initprot
    w.u32(nsects);
    w.u32(0); // flags
    // align 2^4: functions are laid out on 16-byte boundaries
    let (r_off, r_n) = if nreloc > 0 { (reloff as u32, nreloc as u32) } else { (0, 0) };
    w.section("__text", "__TEXT", 0, text_size as u64, text_off as u32, 4, r_off, r_n, S_ATTR_PURE_INSTRUCTIONS | S_ATTR_SOME_INSTRUCTIONS);
    if const_size > 0 {
        w.section("__const", "__TEXT", const_addr as u64, const_size as u64, (text_off + const_addr) as u32, 3, 0, 0, S_REGULAR);
    }
    // LC_BUILD_VERSION
    w.u32(LC_BUILD_VERSION);
    w.u32(24);
    w.u32(PLATFORM_MACOS);
    w.u32(MINOS_11_0);
    w.u32(0); // sdk n/a
    w.u32(0); // ntools
    // LC_SYMTAB
    w.u32(LC_SYMTAB);
    w.u32(24);
    w.u32(symoff as u32);
    w.u32(nsyms as u32);
    w.u32(stroff as u32);
    w.u32(strtab.len() as u32);
    // LC_DYSYMTAB: the blob symbols are local, the functions external.
    w.u32(LC_DYSYMTAB);
    w.u32(80);
    w.u32(0); // ilocalsym
    w.u32(nlocal as u32); // nlocalsym
    w.u32(nlocal as u32); // iextdefsym
    w.u32(syms.len() as u32); // nextdefsym
    w.u32(nsyms as u32); // iundefsym
    w.u32(0); // nundefsym
    for _ in 0..12 {
        w.u32(0); // tocoff .. nlocrel
    }
    debug_assert_eq!(w.0.len(), 32 + sizeofcmds as usize);
    w.pad_to(16);
    debug_assert_eq!(w.0.len(), text_off);
    w.0.extend_from_slice(&text);
    if const_size > 0 {
        w.pad_to(16);
        debug_assert_eq!(w.0.len(), text_off + const_addr);
        for (b, &o) in data.iter().zip(&blob_offs) {
            w.0.resize(text_off + const_addr + o, 0);
            w.0.extend_from_slice(b);
        }
        w.0.resize(text_off + const_addr + const_size, 0);
    }
    w.pad_to(8);
    debug_assert_eq!(w.0.len(), reloff);
    // relocation_info: r_address, then r_symbolnum:24 r_pcrel:1 r_length:2
    // r_extern:1 r_type:4 (low bits first).
    for &(p, k) in &linked.data_refs {
        let sym = used.binary_search(&k).expect("referenced blob has a symbol") as u32;
        for (at, pcrel, ty) in [(p, 1u32, ARM64_RELOC_PAGE21), (p + 1, 0, ARM64_RELOC_PAGEOFF12)] {
            w.u32((at * 4) as u32);
            w.u32(sym | pcrel << 24 | 2 << 25 | 1 << 27 | ty << 28);
        }
    }
    debug_assert_eq!(w.0.len(), symoff);
    for (j, k) in used.iter().enumerate() {
        w.u32(strx[j]);
        w.u8(N_SECT);
        w.u8(2); // n_sect: __const
        w.u16(0);
        w.u64((const_addr + blob_offs[*k as usize]) as u64);
    }
    for (k, (_, value)) in syms.iter().enumerate() {
        w.u32(strx[nlocal + k]);
        w.u8(N_SECT | N_EXT);
        w.u8(1); // n_sect: __text
        w.u16(0); // n_desc
        w.u64(*value);
    }
    w.0.extend_from_slice(&strtab);
    w.0
}

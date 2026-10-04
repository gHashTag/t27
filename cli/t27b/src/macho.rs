//! Hand-written Mach-O 64-bit relocatable object (MH_OBJECT) for arm64.
//!
//! Layout:
//!
//! ```text
//! mach_header_64                      32 bytes
//! LC_SEGMENT_64 + 1 section_64       152 bytes   (__TEXT,__text)
//! LC_BUILD_VERSION                    24 bytes   (macOS 11.0)
//! LC_SYMTAB                           24 bytes
//! LC_DYSYMTAB                         80 bytes
//! padding to 16
//! __text                              the linked functions
//! nlist_64 symbols                    one external N_SECT symbol per fn
//! string table
//! ```
//!
//! Intra-module `bl` offsets are resolved at emit time, so the section has no
//! relocations. For the same reason the header does not set
//! MH_SUBSECTIONS_VIA_SYMBOLS: the linker must keep `__text` as one block.

use crate::codegen::Linked;

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
const N_SECT: u8 = 0xe;
const N_EXT: u8 = 0x1;

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
}

/// Serialise a linked image (no prefix) with one global symbol per function.
/// `names[i]` is the t27 name of function i; the symbol is `_` + name.
pub fn object(linked: &Linked, names: &[String]) -> Vec<u8> {
    let text: Vec<u8> = linked.code.iter().flat_map(|w| w.to_le_bytes()).collect();
    let mut syms: Vec<(String, u64)> = Vec::new();
    for (i, off) in linked.offsets.iter().enumerate() {
        if let Some(o) = off {
            syms.push((format!("_{}", names[i]), (*o as u64) * 4));
        }
    }
    syms.sort();

    let sizeofcmds: u32 = (72 + 80) + 24 + 24 + 80;
    let text_off = ((32 + sizeofcmds as usize) + 15) & !15;
    let text_size = text.len();
    let symoff = (text_off + text_size + 7) & !7;
    let nsyms = syms.len();
    let stroff = symoff + 16 * nsyms;
    let mut strtab: Vec<u8> = vec![0];
    let mut strx = Vec::with_capacity(nsyms);
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
    w.u32(72 + 80);
    w.name16("");
    w.u64(0); // vmaddr
    w.u64(text_size as u64); // vmsize
    w.u64(text_off as u64); // fileoff
    w.u64(text_size as u64); // filesize
    w.u32(7); // maxprot rwx
    w.u32(7); // initprot
    w.u32(1); // nsects
    w.u32(0); // flags
    // section_64 __TEXT,__text
    w.name16("__text");
    w.name16("__TEXT");
    w.u64(0); // addr
    w.u64(text_size as u64);
    w.u32(text_off as u32);
    w.u32(4); // align 2^4: functions are laid out on 16-byte boundaries
    w.u32(0); // reloff
    w.u32(0); // nreloc
    w.u32(S_ATTR_PURE_INSTRUCTIONS | S_ATTR_SOME_INSTRUCTIONS);
    w.u32(0);
    w.u32(0);
    w.u32(0);
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
    // LC_DYSYMTAB: all symbols are external definitions.
    w.u32(LC_DYSYMTAB);
    w.u32(80);
    w.u32(0); // ilocalsym
    w.u32(0); // nlocalsym
    w.u32(0); // iextdefsym
    w.u32(nsyms as u32); // nextdefsym
    w.u32(nsyms as u32); // iundefsym
    w.u32(0); // nundefsym
    for _ in 0..12 {
        w.u32(0); // tocoff .. nlocrel
    }
    debug_assert_eq!(w.0.len(), 32 + sizeofcmds as usize);
    w.pad_to(16);
    debug_assert_eq!(w.0.len(), text_off);
    w.0.extend_from_slice(&text);
    w.pad_to(8);
    debug_assert_eq!(w.0.len(), symoff);
    for (k, (_, value)) in syms.iter().enumerate() {
        w.u32(strx[k]);
        w.u8(N_SECT | N_EXT);
        w.u8(1); // n_sect: __text
        w.u16(0); // n_desc
        w.u64(*value);
    }
    w.0.extend_from_slice(&strtab);
    w.0
}

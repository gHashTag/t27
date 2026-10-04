// bitwalk -- walk a 7-series .bit with the rules of specs/xilinx7/{packets,frames,far}.t27 and nothing else.
// I/O and loops only; every decision is a function of the generated module.
//
//   bitwalk FILE...                         report packets, CRC checks, FDRI, frame ECC, FAR walk, COR0, IDCODE
//   bitwalk --pins TILES FILE               for each line "pin tile base frames offset words" of TILES, does
//                                           the tile carry data in the frames the FAR walk assigns it?
//   bitwalk --bits SEGS FILE                is every set bit of FILE a bit some tile's segbits name, at the
//                                           frame the FAR walk assigns? SEGS: "S type minor bit" segbits,
//                                           "T type base frames offset words shift" tiles (shift: alias)
//   bitwalk --frame FILE [MIN]              print the sparsest frame with a nonzero ECC and >= MIN nonzero words
//   bitwalk --cor0 N [--no-reseal] IN OUT   rewrite COR0's OSCFSEL, re-seal the CRC words
//   bitwalk --write FRAMES OUT --part_file part.yaml --part_name NAME [--source S] [--generator G]
//           [--date D] [--time T]           write FRAMES (prjxray .frames text) as a .bit: frames placed
//                                           by the FAR walk, ECC sealed, packets as packets.t27's SEQ;
//                                           the header fields default to xc7frames2bit's
//   bitwalk --frames BIT OUT                write BIT's nonzero frames as .frames text, addresses from
//                                           the FAR walk (the inverse of --write)
//   bitwalk --fasm DIGEST FASM OUT [--strict]  FASM -> .frames text as prjxray's fasm2frames writes it (not
//                                           sparse); DIGEST is prjxray-db as x7.py fasm_digest renders it;
//                                           segbit positions from frames.t27; --strict refuses a bit
//                                           outside its tile's own words (fasm2frames writes it)
//
// Build (packets.rs and frames.rs are generated, never committed):
//   t27c gen-rust specs/xilinx7/packets.t27 > specs/xilinx7/packets.rs
//   t27c gen-rust specs/xilinx7/frames.t27 > specs/xilinx7/frames.rs
//   t27c gen-rust specs/xilinx7/far.t27 > specs/xilinx7/far.rs
//   rustc -O --edition 2021 specs/xilinx7/bitwalk.rs -o specs/xilinx7/bitwalk
#[allow(unused_parens, dead_code, non_snake_case)]
#[path = "packets.rs"]
mod p;
#[allow(unused_parens, dead_code, non_snake_case)]
#[path = "frames.rs"]
mod f;
#[allow(unused_parens, dead_code, non_snake_case)]
#[path = "far.rs"]
mod w;

use std::collections::{BTreeMap, HashMap, HashSet};

struct Walk {
    sync_at: usize,
    checks: u32,
    bad: u32,
    fdri: u64,
    cor0: Option<u32>,
    idcode: Option<u32>,
    cmds: Vec<u32>,
    writes: BTreeMap<u32, u64>,
    patched: u32,
    ecc_frames: u32,
    ecc_bad: u32,
    ecc_nonzero: u32,
    ecc_data: u32,
    // (frame number, nonzero (word, value) pairs, stored ECC) of the sparsest frame with a nonzero ECC
    sparse: Option<(u32, Vec<(u32, u32)>, u32)>,
    // every FDRI frame, in stream order
    frames: Vec<[u32; 101]>,
}

impl Walk {
    // far.t27's part number for the IDCODE this stream writes (PARTS if none or unknown).
    fn part(&self) -> u32 {
        self.idcode.map_or(w::PARTS, w::part_of_idcode)
    }
}

fn word(b: &[u8], i: usize) -> u32 {
    u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

fn walk(b: &mut [u8], cor0_new: Option<u32>, reseal: bool, min_nz: usize) -> Walk {
    let sync = p::SYNC.to_be_bytes();
    let sync_at = b.windows(4).position(|w| w == sync).expect("no sync word");
    let mut r = Walk { sync_at, checks: 0, bad: 0, fdri: 0, cor0: None, idcode: None,
                       cmds: vec![], writes: BTreeMap::new(), patched: 0,
                       ecc_frames: 0, ecc_bad: 0, ecc_nonzero: 0, ecc_data: 0, sparse: None, frames: vec![] };
    let (mut ecc, mut w50, mut frame) = (0u32, 0u32, [0u32; 101]);
    let (mut reg, mut op, mut left) = (0u32, p::OP_NOP, 0u32);
    let (mut crc, mut crc_new) = (0u32, 0u32);
    let mut i = sync_at + 4;
    while i + 4 <= b.len() {
        let w = word(b, i);
        let in_payload = left > 0;
        if !in_payload {
            reg = p::reg_after(reg, w);
            op = p::op_after(op, w);
            left = p::left_after(0, w);
            i += 4;
            continue;
        }
        left = p::left_after(left, w);
        let is_write = op == p::OP_WRITE;
        if is_write {
            *r.writes.entry(reg).or_default() += 1;
            let mut w_new = w;
            if reg == p::REG_CRC {
                r.checks += 1;
                if !p::crc_ok(w, crc) { r.bad += 1; }
                if reseal { w_new = p::resealed(w, crc, crc_new); }
            } else if reg == p::REG_COR0 {
                r.cor0 = Some(w);
                if let Some(v) = cor0_new { w_new = p::cor0_with_oscfsel(w, v); }
            } else if reg == p::REG_IDCODE {
                r.idcode = Some(w);
            } else if reg == p::REG_CMD {
                r.cmds.push(w);
            } else if reg == p::REG_FDRI {
                let idx = (r.fdri % f::FRAME_WORDS as u64) as u32;
                frame[idx as usize] = w;
                ecc = f::ecc_after(idx, w, ecc);
                if idx == f::ECC_WORD { w50 = w; }
                if idx == f::LAST_WORD {
                    r.frames.push(frame);
                    r.ecc_frames += 1;
                    if !f::ecc_ok(w50, ecc) { r.ecc_bad += 1; }
                    if frame.iter().any(|&v| v != 0) { r.ecc_data += 1; }
                    let stored = f::ecc_stored(w50);
                    if stored != 0 {
                        r.ecc_nonzero += 1;
                        let nz: Vec<(u32, u32)> = (0..101u32).filter(|&k| frame[k as usize] != 0)
                            .map(|k| (k, frame[k as usize])).collect();
                        let fewer = r.sparse.as_ref().map_or(true, |s| nz.len() < s.1.len());
                        if nz.len() >= min_nz && fewer { r.sparse = Some((r.ecc_frames - 1, nz, stored)); }
                    }
                    ecc = 0;
                }
                r.fdri += 1;
            }
            if w_new != w {
                b[i..i + 4].copy_from_slice(&w_new.to_be_bytes());
                r.patched += 1;
            }
            crc = p::crc_after(reg, w, crc);
            crc_new = p::crc_after(reg, w_new, crc_new);
        }
        i += 4;
    }
    r
}

// Pad frames of a whole-part write starting at FAR 0: the ROW_PAD_FRAMES after each group.
fn pad_positions(part: u32) -> Vec<u32> {
    let (mut at, mut pads) = (0u32, vec![]);
    let g0 = w::first_group(part);
    for g in g0..g0 + w::PART_GROUPS[part as usize] {
        at += w::group_frames(g);
        for k in 0..w::ROW_PAD_FRAMES { pads.push(at + k); }
        at += w::ROW_PAD_FRAMES;
    }
    pads
}

fn report(name: &str, r: &Walk) {
    let frames = r.fdri / p::WORDS_PER_FRAME as u64;
    let whole = p::whole_frames(r.fdri as u32);
    let cor0 = r.cor0.map(|c| format!("0x{:08X} (OSCFSEL {})", c, p::cor0_oscfsel(c))).unwrap_or("-".into());
    let id = r.idcode.map(|c| format!("0x{:08X}", c)).unwrap_or("-".into());
    println!("{name}\n  sync @ byte {}  CRC checks {} (bad {})  FDRI {} words = {} frames (whole: {})\n  COR0 {}  IDCODE {}  CMD {:?}",
             r.sync_at, r.checks, r.bad, r.fdri, frames, whole, cor0, id, r.cmds);
    println!("  ECC frames {} (bad {}; with data {}, nonzero ECC {})", r.ecc_frames, r.ecc_bad, r.ecc_data, r.ecc_nonzero);
    let part = r.part();
    if part == w::PARTS {
        println!("  FAR walk: IDCODE {} is not in far.t27's part table", id);
        return;
    }
    let walk = w::part_fdri_frames(part);
    let off = (walk as i64 - r.frames.len() as i64).unsigned_abs();
    let dirty = pad_positions(part).iter()
        .filter(|&&k| r.frames.get(k as usize).map_or(false, |fr| fr.iter().any(|&v| v != 0))).count();
    println!("  FAR walk {} frames (FDRI {}, off by {}; pad frames with data {})", walk, r.frames.len(), off, dirty);
    if let Some(c) = r.cor0 {
        let f: Vec<String> = (0..p::COR0_FIELDS.len())
            .map(|k| format!("{}={}", p::COR0_FIELDS[k], p::field(c, p::COR0_HI[k], p::COR0_LO[k])))
            .collect();
        println!("  COR0 fields: {}", f.join(" "));
    }
}

// Every address of part p, in FDRI order; pads are None.
fn walk_addresses(part: u32) -> Vec<Option<u32>> {
    let mut out = vec![];
    let g0 = w::first_group(part);
    for g in g0..g0 + w::PART_GROUPS[part as usize] {
        let first = w::first_column(g);
        let cols = w::GROUP_COLS[g as usize];
        let mut a = w::group_start(w::GROUP_KEY[g as usize]);
        while a != w::NO_FRAME {
            out.push(Some(a));
            a = w::far_next_in_row(a, w::COL_FRAMES[(first + w::far_column(a)) as usize], cols);
        }
        for _ in 0..w::ROW_PAD_FRAMES {
            out.push(None);
        }
    }
    out
}

fn flag<'a>(a: &'a [String], name: &str) -> Option<&'a str> {
    a.iter().position(|s| s == name).and_then(|i| a.get(i + 1)).map(|s| s.as_str())
}

fn bit_field(out: &mut Vec<u8>, key: u8, text: &str) {
    out.push(key);
    let n = text.len() + 1;
    out.extend_from_slice(&[(n >> 8) as u8, n as u8]);
    out.extend_from_slice(text.as_bytes());
    out.push(0);
}

// frames text -> .bit. Returns the number of frames rejected (no address of the part).
fn write_bit(a: &[String]) -> u32 {
    let (frames_path, out_path) = (&a[1], &a[2]);
    let yaml = std::fs::read_to_string(flag(a, "--part_file").expect("--part_file part.yaml")).unwrap();
    let part_name = flag(a, "--part_name").expect("--part_name NAME");
    let idcode = yaml
        .lines()
        .find_map(|l| l.trim().strip_prefix("idcode:"))
        .map(|v| {
            let v = v.trim();
            v.strip_prefix("0x").map_or_else(|| v.parse().unwrap(), |h| u32::from_str_radix(h, 16).unwrap())
        })
        .expect("idcode: in part file");
    let part = w::part_of_idcode(idcode);
    assert!(part != w::PARTS, "IDCODE 0x{idcode:08X} is not in far.t27's part table");
    let nframes = w::part_fdri_frames(part) as usize;
    let fw = f::FRAME_WORDS as usize;
    let mut data = vec![0u32; nframes * fw];
    let mut placed = vec![false; nframes];
    let (mut rejected, mut dup, mut short) = (0u32, 0u32, 0u32);
    for line in std::fs::read_to_string(frames_path).unwrap().lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let (addr, words) = line.split_once(' ').expect("ADDR WORDS");
        let addr = u32::from_str_radix(addr.trim_start_matches("0x"), 16).unwrap();
        let words: Vec<u32> = words.split(',').map(|v| u32::from_str_radix(v.trim().trim_start_matches("0x"), 16).unwrap()).collect();
        if words.len() != fw {
            short += 1;
            continue;
        }
        let i = w::fdri_index(part, addr);
        if i == w::NO_FRAME {
            println!("  REJECT 0x{addr:08X}: not an address of this part");
            rejected += 1;
            continue;
        }
        let i = i as usize;
        if placed[i] {
            dup += 1;
            continue;
        }
        placed[i] = true;
        data[i * fw..(i + 1) * fw].copy_from_slice(&words);
    }
    for fr in data.chunks_mut(fw) {
        let mut ecc = 0u32;
        for (k, v) in fr.iter().enumerate() {
            ecc = f::ecc_after(k as u32, *v, ecc);
        }
        let e = f::ECC_WORD as usize;
        fr[e] = f::with_ecc(fr[e], ecc);
    }
    let mut words: Vec<u32> = vec![];
    let frame_words = data.len() as u32;
    for s in 0..p::SEQ_STEPS {
        for j in 0..p::step_words(s) {
            words.push(p::step_word(s, j, idcode, frame_words));
        }
        if p::step_kind(s) == p::STEP_FDRI {
            words.extend_from_slice(&data);
        }
    }
    let mut out: Vec<u8> = p::BIT_MAGIC.iter().map(|&b| b as u8).collect();
    let source = flag(a, "--source").map_or_else(
        || std::path::Path::new(frames_path).file_name().unwrap().to_string_lossy().into_owned(),
        |s| s.to_string(),
    );
    let generator = flag(a, "--generator").unwrap_or("bitwalk");
    bit_field(&mut out, b'a', &format!("{source};Generator={generator}"));
    bit_field(&mut out, b'b', part_name);
    bit_field(&mut out, b'c', flag(a, "--date").unwrap_or("2000/01/01"));
    bit_field(&mut out, b'd', flag(a, "--time").unwrap_or("00:00:00"));
    out.push(b'e');
    out.extend_from_slice(&((words.len() * 4) as u32).to_be_bytes());
    for v in &words {
        out.extend_from_slice(&v.to_be_bytes());
    }
    std::fs::write(out_path, &out).unwrap();
    let given = placed.iter().filter(|&&x| x).count();
    println!(
        "{out_path}: {} bytes, {} words, FDRI {} frames ({} from FRAMES, rejected {}, duplicate {}, wrong length {})",
        out.len(), words.len(), nframes, given, rejected, dup, short
    );
    rejected
}

// ---- --fasm: FASM -> frames, as prjxray's fasm2frames (non-sparse) --------------------------
// The db digest (x7.py fasm_digest) is prjxray-db as prjxray's own Database reads it:
//   T tile own_type seg_type [site=alias_site ...]   K block base frames offset shift words
//   S tile IOB_Yn   F block KEY minor_bit|!minor_bit ...   P KEY   R feature   B tile bank
// Every bit position comes from frames.t27 (seg_pos, pos_in_frame, bit_merge); this is lookup
// and text.

struct Tile { own: String, seg: String, sites: Vec<(String, String)>, blocks: Vec<[u32; 7]>, iob: Vec<String> }

struct Db {
    tiles: HashMap<String, Tile>,
    feats: Vec<(u32, Vec<(u32, u32, bool)>)>,
    exact: HashMap<String, [usize; 2]>,
    addr: HashMap<(String, u32), usize>,
    ppips: HashSet<String>,
    required: Vec<String>,
    tile_bank: HashMap<String, String>,
    bank_tiles: HashMap<String, Vec<String>>,
}

const NOFEAT: usize = usize::MAX;

fn read_db(text: &str) -> Db {
    let mut db = Db { tiles: HashMap::new(), feats: vec![], exact: HashMap::new(), addr: HashMap::new(),
                      ppips: HashSet::new(), required: vec![], tile_bank: HashMap::new(), bank_tiles: HashMap::new() };
    let mut cur = String::new();
    for line in text.lines() {
        let mut t = line.split(' ');
        match t.next() {
            Some("T") => {
                let (name, own, seg) = (t.next().unwrap(), t.next().unwrap(), t.next().unwrap());
                let sites = t.map(|kv| { let (a, b) = kv.split_once('=').unwrap(); (a.to_string(), b.to_string()) }).collect();
                cur = name.to_string();
                db.tiles.insert(cur.clone(), Tile { own: own.into(), seg: seg.into(), sites, blocks: vec![], iob: vec![] });
            }
            Some("K") => {
                // The offset is signed: kintex7 has tiles that start below their frame (-2).
                let n: Vec<i64> = t.map(|x| x.parse().unwrap()).collect();
                let (off, below) = (n[3].max(0) as u32, (-n[3]).max(0) as u32);
                db.tiles.get_mut(&cur).unwrap().blocks.push([n[0] as u32, n[1] as u32, n[2] as u32, off, n[4] as u32, n[5] as u32, below]);
            }
            Some("S") => {
                let tile = t.next().unwrap();
                db.tiles.get_mut(tile).unwrap().iob.push(t.next().unwrap().to_string());
            }
            Some("F") => {
                let bt: u32 = t.next().unwrap().parse().unwrap();
                let key = t.next().unwrap();
                let bits = t.filter(|x| !x.is_empty()).map(|x| {
                    let (set, x) = match x.strip_prefix('!') { Some(r) => (false, r), None => (true, x) };
                    let (m, b) = x.split_once('_').unwrap();
                    (m.parse().unwrap(), b.parse().unwrap(), set)
                }).collect();
                let i = db.feats.len();
                db.feats.push((bt, bits));
                db.exact.entry(key.to_string()).or_insert([NOFEAT; 2])[bt as usize] = i;
                if let (Some(s), Some(e)) = (key.rfind('['), key.rfind(']')) {
                    if let Ok(n) = key[s + 1..e].parse::<u32>() {
                        db.addr.insert((key[..s].to_string(), n), i);
                    }
                }
            }
            Some("P") => { db.ppips.insert(t.next().unwrap().to_string()); }
            Some("R") => db.required.push(line[2..].to_string()),
            Some("B") => {
                let (tile, bank) = (t.next().unwrap().to_string(), t.next().unwrap().to_string());
                db.tile_bank.insert(tile.clone(), bank.clone());
                let v = db.bank_tiles.entry(bank).or_default();
                if !v.contains(&tile) { v.push(tile); }
            }
            _ => {}
        }
    }
    db
}

// A FASM line, as fasm's textX grammar reads it: feature, optional [a] or [e:s], optional
// "= value" (Verilog literal or plain decimal), annotations {...}, comment #... .
// Returns (feature, start, end, value bits LSB first); None for a line with no feature.
fn parse_fasm_line(line: &str) -> Result<Option<(String, Option<u32>, Option<u32>, Vec<bool>)>, String> {
    let s = line.trim_start();
    let n = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.')).unwrap_or(s.len());
    if n == 0 {
        let rest = s.trim_start();
        let blank = rest.is_empty() || rest.starts_with('#') || rest.starts_with('{');
        return if blank { Ok(None) } else { Err(format!("cannot parse: {line}")) };
    }
    let feature = s[..n].to_string();
    let mut rest = &s[n..];
    let (mut start, mut end) = (None, None);
    if let Some(r) = rest.strip_prefix('[') {
        let close = r.find(']').ok_or(format!("no ]: {line}"))?;
        let inner = &r[..close];
        let num = |x: &str| x.replace('_', "").parse::<u32>().map_err(|_| format!("address: {line}"));
        match inner.split_once(':') {
            Some((e, st)) => { end = Some(num(e)?); start = Some(num(st)?); }
            None => start = Some(num(inner)?),
        }
        rest = &r[close + 1..];
    }
    rest = rest.trim_start();
    let mut value = vec![true];
    if let Some(r) = rest.strip_prefix('=') {
        let stop = r.find(|c| c == '{' || c == '#').unwrap_or(r.len());
        let v: String = r[..stop].chars().filter(|c| !c.is_whitespace()).collect();
        value = verilog_bits(&v).ok_or(format!("value: {line}"))?;
    }
    Ok(Some((feature, start, end, value)))
}

fn verilog_bits(v: &str) -> Option<Vec<bool>> {
    let (base, digits) = match v.split_once('\'') {
        Some((_, r)) => (r.chars().next()?.to_ascii_lowercase(), &r[1..]),
        None => ('d', v),
    };
    let digits: Vec<u32> = digits.chars().filter(|&c| c != '_').map(|c| c.to_digit(16)).collect::<Option<_>>()?;
    let mut bits = vec![];
    let per = match base { 'h' => 4, 'o' => 3, 'b' => 1, 'd' => 0, _ => return None };
    if per > 0 {
        for &d in digits.iter().rev() {
            if d >> per != 0 { return None; }
            for k in 0..per { bits.push(d >> k & 1 == 1); }
        }
        return Some(bits);
    }
    let mut dec = digits;
    if dec.iter().any(|&d| d > 9) { return None; }
    while dec.iter().any(|&d| d != 0) {
        let mut carry = 0;
        for d in dec.iter_mut() { let x = carry * 10 + *d; *d = x / 2; carry = x % 2; }
        bits.push(carry == 1);
    }
    Some(bits)
}

struct Asm<'a> {
    db: &'a Db,
    bits: HashMap<(u32, bool, u32), u32>,
    missing: Vec<String>,
    set_features: Vec<String>,
    wrapped: u32,
    dropped: u32,
    foreign: u32,
    first_foreign: Option<String>,
    // The tile's own bits that still leave the frame, because the tile starts below it
    // (kintex7 offset -2). Kept apart from `foreign` so that wrapped + dropped stays a second
    // formula for `foreign` alone on every tile that starts inside its frame.
    own_out: u32,
    ppip_lines: u32,
}

impl<'a> Asm<'a> {
    fn lookup(&self, key: &str, address: u32) -> Option<usize> {
        if address == 0 {
            if let Some(e) = self.db.exact.get(key) {
                for &i in e { if i != NOFEAT { return Some(i); } }
            }
        }
        self.db.addr.get(&(key.to_string(), address)).copied()
    }

    fn enable(&mut self, tile: &str, feature: &str, address: u32, line: &str) -> Result<(), String> {
        let t = self.db.tiles.get(tile).ok_or(format!("no tile {tile} in the part, line '{line}'"))?;
        let db_k = format!("{}.{}", t.own, feature);
        if self.db.ppips.contains(&db_k) { self.ppip_lines += 1; return Ok(()); }
        let mut key = db_k.clone();
        if t.seg != t.own {
            let mut parts: Vec<&str> = db_k.split('.').collect();
            parts[0] = &t.seg;
            if parts.len() > 1 {
                if let Some((_, to)) = t.sites.iter().find(|(from, _)| from == parts[1]) { parts[1] = to; }
            }
            key = parts.join(".");
            if self.db.ppips.contains(&key) { self.ppip_lines += 1; return Ok(()); }
        }
        let lost = || format!("Segment DB {}, key {} not found from line '{}'", t.own, db_k, line);
        let i = self.lookup(&key, address).ok_or_else(lost)?;
        let (bt, ref fb) = self.db.feats[i];
        let k = *t.blocks.iter().find(|k| k[0] == bt).ok_or_else(lost)?;
        let (base, off, shift, words) = (k[1], k[3], k[4], k[5]);
        let at = f::seg_shift(k[6], shift);
        for &(minor, bit, set) in fb {
            // Outside the tile's own words: fasm2frames writes it anyway (or drops it above the
            // frame). Counted here; --strict refuses it.
            let own = f::seg_in_window(bit, shift, words);
            let pos = f::seg_pos(off, bit, at);
            let below = f::seg_below(off, bit, at);
            let inside = f::pos_in_frame(pos);
            if !own {
                self.foreign += 1;
                if self.first_foreign.is_none() { self.first_foreign = Some(format!("{line} (tile {tile})")); }
            }
            // A tile's own bit can still leave the frame, but only when the tile starts below it.
            // On any other tile such a bit is left uncounted on purpose: it then breaks
            // outside-the-tile == wrapped + dropped, which is how a wrong window shows up.
            let starts_below = k[6] > 0;
            let own_out = starts_below && own && (below || !inside);
            if own_out {
                self.own_out += 1;
                if self.first_foreign.is_none() { self.first_foreign = Some(format!("{line} (tile {tile})")); }
            }
            if below { self.wrapped += 1; }
            if !inside { self.dropped += 1; continue; }
            let want = if set { f::BIT_SET } else { f::BIT_CLEAR };
            let slot = self.bits.entry((base + minor, below, pos)).or_insert(f::BIT_NONE);
            let now = f::bit_merge(*slot, want);
            let conflict = now == f::BIT_CONFLICT;
            if conflict {
                return Err(format!("FasmInconsistentBits: line '{line}' at frame 0x{:08X} bit {pos}", base + minor));
            }
            *slot = now;
        }
        Ok(())
    }

    // add_fasm_line: canonical_features, then enable_feature per set bit.
    fn add(&mut self, line: &str) -> Result<(), String> {
        let Some((feature, start, end, value)) = parse_fasm_line(line)? else { return Ok(()) };
        let nonzero = value.iter().any(|&b| b);
        if nonzero { self.set_features.push(feature.clone()); }
        let (tile, rest) = feature.split_once('.').unwrap_or((&feature, ""));
        let addrs: Vec<u32> = match (start, end) {
            (None, _) | (Some(_), None) => if nonzero { vec![start.unwrap_or(0)] } else { vec![] },
            (Some(s), Some(e)) => (s..=e).filter(|&a| value.get((a - s) as usize).copied().unwrap_or(false)).collect(),
        };
        for address in addrs {
            match self.enable(tile, rest, address, line.trim()) {
                Ok(()) => {}
                Err(e) if e.starts_with("FasmInconsistentBits") => return Err(e),
                Err(e) => self.missing.push(e),
            }
        }
        Ok(())
    }
}

fn hex8(out: &mut Vec<u8>, v: u32) {
    const H: &[u8; 16] = b"0123456789ABCDEF";
    out.extend_from_slice(b"0x");
    for k in (0..8).rev() { out.push(H[(v >> (4 * k) & 15) as usize]); }
}

fn fasm_frames(a: &[String]) -> i32 {
    let t0 = std::time::Instant::now();
    let db = read_db(&std::fs::read_to_string(&a[1]).unwrap());
    let t_db = t0.elapsed();
    let text = std::fs::read_to_string(&a[2]).unwrap();
    let mut asm = Asm { db: &db, bits: HashMap::new(), missing: vec![], set_features: vec![],
                        wrapped: 0, dropped: 0, foreign: 0, first_foreign: None, own_out: 0, ppip_lines: 0 };
    let strict = a.iter().any(|s| s == "--strict");
    let fail = |e: String| { eprintln!("{e}"); 1 };
    for line in text.lines().chain(db.required.iter().map(|s| s.as_str())) {
        if let Err(e) = asm.add(line) { return fail(e); }
    }
    if !asm.missing.is_empty() { return fail(asm.missing.join("\n")); }
    // STEPDOWN (fasm2frames.py:233-283): a bank with one STEPDOWN feature gets it on every unused
    // IOB site and on its HCLK_IOI3 tile.
    let (mut used, mut tags): (HashSet<(String, String)>, BTreeMap<String, Vec<String>>) = (HashSet::new(), BTreeMap::new());
    for feat in &asm.set_features {
        let p: Vec<&str> = feat.splitn(3, '.').collect();
        if p.len() < 3 { continue; }
        if p[0].contains("IOB33") { used.insert((p[0].to_string(), p[1].to_string())); }
        if p[2].contains("STEPDOWN") {
            let Some(bank) = db.tile_bank.get(p[0]) else { return fail(format!("STEPDOWN in {} with no bank", p[0])) };
            let v = tags.entry(bank.clone()).or_default();
            if !v.contains(&p[2].to_string()) { v.push(p[2].to_string()); }
        }
    }
    let mut extra = vec![];
    for (bank, tg) in &tags {
        for tile in db.bank_tiles.get(bank).map(|v| v.as_slice()).unwrap_or(&[]) {
            if tile.contains("IOB33") {
                for site in db.tiles.get(tile).map(|t| t.iob.as_slice()).unwrap_or(&[]) {
                    if used.contains(&(tile.clone(), site.clone())) { continue; }
                    for tag in tg { extra.push(format!("{tile}.{site}.{tag}")); }
                }
            }
            if tile.contains("HCLK_IOI3") { extra.push(format!("{tile}.STEPDOWN")); }
        }
    }
    for line in &extra {
        if let Err(e) = asm.add(line) { return fail(e); }
    }
    if !asm.missing.is_empty() { return fail(asm.missing.join("\n")); }
    let strays = asm.foreign + asm.own_out;
    if strict && strays > 0 {
        return fail(format!("--strict: {} bit(s) outside their tile's own words or their frame, first from line '{}'",
                            strays, asm.first_foreign.as_deref().unwrap_or("")));
    }
    // Non-sparse: every frame of every tile, zero, then the set bits.
    let mut frames: BTreeMap<u32, [u32; 101]> = BTreeMap::new();
    for t in db.tiles.values() {
        for k in &t.blocks {
            for m in 0..k[2] { frames.entry(k[1] + m).or_insert([0; 101]); }
        }
    }
    for (&(fr, _, pos), &v) in &asm.bits {
        let fw = frames.entry(fr).or_insert([0; 101]);
        let set = v == f::BIT_SET;
        if set { fw[f::pos_word(pos) as usize] |= 1 << f::pos_bit(pos); }
    }
    let mut out = Vec::with_capacity(frames.len() * 1120);
    for (addr, words) in &frames {
        hex8(&mut out, *addr);
        out.push(b' ');
        for (i, w) in words.iter().enumerate() {
            if i > 0 { out.push(b','); }
            hex8(&mut out, *w);
        }
        out.push(b'\n');
    }
    std::fs::write(&a[3], &out).unwrap();
    println!("{}: {} frames, {} bits, required {}, stepdown {}, ppip lines {}, outside the tile {} (wrapped below the frame {}, dropped above it {}), own bits outside the frame {} (db {:.1} ms, total {:.1} ms)",
             a[3], frames.len(), asm.bits.len(), db.required.len(), extra.len(), asm.ppip_lines, asm.foreign, asm.wrapped, asm.dropped, asm.own_out,
             t_db.as_secs_f64() * 1e3, t0.elapsed().as_secs_f64() * 1e3);
    0
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.first().map(|s| s.as_str()) == Some("--cor0") {
        let v: u32 = a[1].parse().expect("OSCFSEL 0..63");
        let reseal = !a.iter().any(|s| s == "--no-reseal");
        let files: Vec<&String> = a[2..].iter().filter(|s| !s.starts_with("--")).collect();
        let mut b = std::fs::read(files[0]).unwrap();
        let r = walk(&mut b, Some(v), reseal, 2);
        std::fs::write(files[1], &b).unwrap();
        println!("patched {} word(s), reseal={reseal}", r.patched);
        return;
    }
    if a.first().map(|s| s.as_str()) == Some("--fasm") {
        std::process::exit(fasm_frames(&a));
    }
    if a.first().map(|s| s.as_str()) == Some("--write") {
        let rejected = write_bit(&a);
        std::process::exit(if rejected == 0 { 0 } else { 1 });
    }
    if a.first().map(|s| s.as_str()) == Some("--frames") {
        let mut b = std::fs::read(&a[1]).unwrap();
        let r = walk(&mut b, None, false, 2);
        let part = r.part();
        assert!(part != w::PARTS, "IDCODE not in far.t27's part table");
        let addrs = walk_addresses(part);
        let mut text = String::new();
        let mut n = 0;
        for (fr, addr) in r.frames.iter().zip(addrs.iter()) {
            let data = fr.iter().any(|&v| v != 0);
            if let (true, Some(addr)) = (data, addr) {
                let ws: Vec<String> = fr.iter().map(|v| format!("0x{v:08X}")).collect();
                text.push_str(&format!("0x{addr:08X} {}\n", ws.join(",")));
                n += 1;
            }
        }
        std::fs::write(&a[2], text).unwrap();
        println!("{}: {} frames with data", a[2], n);
        return;
    }
    if a.first().map(|s| s.as_str()) == Some("--pins") {
        let tiles = std::fs::read_to_string(&a[1]).unwrap();
        let mut b = std::fs::read(&a[2]).unwrap();
        let r = walk(&mut b, None, false, 2);
        let (mut hit, mut total) = (0, 0);
        for line in tiles.lines().filter(|l| !l.trim().is_empty()) {
            let t: Vec<&str> = line.split_whitespace().collect();
            let n: Vec<u32> = t[2..6].iter().map(|x| x.parse().unwrap()).collect();
            let (base, nfr, off, nw) = (n[0], n[1], n[2] as usize, n[3] as usize);
            let data = (0..nfr).map(|k| w::fdri_index(r.part(), base + k)).any(|i| {
                i != w::NO_FRAME && r.frames.get(i as usize).map_or(false, |fr| fr[off..off + nw].iter().any(|&v| v != 0))
            });
            total += 1;
            if data { hit += 1; } else { println!("  MISS {} {} base 0x{:08X}", t[0], t[1], base); }
        }
        println!("{} pins {}/{} land in frames with data", a[2], hit, total);
        return;
    }
    if a.first().map(|s| s.as_str()) == Some("--bits") {
        let segs = std::fs::read_to_string(&a[1]).unwrap();
        let mut b = std::fs::read(&a[2]).unwrap();
        let r = walk(&mut b, None, false, 2);
        let part = r.part();
        let mut bits: HashMap<&str, Vec<Vec<u32>>> = HashMap::new();
        let (mut known, mut window) = (HashSet::new(), HashSet::new());
        for line in segs.lines() {
            let t: Vec<&str> = line.split_whitespace().collect();
            let n: Vec<u32> = t[2..].iter().map(|x| x.parse().unwrap()).collect();
            if t[0] == "S" {
                let by_minor = bits.entry(t[1]).or_default();
                if by_minor.len() <= n[0] as usize { by_minor.resize(n[0] as usize + 1, vec![]); }
                by_minor[n[0] as usize].push(n[1]);
                continue;
            }
            let (base, nfr, off, nw, shift) = (n[0], n[1], n[2], n[3], n[4]);
            let (no_type, no_bits): (Vec<Vec<u32>>, Vec<u32>) = (vec![], vec![]);
            let by_minor = bits.get(t[1]).unwrap_or(&no_type);
            for k in 0..nfr {
                let i = w::fdri_index(part, base + k);
                let fr = match r.frames.get(i as usize) { Some(fr) if i != w::NO_FRAME => fr, _ => continue };
                let quiet = fr[off as usize..(off + nw) as usize].iter().all(|&v| v == 0);
                if quiet { continue; }
                for wd in off..off + nw { window.insert((i, wd)); }
                for &bit in by_minor.get(k as usize).unwrap_or(&no_bits) {
                    let inside = f::seg_in_window(bit, shift, nw);
                    if inside { known.insert((i, f::seg_pos(off, bit, shift))); }
                }
            }
        }
        let (mut total, mut unknown, mut outside) = (0u32, 0u32, 0u32);
        for (i, fr) in r.frames.iter().enumerate() {
            for (wd, &v0) in fr.iter().enumerate() {
                let mut v = if wd as u32 == f::ECC_WORD { v0 & f::ECC_KEEP } else { v0 };
                while v != 0 {
                    let bit = v.trailing_zeros();
                    v &= v - 1;
                    total += 1;
                    let at = (i as u32, wd as u32);
                    if !window.contains(&at) { outside += 1; continue; }
                    if known.contains(&(at.0, at.1 * 32 + bit)) { continue; }
                    unknown += 1;
                    if unknown <= 5 { println!("  UNKNOWN frame {} word {} bit {}", at.0, at.1, bit); }
                }
            }
        }
        println!("{} bits {}/{} named by segbits (unknown {}, outside every tile {})",
                 a[2], total - unknown - outside, total, unknown, outside);
        return;
    }
    if a.first().map(|s| s.as_str()) == Some("--frame") {
        let mut b = std::fs::read(&a[1]).unwrap();
        let min_nz = a.get(2).map_or(2, |s| s.parse().expect("MIN nonzero words"));
        let r = walk(&mut b, None, false, min_nz);
        match r.sparse {
            Some((n, nz, stored)) => {
                println!("{} frame {} of the FDRI stream, stored ECC 0x{:04X}", a[1], n, stored);
                for (k, v) in nz { println!("  word {:3} = 0x{:08X}", k, v); }
            }
            None => println!("{}: no frame with a nonzero ECC", a[1]),
        }
        return;
    }
    for path in &a {
        let mut b = std::fs::read(path).unwrap();
        let t = std::time::Instant::now();
        let r = walk(&mut b, None, false, 2);
        report(path, &r);
        println!("  walked in {:.1} ms", t.elapsed().as_secs_f64() * 1e3);
    }
}

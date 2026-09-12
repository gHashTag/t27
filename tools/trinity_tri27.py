#!/usr/bin/env python3
"""The TRI-27 machine of gHashTag/trinity: golden programs, negatives and loader vectors
replayed through the generated C of three specs.

WHY THIS EXISTS
---------------
S05 of gHashTag/trinity#988 (gHashTag/t27#3567): the consumer's TRI-27 layer has one decoder,
one executor and one loader that its tests drive (src/tri27/emu/decoder.zig, executor.zig,
loader.zig at the pin), and five documents, two assemblers and an emitted template that each
describe something else. specs/isa/ternary_encoding.t27, specs/isa/tri27_machine.t27 and
specs/isa/tri27_bytecode.t27 state what the code does, rule by rule, as functions the C backend
executes. A rule is not a program: this tool composes the rules into a reference interpreter and
runs programs through it, so that the acceptance criteria of S05 -- golden programs with matching
results and bounded traces, and invalid opcodes, memory, stack and budget failing deterministically
with a nonzero status -- are measured, not asserted.

WHAT IS COMPARED WITH WHAT
--------------------------
  model   a Python statement of decoder.zig, executor.zig and loader.zig as read at the pin: the
          instruction word, the forty-seven opcodes, the register file, the byte view of memory,
          the fetch at pc * 4, both entry profiles (CPUState.init + memcpy as the tests and
          `tri tri27 run` do; loader.load as tri_emu_main does), flags, the modular and fixed-point
          arithmetic, CALL/RET, the silent halts and the budget. `vectors` assembles the golden
          programs with the model's assembler (the layout of decoder.encode; tri_asm.zig's twelve-byte
          header), runs them, and writes conformance/trinity/tri27_programs.json with the final
          state, a bounded trace and the status of every vector.
  spec    the three specs generated to C by `t27c gen-c` and linked into a driver this tool writes;
          the driver owns the arrays and the loops (memory, registers, fetch, dispatch), every
          decision -- field extraction, opcode validity, arithmetic, flags, bounds, stack, budget,
          jumps, container checks -- is a call into the generated functions. `run` replays every
          vector and records the verdict, the compiler and the spec hashes.
The status numbers are the machine spec's own: the owner's error set has no ordinals and three of
its halts are silent; STATUS_HALTED is 0 and every other status is nonzero. What is NOT compared:
the owner's Zig. The emulator has no build target, its standalone binaries do not compile, and no
Zig on this host builds it (spec: findings); the reading is checked against the owner's own
test expectations where they exist (test_comprehensive.zig, test_golden.zig) and the record says
which vectors those are.

Usage:
  python3 tools/trinity_tri27.py vectors [--out conformance/trinity/tri27_programs.json]
  python3 tools/trinity_tri27.py run     [--report conformance/trinity/tri27_programs.json]
  python3 tools/trinity_tri27.py check   [--report conformance/trinity/tri27_programs.json]
  python3 tools/trinity_tri27.py --self-check

Exit codes: 0 no finding; 1 findings; 2 could not run (no t27c, no cc, no spec, no report).
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import pathlib
import platform
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC_ENC = ROOT / "specs/isa/ternary_encoding.t27"
SPEC_MACH = ROOT / "specs/isa/tri27_machine.t27"
SPEC_TBIN = ROOT / "specs/isa/tri27_bytecode.t27"
REPORT = ROOT / "conformance/trinity/tri27_programs.json"
CONST_RE = re.compile(r"^pub const (\w+) : ([^=]+?) = (.*);\s*$")
INT_TYPES = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize"}
TRACE_LIMIT = 32

# ---------------------------------------------------------------------------
# The owner as read at 976df517: numbers the model states on its own.
# ---------------------------------------------------------------------------
OPS = {"NOP": 0x00, "LD": 0x02, "ST": 0x03, "LDI": 0x04, "STI": 0x05, "MOV": 0x0F, "ADD": 0x10, "SUB": 0x11, "MUL": 0x12, "DIV": 0x13,
       "INC": 0x14, "DEC": 0x15, "EXP": 0x16, "SIN": 0x17, "AND": 0x18, "OR": 0x19, "XOR": 0x1A, "NOT": 0x1B, "SHL": 0x1C, "SHR": 0x1D,
       "STR_LOAD": 0x20, "STR_CONCAT": 0x21, "STR_PRINT": 0x22, "FILE_READ": 0x23, "FILE_WRITE": 0x24, "FILE_EXISTS": 0x25,
       "JMP": 0x40, "JZ": 0x41, "JNZ": 0x42, "CALL": 0x43, "JGT": 0x44, "JLT": 0x45, "RET": 0x4B, "HALT": 0x4D,
       "DOT": 0x60, "BIND": 0x61, "BUNDLE2": 0x62, "BUNDLE3": 0x63, "PHI_CONST": 0x80, "PI_CONST": 0x81, "E_CONST": 0x82, "SACR": 0x83,
       "LD_IMM": 0x84, "ADD3": 0x85, "SUB3": 0x86, "CMP3": 0x87, "SYSCALL": 0x88}
NAMES = {v: k for k, v in OPS.items()}
IMM_FORM = {"LD", "ST", "LDI", "STI", "LD_IMM", "PHI_CONST", "PI_CONST", "E_CONST", "JMP", "JZ", "JNZ", "JGT", "JLT", "CALL", "RET",
            "SHL", "SHR", "STR_LOAD", "STR_CONCAT", "STR_PRINT", "FILE_READ", "FILE_WRITE", "FILE_EXISTS"}
SRC2_FORM = {"ADD", "SUB", "MUL", "DIV", "AND", "OR", "XOR"}
HOST_IO = {"STR_LOAD", "STR_CONCAT", "STR_PRINT", "FILE_READ", "FILE_WRITE", "FILE_EXISTS"}
MODEL = {"MEMORY_WORDS": 19683, "WORD_BYTES": 8, "MEMORY_BYTES": 157464, "MAX_INSTRUCTIONS": 100000, "MODULUS": 19683,
         "PHI_SCALED": 9842, "PI_SCALED": 19088, "E_SCALED": 16514, "ENTRY_PC_INIT": 3, "ENTRY_SP_INIT": 0, "ENTRY_PC_LOADER": 0,
         "ENTRY_SP_LOADER": 19682, "MAGIC": 0x54524932, "VERSION": 1, "CODE_START_WORD": 3, "IMM_MIN": -16384, "IMM_MAX": 16383}
ST = {"halted": 0, "InvalidRegister": 1, "InvalidMemory": 2, "DivisionByZero": 3, "StackOverflow": 4, "StackUnderflow": 5, "InvalidOpcode": 6,
      "Halted": 7, "fetch out of range": 8, "budget exhausted": 9, "RET underflow": 10, "host I/O not replayed": 11}
STN = {v: k for k, v in ST.items()}
LOAD = {"ok": 0, "Truncated": 1, "InvalidMagic": 2, "InvalidVersion": 3, "InvalidSection": 4, "DataTooLarge": 5, "SectionMissing": 6}
LOADN = {v: k for k, v in LOAD.items()}
I64_MAX = (1 << 63) - 1


def sha256(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def run(cmd: list[str], cwd=None, timeout: int = 300) -> tuple[int, str, str]:
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout, p.stderr
    except (OSError, subprocess.TimeoutExpired) as e:
        return 127, "", str(e)


def t27c_path() -> pathlib.Path | None:
    for p in ("target/release/t27c", "target/debug/t27c"):
        if (ROOT / p).exists():
            return ROOT / p
    w = shutil.which("t27c")
    return pathlib.Path(w) if w else None


def tool_version(cmd: list[str]) -> str | None:
    rc, out, err = run(cmd, timeout=60)
    text = (out or err).strip().split("\n")[0] if (out or err) else ""
    return text[:120] if rc == 0 and text else None


def parse_value(raw: str, typ: str):
    raw = raw.strip()
    if typ == "bool":
        return raw == "true"
    if typ in INT_TYPES:
        return int(raw)
    if typ == "str":
        return json.loads(raw)
    if typ == "f64":
        return float(raw)
    m = re.fullmatch(r"\[(\d+)\](\w+)", typ)
    if m:
        if m.group(2) == "str":
            items = json.loads("[" + raw[1:-1] + "]")
        elif m.group(2) == "bool":
            items = [x.strip() == "true" for x in raw[1:-1].split(",") if x.strip()]
        else:
            items = [int(x) for x in raw[1:-1].split(",") if x.strip()]
        if len(items) != int(m.group(1)):
            raise ValueError(f"annotated {typ}, holds {len(items)}")
        return items
    raise ValueError(f"unknown type {typ}")


def load_spec(path: pathlib.Path, required: list[str]) -> dict:
    text = path.read_text(encoding="utf-8")
    if re.search(r"[^\x00-\x7f]", text):
        raise ValueError(f"{path}: non-ASCII byte (L3)")
    f = {}
    for n, line in enumerate(text.split("\n"), 1):
        m = CONST_RE.match(line)
        if m:
            f[m.group(1)] = parse_value(m.group(3), m.group(2).strip())
        elif line.startswith("pub const"):
            raise ValueError(f"{path}:{n}: cannot read constant line")
    missing = [k for k in required if k not in f]
    if missing:
        raise ValueError(f"{path}: missing {', '.join(missing)}")
    return f


def load_specs(enc=SPEC_ENC, mach=SPEC_MACH, tbin=SPEC_TBIN) -> dict:
    e = load_spec(enc, ["OPCODE_NAMES", "OPCODE_VALUES", "IMM_FORM", "SRC2_FORM", "IMM_MIN", "IMM_MAX"])
    m = load_spec(mach, ["MEMORY_WORDS", "WORD_BYTES", "MEMORY_BYTES", "MAX_INSTRUCTIONS", "MODULUS", "PHI_SCALED", "PI_SCALED", "E_SCALED",
                         "ENTRY_PC_INIT", "ENTRY_SP_INIT", "ENTRY_PC_LOADER", "ENTRY_SP_LOADER", "STATUS_NAMES", "STATUS_HALTED"])
    b = load_spec(tbin, ["MAGIC", "VERSION", "CODE_START_WORD", "LOAD_PC", "LOAD_SP", "LOAD_STATUS_NAMES", "WRITER_HEADER"])
    return {"enc": e, "mach": m, "tbin": b}


def spec_findings(s: dict) -> list[str]:
    out = []
    e, m, b = s["enc"], s["mach"], s["tbin"]
    table = dict(zip(e["OPCODE_NAMES"], e["OPCODE_VALUES"]))
    if table != OPS:
        out.append("the opcode table of the encoding spec differs from the model's reading of decoder.zig")
    if set(e["IMM_FORM"]) != {OPS[n] for n in IMM_FORM} or set(e["SRC2_FORM"]) != {OPS[n] for n in SRC2_FORM}:
        out.append("IMM_FORM or SRC2_FORM differs from the model's reading of decoder.zig")
    for k in ("MEMORY_WORDS", "WORD_BYTES", "MEMORY_BYTES", "MAX_INSTRUCTIONS", "MODULUS", "PHI_SCALED", "PI_SCALED", "E_SCALED",
              "ENTRY_PC_INIT", "ENTRY_SP_INIT", "ENTRY_PC_LOADER", "ENTRY_SP_LOADER"):
        if m[k] != MODEL[k]:
            out.append(f"{k}: the machine spec says {m[k]}, the model reads {MODEL[k]}")
    if e["IMM_MIN"] != MODEL["IMM_MIN"] or e["IMM_MAX"] != MODEL["IMM_MAX"]:
        out.append("the immediate range of the encoding spec differs from the model")
    if b["MAGIC"] != MODEL["MAGIC"] or b["VERSION"] != MODEL["VERSION"] or b["CODE_START_WORD"] != MODEL["CODE_START_WORD"]:
        out.append("the container constants of the bytecode spec differ from the model")
    if b["LOAD_PC"] != MODEL["ENTRY_PC_LOADER"] or b["LOAD_SP"] != MODEL["ENTRY_SP_LOADER"]:
        out.append("the loader's entry point differs between the bytecode spec and the machine spec")
    if list(m["STATUS_NAMES"]) != [STN[i] for i in range(len(STN))]:
        out.append("STATUS_NAMES of the machine spec differ from the model's numbering")
    if list(b["LOAD_STATUS_NAMES"]) != [LOADN[i] for i in range(len(LOADN))]:
        out.append("LOAD_STATUS_NAMES of the bytecode spec differ from the model's numbering")
    return out


# ---------------------------------------------------------------------------
# The model.
# ---------------------------------------------------------------------------
def decode(word: int) -> dict:
    op = word & 0xFF
    name = NAMES.get(op, "NOP")
    has_imm, has_src2 = name in IMM_FORM, name in SRC2_FORM
    dst = (word >> 8) & 0x1F
    src1 = (word >> 13) & (0x0F if has_imm else 0x1F)
    src2 = (word >> 18) & 0x1F if has_src2 else 0
    imm = 0
    if has_imm:
        raw = (word >> 17) & 0x7FFF
        imm = raw - 32768 if raw & 0x4000 else raw
    cond = 0
    if name == "BUNDLE3":
        src2 = (word >> 18) & 0x1F
        cond = (word >> 23) & 0x1F
    return {"op": name, "dst": dst, "src1": src1, "src2": src2, "imm": imm, "cond": cond}


def encode(name: str, dst: int = 0, src1: int = 0, src2: int = 0, imm: int = 0) -> int:
    word = OPS[name] | ((dst & 0x1F) << 8)
    if name in IMM_FORM:
        c = max(MODEL["IMM_MIN"], min(MODEL["IMM_MAX"], imm))
        return word | ((src1 & 0x0F) << 13) | (((c + 65536) & 0x7FFF) << 17)
    if name in SRC2_FORM:
        return word | ((src1 & 0x1F) << 13) | ((src2 & 0x1F) << 18)
    return word | ((src1 & 0x1F) << 13)


def bundle3_word(dst: int, src1: int, src2: int, v3: int) -> int:
    return OPS["BUNDLE3"] | ((dst & 0x1F) << 8) | ((src1 & 0x1F) << 13) | ((src2 & 0x1F) << 18) | ((v3 & 0x1F) << 23)


def divt(a: int, b: int) -> int:
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b >= 0) else -q


def modp(a: int, m: int) -> int:
    return a % m


class Machine:
    def __init__(self):
        self.t = [0] * 27
        self.mem = bytearray(MODEL["MEMORY_BYTES"])
        self.pc, self.sp, self.fp = MODEL["ENTRY_PC_INIT"], MODEL["ENTRY_SP_INIT"], 0
        self.Z = self.N = self.V = self.H = False
        self.executed = 0
        self.status = None
        self.trace = []

    def rd32(self, a: int) -> int:
        return int.from_bytes(self.mem[a:a + 4], "little")

    def wr32(self, a: int, v: int):
        self.mem[a:a + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")

    def set(self, dst: int, r: int):
        if not -I64_MAX - 1 <= r <= I64_MAX:
            raise OverflowError("the model does not follow an i64 overflow (a panic in the owner)")
        self.t[dst] = r
        self.Z, self.N = r == 0, r < 0

    def entry_cli(self, data: bytes):
        self.mem[0:len(data)] = data
        self.pc, self.sp, self.fp = MODEL["ENTRY_PC_INIT"], MODEL["ENTRY_SP_INIT"], 0

    def load_tbin(self, code: bytes) -> int:
        n = len(code)
        if n < 6:
            return LOAD["Truncated"]
        if int.from_bytes(code[0:4], "little") != MODEL["MAGIC"]:
            return LOAD["InvalidMagic"]
        if code[4] != MODEL["VERSION"]:
            return LOAD["InvalidVersion"]
        count, offset, code_size = code[5], 6, 0
        for _ in range(count):
            if offset + 1 > n:
                return LOAD["Truncated"]
            kind = code[offset]
            if kind == 1:
                if offset + 4 > n:
                    return LOAD["Truncated"]
                size = code[offset + 2] | (code[offset + 3] << 8)
                offset += 4
                if offset + size > n:
                    return LOAD["Truncated"]
                if offset + size > MODEL["MEMORY_WORDS"]:
                    return LOAD["DataTooLarge"]
                payload = code[offset:offset + size]
                for i in range(size // 4):
                    w = int.from_bytes(payload[4 * i:4 * i + 4], "little")
                    self.mem[(3 + i) * 8:(3 + i) * 8 + 8] = w.to_bytes(8, "little")
                code_size = size
                offset += size
            elif kind == 2:
                if offset + 1 > n:
                    return LOAD["Truncated"]
                cnt = code[offset]
                offset += 1
                if offset + cnt * 8 > n:
                    return LOAD["Truncated"]
                offset += cnt * 8
            elif kind == 3:
                if offset + 3 > n:
                    return LOAD["Truncated"]
                size = code[offset + 1] | (code[offset + 2] << 8)
                offset += 3
                if offset + size > n:
                    return LOAD["Truncated"]
                if code_size + size > MODEL["MEMORY_WORDS"]:
                    return LOAD["DataTooLarge"]
                for i in range(size):
                    idx = code_size + i
                    self.mem[idx * 8:idx * 8 + 8] = code[offset + i].to_bytes(8, "little")
                offset += size
            elif kind == 4:
                if offset + 3 > n:
                    return LOAD["Truncated"]
                offset += 3
            else:
                return LOAD["InvalidSection"]
        self.pc, self.sp, self.fp = MODEL["ENTRY_PC_LOADER"], MODEL["ENTRY_SP_LOADER"], 0
        if code_size == 0:
            return LOAD["SectionMissing"]
        return LOAD["ok"]

    def execute(self, i: dict):
        if i["dst"] >= 27 or i["src1"] >= 27 or i["src2"] >= 27 or i["cond"] >= 27:
            return ST["InvalidRegister"]
        self.executed += 1
        op, dst, s1, s2, imm, t = i["op"], i["dst"], i["src1"], i["src2"], i["imm"], self.t
        M, MB, MW = MODEL["MODULUS"], MODEL["MEMORY_BYTES"], MODEL["MEMORY_WORDS"]
        if op == "NOP" or op == "SYSCALL":
            self.pc += 1
        elif op == "LDI":
            self.set(dst, imm); self.pc += 1
        elif op == "LD_IMM":
            self.set(dst, max(-1, min(1, imm))); self.pc += 1
        elif op == "ST":
            addr = abs(imm)
            if addr + 4 > MB:
                return ST["InvalidMemory"]
            self.wr32(addr, t[dst] & 0xFFFFFFFF); self.pc += 1
        elif op == "LD":
            addr = abs(imm) * 4
            if addr + 4 > MB:
                return ST["InvalidMemory"]
            w = self.rd32(addr)
            self.set(dst, w - (1 << 32) if w & 0x80000000 else w); self.pc += 1
        elif op == "MOV":
            self.set(dst, t[s1]); self.pc += 1
        elif op in ("ADD", "SUB", "MUL"):
            a, b = t[s1], t[s2]
            self.set(dst, a + b if op == "ADD" else a - b if op == "SUB" else a * b); self.pc += 1
        elif op == "DIV":
            if t[s2] == 0:
                return ST["DivisionByZero"]
            self.set(dst, divt(t[s1], t[s2])); self.pc += 1
        elif op == "INC":
            self.set(dst, t[dst] + 1); self.pc += 1
        elif op == "DEC":
            self.set(dst, t[dst] - 1); self.pc += 1
        elif op == "EXP":
            xs = t[s1] * 1000; x2 = divt(xs * xs, 1000); x3 = divt(x2 * xs, 1000); x4 = divt(x3 * xs, 1000)
            self.set(dst, 1000 + xs + divt(x2, 2) + divt(x3, 6) + divt(x4, 24)); self.pc += 1
        elif op == "SIN":
            xs = t[s1] * 1000; x2 = divt(xs * xs, 1000); x3 = divt(x2 * xs, 1000); x5 = divt(x3 * x2, 1000); x7 = divt(x5 * x2, 1000)
            self.set(dst, xs - divt(x3, 6) + divt(x5, 120) - divt(x7, 5040)); self.pc += 1
        elif op == "AND":
            self.set(dst, t[s1] & t[s2]); self.pc += 1
        elif op == "OR":
            a, b = t[s1], t[s2]
            self.set(dst, a if (a > 0 or b > 0) else (b if (a < 0 and b < 0) else a)); self.pc += 1
        elif op == "XOR":
            a, b = t[s1], t[s2]
            self.set(dst, a if (a != 0 and b != 0) else (b if (a == 0 or b == 0) else a)); self.pc += 1
        elif op == "NOT":
            self.set(dst, ~t[dst]); self.pc += 1
        elif op == "SHL":
            n = abs(imm)
            if n >= 64:
                raise OverflowError("a shift of 64 or more is a panic in the owner")
            self.set(dst, t[s1] << n); self.pc += 1
        elif op == "SHR":
            self.set(dst, t[s1] >> min(abs(imm), 63)); self.pc += 1
        elif op == "DOT":
            self.set(dst, modp(t[s1] * t[s2], M)); self.pc += 1
        elif op == "BIND":
            a, b = t[s1], t[s2]
            self.set(dst, b if a == 0 else (a if b == 0 else modp(a + b, M))); self.pc += 1
        elif op == "BUNDLE2":
            a, b = t[s1], t[s2]
            self.set(dst, b if a == 0 else (a if b == 0 else divt(a + b, 2))); self.pc += 1
        elif op == "BUNDLE3":
            a, b, c = t[s1], t[s2], t[i["cond"]]
            self.set(dst, a if a == b else (a if a == c else (b if b == c else a))); self.pc += 1
        elif op in ("PHI_CONST", "PI_CONST", "E_CONST"):
            t[dst] = {"PHI_CONST": MODEL["PHI_SCALED"], "PI_CONST": MODEL["PI_SCALED"], "E_CONST": MODEL["E_SCALED"]}[op]
            self.Z = self.N = False; self.pc += 1
        elif op == "SACR":
            a, b, mode = t[s1], t[dst], imm
            if mode == 3 and b == 0:
                return ST["DivisionByZero"]
            r = modp(a * b, M) if mode == 2 else divt(a, b) if mode == 3 else modp(a ** b, M) if mode == 4 else modp(a + b, M)
            self.set(dst, r); self.pc += 1
        elif op == "JMP":
            self.pc = abs(imm)
        elif op in ("JZ", "JNZ"):
            taken = (t[dst] == 0) if op == "JZ" else (t[dst] != 0)
            self.pc = abs(imm) if taken else self.pc + 1
        elif op in ("JGT", "JLT"):
            taken = (t[dst] > t[s1]) if op == "JGT" else (t[dst] < t[s1])
            self.pc = abs(imm) if taken else self.pc + 1
        elif op == "CALL":
            if self.sp + 4 > MW:
                return ST["StackOverflow"]
            self.wr32(self.sp, self.pc); self.sp += 4; self.pc = abs(imm)
        elif op == "RET":
            if self.sp < 4:
                self.H = True
                return ST["RET underflow"]
            self.sp -= 4; self.pc = self.rd32(self.sp)
        elif op == "HALT":
            self.H = True
        elif op == "STI":
            addr = abs(imm)
            if addr + 4 > MW:
                return ST["InvalidMemory"]
            self.wr32(addr, max(-1, min(1, imm)) & 0xFFFFFFFF); self.pc += 1
        elif op in HOST_IO:
            return ST["host I/O not replayed"]
        else:
            return ST["InvalidOpcode"]
        return None

    def run(self) -> int:
        while not self.H:
            if self.executed >= MODEL["MAX_INSTRUCTIONS"]:
                self.H = True
                self.status = ST["budget exhausted"]
                break
            if self.pc * 4 + 4 > MODEL["MEMORY_BYTES"]:
                self.H = True
                self.status = ST["fetch out of range"]
                break
            inst = decode(self.rd32(self.pc * 4))
            if len(self.trace) < TRACE_LIMIT:
                self.trace.append(f"{self.pc}:{inst['op']}")
            st = self.execute(inst)
            if st is not None:
                self.status = st
                break
        if self.status is None:
            self.status = ST["halted"]
        return self.status


# ---------------------------------------------------------------------------
# The assembler of the model (decoder.encode's layout, tri_asm.zig's container).
# ---------------------------------------------------------------------------
FORMS = {"NOP": "none", "HALT": "none", "RET": "none", "SYSCALL": "none", "ADD3": "none", "SUB3": "none", "CMP3": "none",
         "STR_PRINT": "dst", "STR_LOAD": "dst_imm", "STR_CONCAT": "dst_src1", "FILE_READ": "dst_src1", "FILE_WRITE": "dst_src1", "FILE_EXISTS": "dst",
         "JMP": "imm", "CALL": "imm", "LDI": "dst_imm", "LD_IMM": "dst_imm", "LD": "dst_imm", "ST": "dst_imm", "JZ": "dst_imm", "JNZ": "dst_imm",
         "PHI_CONST": "dst", "PI_CONST": "dst", "E_CONST": "dst", "NOT": "dst", "INC": "dst", "DEC": "dst", "STI": "sti",
         "JGT": "dst_src1_imm", "JLT": "dst_src1_imm", "SHL": "dst_src1_imm", "SHR": "dst_src1_imm", "SACR": "dst_src1_imm",
         "ADD": "dst_src1_src2", "SUB": "dst_src1_src2", "MUL": "dst_src1_src2", "DIV": "dst_src1_src2", "AND": "dst_src1_src2", "OR": "dst_src1_src2", "XOR": "dst_src1_src2",
         "MOV": "dst_src1", "EXP": "dst_src1", "SIN": "dst_src1", "DOT": "dst_src1", "BIND": "dst_src1", "BUNDLE2": "dst_src1", "BUNDLE3": "bundle3"}


def assemble(lines: list[str]) -> list[int]:
    labels, idx, items = {}, MODEL["CODE_START_WORD"], []
    for raw in lines:
        line = raw.split(";")[0].strip()
        if not line:
            continue
        if line.endswith(":"):
            labels[line[:-1].strip()] = idx
            continue
        items.append(line)
        idx += 1

    def reg(tok: str) -> int:
        if not re.fullmatch(r"t\d+", tok):
            raise ValueError(f"not a register: {tok}")
        return int(tok[1:])

    def val(tok: str) -> int:
        if tok in labels:
            return labels[tok]
        if tok.startswith("0x"):
            return int(tok, 16)
        return int(tok)

    words = []
    for line in items:
        parts = line.replace(",", " ").split()
        mn, args = parts[0].upper(), parts[1:]
        if mn == ".RAW":
            words.append(val(args[0]) & 0xFFFFFFFF)
            continue
        form = FORMS[mn]
        if form == "none":
            words.append(encode(mn))
        elif form == "imm":
            words.append(encode(mn, imm=val(args[0])))
        elif form == "dst":
            words.append(encode(mn, dst=reg(args[0])))
        elif form == "dst_imm":
            words.append(encode(mn, dst=reg(args[0]), imm=val(args[1])))
        elif form == "sti":
            words.append(encode(mn, imm=val(args[0])))
        elif form == "dst_src1_imm":
            words.append(encode(mn, dst=reg(args[0]), src1=reg(args[1]), imm=val(args[2])))
        elif form == "dst_src1_src2":
            words.append(encode(mn, dst=reg(args[0]), src1=reg(args[1]), src2=reg(args[2])))
        elif form == "dst_src1":
            words.append(encode(mn, dst=reg(args[0]), src1=reg(args[1])))
        elif form == "bundle3":
            words.append(bundle3_word(reg(args[0]), reg(args[1]), reg(args[2]), reg(args[3])))
    return words


def container(words: list[int]) -> bytes:
    """tri_asm.zig's container: a twelve-byte header (two padding bytes the loader reads as code)."""
    code = b"".join(w.to_bytes(4, "little") for w in words)
    return bytes([0x32, 0x49, 0x52, 0x54, 0x01, 0x01, 0x01, 0x00, len(code) & 0xFF, (len(code) >> 8) & 0xFF, 0x00, 0x00]) + code


def container_native(words: list[int], before: bytes = b"", after: bytes = b"", count: int = 1) -> bytes:
    """A container as loader.load reads it: six-byte header, four-byte CODE header, the code; no writer in the tree produces it."""
    code = b"".join(w.to_bytes(4, "little") for w in words)
    return bytes([0x32, 0x49, 0x52, 0x54, 0x01, count]) + before + bytes([0x01, 0x00, len(code) & 0xFF, (len(code) >> 8) & 0xFF]) + code + after


# ---------------------------------------------------------------------------
# The vectors.
# ---------------------------------------------------------------------------
def state_of(m: Machine, before: bytes | None = None) -> dict:
    out = {"status": m.status, "status_name": STN[m.status], "executed": m.executed, "pc": m.pc, "sp": m.sp,
           "flags": {"Z": m.Z, "N": m.N, "H": m.H}, "registers": {f"t{i}": v for i, v in enumerate(m.t) if v != 0}}
    if before is not None:
        out["memory"] = [[a, m.mem[a]] for a in range(len(m.mem)) if m.mem[a] != before[a]]
    return out


def program_vector(vid: str, entry: str, asm: list[str], note: str, owner_test: str = "") -> dict:
    words = assemble(asm)
    tbin = container(words) if entry == "cli" else container_native(words)
    m = Machine()
    load_status = None
    if entry == "cli":
        m.entry_cli(tbin)
        before = bytes(m.mem)
    else:
        load_status = m.load_tbin(tbin)
        before = bytes(m.mem)
        if load_status != 0:
            raise ValueError(f"{vid}: the container is rejected ({LOADN[load_status]})")
    m.run()
    v = {"id": vid, "kind": "program", "entry": entry, "container": "tri_asm.zig (twelve-byte header)" if entry == "cli" else "loader-native (ten-byte header)", "asm": asm, "words": [f"0x{w:08x}" for w in words], "tbin": tbin.hex(), "note": note,
         "expect": state_of(m, before), "trace": list(m.trace), "trace_limit": TRACE_LIMIT}
    if load_status is not None:
        v["expect"]["load_status"] = load_status
    if owner_test:
        v["owner_test"] = owner_test
    return v


def loader_vector(vid: str, data: bytes, note: str) -> dict:
    m = Machine()
    before = bytes(m.mem)
    st = m.load_tbin(data)
    exp = {"load_status": st, "load_status_name": LOADN[st]}
    if st == 0:
        exp.update({"pc": m.pc, "sp": m.sp, "memory": [[a, m.mem[a]] for a in range(len(m.mem)) if m.mem[a] != before[a]]})
    return {"id": vid, "kind": "loader", "tbin": data.hex(), "note": note, "expect": exp}


def build_vectors() -> list[dict]:
    V = []
    P = lambda *a, **k: V.append(program_vector(*a, **k))
    P("halt-only", "cli", ["HALT"], "HALT sets H and leaves pc where it is", "executor.zig `execute HALT`")
    P("ldi-add", "cli", ["LDI t1, 5", "LDI t2, 7", "ADD t3, t1, t2", "HALT"], "plain i64 addition, Z and N from the result", "test_golden.zig `full cycle` (LDI, ADD, HALT)")
    P("ldi-negative", "cli", ["LDI t1, -5", "LDI t2, -16384", "LDI t3, 16383", "SUB t4, t1, t2", "HALT"], "the fifteen-bit immediate at both ends")
    P("countdown", "cli", ["LDI t1, 5", "loop:", "DEC t1", "JNZ t1, loop", "HALT"], "a loop through JNZ; the label counts from word 3")
    P("jumps", "cli", ["LDI t1, 2", "LDI t2, 3", "JGT t1, t2, skip1", "INC t3", "skip1:", "JLT t1, t2, skip2", "INC t3", "skip2:", "JZ t3, done", "INC t4", "done:", "HALT"],
      "JGT not taken, JLT taken, JZ not taken", "test_comprehensive.zig `Control Flow - JZ/JNZ` (JZ, JNZ only)")
    P("mov-inc-dec-mul-div", "cli", ["LDI t1, 7", "MOV t2, t1", "INC t2", "DEC t1", "MUL t3, t1, t2", "LDI t4, -2", "DIV t5, t3, t4", "HALT"], "MOV copies, INC and DEC act on dst, DIV truncates toward zero",
      "test_comprehensive.zig `Arithmetic - MUL`, `DIV`, `INC`, `DEC`")
    P("logic-shift", "cli", ["LDI t1, -1", "LDI t2, 1", "AND t3, t1, t2", "OR t4, t1, t2", "XOR t5, t1, t2", "MOV t6, t2", "NOT t6", "LDI t7, 3", "SHL t8, t7, 2", "SHR t9, t7, 1", "HALT"],
      "AND is bitwise, OR and XOR are the owner's ternary-shaped rules, NOT complements dst", "test_comprehensive.zig `Logical - AND/OR/XOR/NOT`, `Bitwise - SHL/SHR`")
    P("vsa-scalar", "cli", ["LDI t0, 200", "LDI t1, 100", "DOT t3, t1", "BIND t4, t1", "BUNDLE2 t5, t1", "LDI t0, 0", "BIND t7, t1", "LDI t9, -5", "BIND t10, t9", "BUNDLE3 t8, t1, t4, t4", "HALT"],
      "DOT, BIND and BUNDLE2 read their second operand from t0 because the word cannot carry src2; a zero operand is BIND's identity even for a negative value; BUNDLE3 carries three registers in the raw word",
      "test_comprehensive.zig `Ternary - DOT/BIND/BUNDLE2/BUNDLE3` (Instruction values built directly)")
    P("sacred", "cli", ["PHI_CONST t1", "PI_CONST t2", "E_CONST t3", "LDI t4, 3", "SACR t4, t1, 2", "HALT"],
      "the three scaled constants clear Z and N; the decoded SACR adds modulo 19683 whatever mode the source wrote", "test_comprehensive.zig `Sacred - PHI_CONST/PI_CONST/E_CONST/SACR add`")
    P("transcendental", "cli", ["LDI t1, 1", "EXP t2, t1", "SIN t3, t1", "LDI t4, -1", "EXP t5, t4", "SIN t6, t4", "HALT"], "fixed-point Taylor series scaled by 1000", "smoke_tests.zig `transcendental: EXP`, `SIN`")
    P("memory", "cli", ["LDI t1, 1234", "ST t1, 100", "LD t2, 25", "STI 8", "LD t3, 2", "LDI t4, -1", "ST t4, 200", "LD t5, 50", "HALT"],
      "ST writes the low 32 bits at a byte address, LD reads a word index times four and sign-extends, STI stores a clamped trit at byte |imm|", "test_comprehensive.zig `Memory - LD`")
    P("syscall-is-a-nop", "cli", ["LDI t1, 1", "SYSCALL", "INC t1", "HALT"], "SYSCALL advances pc and does nothing")
    P("call-never-returns", "cli", ["JMP main", "sub:", "INC t5", "RET", "main:", "LDI t5, 0", "CALL sub", "HALT"],
      "CALL pushes its own address and RET returns to it: the pair is an endless loop with sp alternating between 4 and 8, stopped by the budget after 33333 rounds; nonzero status")
    P("budget", "cli", ["loop:", "JMP loop"], "an endless jump stops at 100000 instructions; the owner reports a plain halt, the replay a nonzero status")
    P("fetch-out-of-range", "cli", ["LDI t1, 16383", "SHL t1, t1, 4", "CALL sub", "sub:", "ST t1, 0", "RET"],
      "RET pops a return address that ST overwrote with 262128; the fetch at byte 1048512 is past memory and the run stops silently in the owner, nonzero here")
    P("ret-underflow", "cli", ["RET"], "RET with sp < 4 sets H and returns no error in the owner; nonzero here")
    P("invalid-register", "cli", [".raw 0x00081b10", "HALT"], "ADD with dst 27 (a five-bit field can name 27..31) is refused before it counts as executed")
    P("invalid-opcode", "cli", ["ADD3"], "ADD3, SUB3, CMP3 are declared and fall through to InvalidOpcode")
    P("division-by-zero", "cli", ["LDI t1, 5", "LDI t2, 0", "DIV t3, t1, t2", "HALT"], "DIV by a zero register")
    P("host-io-not-replayed", "cli", ["STR_PRINT t0"], "the string and file opcodes are not replayed; the replay stops with its own status")
    P("loader-entry", "loader", ["LDI t1, 5", "HALT"], "a loader-native container: after loader.load the code sits in eight-byte words from word 3 and pc is 0, so six zero fetches run as NOP, then LDI at pc 6, a zero fetch at 7, then HALT at 8")
    P("loader-entry-call", "loader", ["CALL 6"], "a loader-native container: after loader.load sp is 19682 and the first CALL (at pc 6) overflows the stack guard")
    good = container(assemble(["LDI t1, 5", "HALT"]))
    L = lambda *a: V.append(loader_vector(*a))
    L("load-good", good, "the writer's twelve-byte container is accepted, pc 0, sp 19682, but the loader takes its code from byte 10: the two padding bytes lead the first word and every word is shifted by two bytes (bytes 26, 27, 32 and 34 change instead of 24, 25, 26 and 32)")
    native = container_native(assemble(["LDI t1, 5", "HALT"]))
    L("load-native", native, "the ten-byte container the loader expects: the two words land whole at words 3 and 4")
    L("load-truncated-header", good[:5], "five bytes: Truncated before the magic is read")
    L("load-bad-magic", bytes([0x54, 0x52, 0x49, 0x32]) + good[4:], "the magic in the other byte order: InvalidMagic")
    L("load-version-2", good[:4] + bytes([2]) + good[5:], "version 2: InvalidVersion")
    L("load-version-0", good[:4] + bytes([0]) + good[5:], "version 0: InvalidVersion")
    L("load-unknown-section", good[:6] + bytes([9]) + good[7:], "section id 9: InvalidSection")
    L("load-truncated-section-header", good[:7], "a section id with no header behind it: Truncated")
    L("load-truncated-code", good[:14], "the code header promises eight bytes and two follow: Truncated")
    L("load-no-sections", good[:5] + bytes([0]), "zero sections: SectionMissing, decided after pc and sp are set")
    L("load-bss-only", good[:5] + bytes([1, 4, 8, 0]), "a BSS section and no code: SectionMissing")
    big = bytes([0x32, 0x49, 0x52, 0x54, 0x01, 0x01, 0x01, 0x00, 0xE6, 0x4C]) + bytes(19686)
    L("load-code-too-large", big, "19686 code bytes at offset 10: 19696 exceeds the 19683 words the check compares against: DataTooLarge")
    fit = bytes([0x32, 0x49, 0x52, 0x54, 0x01, 0x01, 0x01, 0x00, 0xD9, 0x4C]) + bytes(19673)
    L("load-code-largest-that-fits", fit, "19673 code bytes at offset 10 fit exactly; 4918 words land from word 3")
    words = assemble(["LDI t1, 5", "HALT"])
    L("load-constants-and-bss", container_native(words, before=bytes([2]) + bytes(16) + bytes([4, 2, 0]), count=3), "a CONSTANTS section (the id byte read as the count 2, sixteen bytes skipped) and a BSS section before the code: accepted, cpu.f untouched")
    L("load-constants-documented-layout", container_native(words, before=bytes([2, 1]) + bytes(8), count=2), "a CONSTANTS section written as the documents say (id, count 1, eight bytes): the loader skips sixteen bytes from the count byte and lands inside the code that follows")
    L("load-data-section", container_native(words, after=bytes([3, 3, 0, 0xAA, 0xBB, 0xCC]), count=2), "three data bytes land one per word from word index 8, the code size in bytes")
    L("load-data-truncated", container_native(words, after=bytes([3, 5, 0, 0xAA]), count=2), "the data header promises five bytes and one follows: Truncated")
    code9 = b"".join(w.to_bytes(4, "little") for w in words) + bytes([0x4D])
    L("load-partial-word", bytes([0x32, 0x49, 0x52, 0x54, 0x01, 0x01, 0x01, 0x00, 9, 0]) + code9, "nine code bytes in a loader-native container: two words are packed, the ninth byte is dropped")
    return V


def build_report(s: dict) -> dict:
    return {
        "version": 1,
        "generated_by": "tools/trinity_tri27.py vectors",
        "format_family": "Conformance",
        "vector_name": "Trinity TRI-27 golden programs, negatives and loader vectors",
        "specs": {"encoding": "specs/isa/ternary_encoding.t27", "machine": "specs/isa/tri27_machine.t27", "bytecode": "specs/isa/tri27_bytecode.t27"},
        "consumer": {"repo": "gHashTag/trinity", "pinned_revision": s["mach"]["PINNED_REVISION"], "owner": ["src/tri27/emu/decoder.zig", "src/tri27/emu/executor.zig", "src/tri27/emu/cpu_state.zig", "src/tri27/emu/loader.zig", "src/tri27/emu/tri_asm.zig"]},
        "description": "Golden programs assembled in the layout of decoder.encode into tri_asm.zig's twelve-byte container and run by a Python model of executor.zig; negatives for an invalid register, an invalid opcode, division by zero, the stack guard, a fetch past memory, RET underflow and the budget; loader vectors for every rejection rule of loader.zig. Replayed through the generated C of the three specs by `run`.",
        "entry_profiles": {"cli": "the file at byte 0, pc 3, sp 0 (CPUState.init + memcpy: test_golden.zig, `tri tri27 run`)", "loader": "loader.load: code in eight-byte words from word 3, pc 0, sp 19682 (tri_emu_main.zig, no build target)"},
        "status_codes": {k: v for k, v in ST.items()},
        "load_status_codes": {k: v for k, v in LOAD.items()},
        "model": {k: v for k, v in MODEL.items()},
        "trace_limit": TRACE_LIMIT,
        "not_replayed": "STR_LOAD, STR_CONCAT, STR_PRINT, FILE_READ, FILE_WRITE, FILE_EXISTS (host I/O by opcode); cpu.f and the vector registers (never read by an executed opcode); SACR modes 2..4 (unreachable through the decoder)",
        "vectors": build_vectors(),
    }


# ---------------------------------------------------------------------------
# The replay: three specs generated to C, one driver.
# ---------------------------------------------------------------------------
def prototypes(c_text: str) -> list[str]:
    out = []
    for line in c_text.split("\n"):
        if re.match(r"^(?:u?int\d+_t|bool|double|float|void|const char\s*\*)\s+[a-z_]\w*\(.*\);\s*$", line):
            out.append(line.strip())
    return out


DRIVER_HEAD = r"""
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
@PROTOTYPES@
#define MEM_BYTES @MEM_BYTES@
#define MEM_WORDS @MEM_WORDS@
static const char *OPNAME[256];
typedef struct { int64_t t[27]; uint8_t mem[MEM_BYTES]; uint32_t pc, sp, fp; bool Z, N, H; uint32_t executed; int status; char trace[@TRACE_LIMIT@][24]; int ntrace; } M;
static M machine;
static uint32_t rd32(M *m, uint32_t a) { return (uint32_t)m->mem[a] | ((uint32_t)m->mem[a + 1] << 8) | ((uint32_t)m->mem[a + 2] << 16) | ((uint32_t)m->mem[a + 3] << 24); }
static void wr32(M *m, uint32_t a, uint32_t v) { m->mem[a] = v & 0xFF; m->mem[a + 1] = (v >> 8) & 0xFF; m->mem[a + 2] = (v >> 16) & 0xFF; m->mem[a + 3] = (v >> 24) & 0xFF; }
static void wr64(M *m, uint32_t w, uint64_t v) { for (int i = 0; i < 8; i++) m->mem[w * 8 + i] = (uint8_t)(v >> (8 * i)); }
static void reset(M *m) { memset(m, 0, sizeof(*m)); m->pc = @ENTRY_PC_INIT@; m->sp = @ENTRY_SP_INIT@; m->status = -1; }
static void setreg(M *m, uint32_t dst, int64_t r) { m->t[dst] = r; m->Z = flag_z(r); m->N = flag_n(r); }
static uint32_t load_tbin(M *m, const uint8_t *d, uint32_t len) {
    uint32_t st = header_status(len, len > 0 ? d[0] : 0, len > 1 ? d[1] : 0, len > 2 ? d[2] : 0, len > 3 ? d[3] : 0, len > 4 ? d[4] : 0);
    if (st != @LOAD_OK@) return st;
    uint32_t count = d[5], offset = 6, code_size = 0;
    for (uint32_t k = 0; k < count; k++) {
        if (!fits_file(offset, 1, len)) return @LOAD_TRUNCATED@;
        uint32_t id = d[offset];
        if (!section_kind_ok(id)) return @LOAD_INVALID_SECTION@;
        if (id == @SECTION_CODE@) {
            if (!fits_file(offset, section_header_bytes(id), len)) return @LOAD_TRUNCATED@;
            uint32_t size = size_field(d[offset + 2], d[offset + 3]);
            offset += section_header_bytes(id);
            if (!fits_file(offset, size, len)) return @LOAD_TRUNCATED@;
            if (!code_fits_memory(offset, size, MEM_WORDS)) return @LOAD_DATA_TOO_LARGE@;
            for (uint32_t i = 0; i < code_words(size); i++) wr64(m, code_word_index(i), (uint64_t)((uint32_t)d[offset + 4 * i] | ((uint32_t)d[offset + 4 * i + 1] << 8) | ((uint32_t)d[offset + 4 * i + 2] << 16) | ((uint32_t)d[offset + 4 * i + 3] << 24)));
            code_size = size;
            offset += size;
        } else if (id == @SECTION_CONSTANTS@) {
            if (!fits_file(offset, section_header_bytes(id), len)) return @LOAD_TRUNCATED@;
            uint32_t cnt = d[offset];
            offset += section_header_bytes(id);
            if (!fits_file(offset, constants_payload(cnt), len)) return @LOAD_TRUNCATED@;
            offset += constants_payload(cnt);
        } else if (id == @SECTION_DATA@) {
            if (!fits_file(offset, section_header_bytes(id), len)) return @LOAD_TRUNCATED@;
            uint32_t size = size_field(d[offset + 1], d[offset + 2]);
            offset += section_header_bytes(id);
            if (!fits_file(offset, size, len)) return @LOAD_TRUNCATED@;
            if (!data_fits_memory(code_size, size, MEM_WORDS)) return @LOAD_DATA_TOO_LARGE@;
            for (uint32_t i = 0; i < size; i++) wr64(m, data_word_index(code_size, i), d[offset + i]);
            offset += size;
        } else {
            if (!fits_file(offset, section_header_bytes(id), len)) return @LOAD_TRUNCATED@;
            offset += section_header_bytes(id);
        }
    }
    m->pc = @LOAD_PC@; m->sp = @LOAD_SP@; m->fp = 0;
    if (!code_present(code_size)) return @LOAD_SECTION_MISSING@;
    return @LOAD_OK@;
}
static int exec_one(M *m, uint32_t word) {
    uint32_t op = decode_opcode(opcode_of(word));
    uint32_t dst = dst_of(word), s1 = src1_of(word, op), s2 = src2_of(word, op), v3 = v3_of(word, op);
    int32_t imm = imm_of(word, op);
    if (!registers_valid(dst, s1, s2, v3)) return @ST_INVALID_REGISTER@;
    m->executed++;
    int64_t *t = m->t;
    switch (op) {
    case @OP_NOP@: case @OP_SYSCALL@: m->pc++; break;
    case @OP_LDI@: setreg(m, dst, (int64_t)imm); m->pc++; break;
    case @OP_LD_IMM@: setreg(m, dst, clamp_trit((int64_t)imm)); m->pc++; break;
    case @OP_ST@: if (!st_in_range(imm, MEM_BYTES)) return @ST_INVALID_MEMORY@; wr32(m, abs_imm(imm), st_low32(t[dst])); m->pc++; break;
    case @OP_LD@: if (!ld_in_range(imm, MEM_BYTES)) return @ST_INVALID_MEMORY@; setreg(m, dst, ld_sign_extend(rd32(m, ld_byte_addr(imm)))); m->pc++; break;
    case @OP_MOV@: setreg(m, dst, t[s1]); m->pc++; break;
    case @OP_ADD@: setreg(m, dst, alu_add(t[s1], t[s2])); m->pc++; break;
    case @OP_SUB@: setreg(m, dst, alu_sub(t[s1], t[s2])); m->pc++; break;
    case @OP_MUL@: setreg(m, dst, alu_mul(t[s1], t[s2])); m->pc++; break;
    case @OP_DIV@: if (!alu_div_defined(t[s2])) return @ST_DIVISION_BY_ZERO@; setreg(m, dst, alu_div(t[s1], t[s2])); m->pc++; break;
    case @OP_INC@: setreg(m, dst, alu_inc(t[dst])); m->pc++; break;
    case @OP_DEC@: setreg(m, dst, alu_dec(t[dst])); m->pc++; break;
    case @OP_EXP@: setreg(m, dst, exp_fixed(t[s1])); m->pc++; break;
    case @OP_SIN@: setreg(m, dst, sin_fixed(t[s1])); m->pc++; break;
    case @OP_AND@: setreg(m, dst, and_bits(t[s1], t[s2])); m->pc++; break;
    case @OP_OR@: setreg(m, dst, or_ternary(t[s1], t[s2])); m->pc++; break;
    case @OP_XOR@: setreg(m, dst, xor_ternary(t[s1], t[s2])); m->pc++; break;
    case @OP_NOT@: setreg(m, dst, not_bits(t[dst])); m->pc++; break;
    case @OP_SHL@: setreg(m, dst, shl_bits(t[s1], abs_imm(imm))); m->pc++; break;
    case @OP_SHR@: setreg(m, dst, shr_bits(t[s1], abs_imm(imm))); m->pc++; break;
    case @OP_DOT@: setreg(m, dst, dot_mod(t[s1], t[s2])); m->pc++; break;
    case @OP_BIND@: setreg(m, dst, bind_mod(t[s1], t[s2])); m->pc++; break;
    case @OP_BUNDLE2@: setreg(m, dst, bundle2_avg(t[s1], t[s2])); m->pc++; break;
    case @OP_BUNDLE3@: setreg(m, dst, bundle3_majority(t[s1], t[s2], t[v3])); m->pc++; break;
    case @OP_PHI_CONST@: t[dst] = @PHI_SCALED@; m->Z = false; m->N = false; m->pc++; break;
    case @OP_PI_CONST@: t[dst] = @PI_SCALED@; m->Z = false; m->N = false; m->pc++; break;
    case @OP_E_CONST@: t[dst] = @E_SCALED@; m->Z = false; m->N = false; m->pc++; break;
    case @OP_SACR@: if (!sacr_defined(imm, t[dst])) return @ST_DIVISION_BY_ZERO@; setreg(m, dst, sacr(imm, t[s1], t[dst])); m->pc++; break;
    case @OP_JMP@: m->pc = abs_imm(imm); break;
    case @OP_JZ@: m->pc = taken_z(t[dst]) ? abs_imm(imm) : m->pc + 1; break;
    case @OP_JNZ@: m->pc = taken_nz(t[dst]) ? abs_imm(imm) : m->pc + 1; break;
    case @OP_JGT@: m->pc = taken_gt(t[dst], t[s1]) ? abs_imm(imm) : m->pc + 1; break;
    case @OP_JLT@: m->pc = taken_lt(t[dst], t[s1]) ? abs_imm(imm) : m->pc + 1; break;
    case @OP_CALL@: if (call_overflows(m->sp, MEM_WORDS)) return @ST_STACK_OVERFLOW@; wr32(m, m->sp, m->pc); m->sp += 4; m->pc = abs_imm(imm); break;
    case @OP_RET@: if (ret_underflows(m->sp)) { m->H = true; return @ST_RET_UNDERFLOW@; } m->sp -= 4; m->pc = rd32(m, m->sp); break;
    case @OP_HALT@: m->H = true; break;
    case @OP_STI@: if (!sti_in_range(imm, MEM_WORDS)) return @ST_INVALID_MEMORY@; wr32(m, abs_imm(imm), (uint32_t)(int32_t)sti_value(imm)); m->pc++; break;
    case @OP_STR_LOAD@: case @OP_STR_CONCAT@: case @OP_STR_PRINT@: case @OP_FILE_READ@: case @OP_FILE_WRITE@: case @OP_FILE_EXISTS@: return @ST_HOST_IO@;
    default: return @ST_INVALID_OPCODE@;
    }
    return -1;
}
static void run_machine(M *m) {
    while (!m->H) {
        if (budget_exhausted(m->executed)) { m->H = true; m->status = @ST_BUDGET@; break; }
        if (!fetch_in_range(m->pc, MEM_BYTES)) { m->H = true; m->status = @ST_FETCH@; break; }
        uint32_t word = rd32(m, m->pc * 4);
        if (m->ntrace < @TRACE_LIMIT@) { snprintf(m->trace[m->ntrace], 24, "%u:%s", m->pc, OPNAME[decode_opcode(opcode_of(word))]); m->ntrace++; }
        int st = exec_one(m, word);
        if (st >= 0) { m->status = st; break; }
    }
    if (m->status < 0) m->status = @ST_HALTED@;
}
static int passes = 0, failures = 0;
static void verdict(const char *vid, const char *fail) {
    if (fail) { printf("[VEC] %s : FAILED (%s)\n", vid, fail); failures++; } else { printf("[VEC] %s : PASSED\n", vid); passes++; }
}
int main(void) {
    for (int i = 0; i < 256; i++) OPNAME[i] = "NOP";
@OPNAMES@
    static char msg[256];
"""
DRIVER_TAIL = r"""
    printf("vectors: %d passed, %d failed\n", passes, failures);
    return failures ? 1 : 0;
}
"""


def c_bytes(name: str, data: bytes) -> str:
    body = ", ".join(str(b) for b in data) if data else "0"
    return f"static const uint8_t {name}[] = {{{body}}}; const uint32_t {name}_n = {len(data)};"


def driver_block(i: int, v: dict) -> str:
    vid = json.dumps(v["id"])
    exp = v["expect"]
    data = bytes.fromhex(v["tbin"])
    lines = ["    {", c_bytes(f"d{i}", data), "reset(&machine); M *m = &machine; const char *fail = NULL;"]
    if v["kind"] == "loader":
        lines.append(f"uint32_t ls = load_tbin(m, d{i}, d{i}_n);")
        lines.append(f"if (ls != {exp['load_status']}u) {{ snprintf(msg, sizeof msg, \"load status %u, want {exp['load_status']}\", ls); fail = msg; }}")
        if exp["load_status"] == 0:
            lines.append(f"if (!fail && (m->pc != {exp['pc']}u || m->sp != {exp['sp']}u)) {{ snprintf(msg, sizeof msg, \"pc %u sp %u, want {exp['pc']} {exp['sp']}\", m->pc, m->sp); fail = msg; }}")
            for a, b in exp.get("memory", []):
                lines.append(f"if (!fail && m->mem[{a}] != {b}) {{ snprintf(msg, sizeof msg, \"byte {a} is %u, want {b}\", m->mem[{a}]); fail = msg; }}")
            lines.append(f"if (!fail) {{ uint32_t changed = 0; for (uint32_t a = 0; a < MEM_BYTES; a++) if (m->mem[a] != 0) changed++; if (changed != {len(exp.get('memory', []))}u) {{ snprintf(msg, sizeof msg, \"%u bytes changed, want {len(exp.get('memory', []))}\", changed); fail = msg; }} }}")
        lines.append(f"verdict({vid}, fail);")
    else:
        if v["entry"] == "cli":
            lines.append(f"memcpy(m->mem, d{i}, d{i}_n);")
        else:
            lines.append(f"uint32_t ls = load_tbin(m, d{i}, d{i}_n); if (ls != 0u) {{ snprintf(msg, sizeof msg, \"load status %u\", ls); fail = msg; }}")
        lines.append("static uint8_t before[MEM_BYTES]; memcpy(before, m->mem, MEM_BYTES);")
        lines.append("if (!fail) run_machine(m);")
        lines.append(f"if (!fail && m->status != {exp['status']}) {{ snprintf(msg, sizeof msg, \"status %d, want {exp['status']} ({exp['status_name']})\", m->status); fail = msg; }}")
        lines.append(f"if (!fail && m->executed != {exp['executed']}u) {{ snprintf(msg, sizeof msg, \"executed %u, want {exp['executed']}\", m->executed); fail = msg; }}")
        lines.append(f"if (!fail && (m->pc != {exp['pc']}u || m->sp != {exp['sp']}u)) {{ snprintf(msg, sizeof msg, \"pc %u sp %u, want {exp['pc']} {exp['sp']}\", m->pc, m->sp); fail = msg; }}")
        fl = exp["flags"]
        lines.append(f"if (!fail && (m->Z != {int(fl['Z'])} || m->N != {int(fl['N'])} || m->H != {int(fl['H'])})) {{ snprintf(msg, sizeof msg, \"flags Z%d N%d H%d, want Z{int(fl['Z'])} N{int(fl['N'])} H{int(fl['H'])}\", m->Z, m->N, m->H); fail = msg; }}")
        regs = [0] * 27
        for k, val in exp["registers"].items():
            regs[int(k[1:])] = val
        lines.append("{ static const int64_t want[27] = {" + ", ".join(f"{r}LL" for r in regs) + "}; for (int r = 0; r < 27 && !fail; r++) if (m->t[r] != want[r]) { snprintf(msg, sizeof msg, \"t%d is %lld, want %lld\", r, (long long)m->t[r], (long long)want[r]); fail = msg; } }")
        for a, b in exp.get("memory", []):
            lines.append(f"if (!fail && m->mem[{a}] != {b}) {{ snprintf(msg, sizeof msg, \"byte {a} is %u, want {b}\", m->mem[{a}]); fail = msg; }}")
        lines.append(f"if (!fail) {{ uint32_t changed = 0; for (uint32_t a = 0; a < MEM_BYTES; a++) if (m->mem[a] != before[a]) changed++; if (changed != {len(exp.get('memory', []))}u) {{ snprintf(msg, sizeof msg, \"%u bytes changed, want {len(exp.get('memory', []))}\", changed); fail = msg; }} }}")
        trace = v.get("trace", [])
        lines.append("{ static const char *want_trace[] = {" + (", ".join(json.dumps(x) for x in trace) if trace else "\"\"") + f"}}; int nt = {len(trace)}; if (!fail && m->ntrace != nt) {{ snprintf(msg, sizeof msg, \"trace length %d, want %d\", m->ntrace, nt); fail = msg; }} for (int k = 0; k < nt && !fail; k++) if (strcmp(m->trace[k], want_trace[k]) != 0) {{ snprintf(msg, sizeof msg, \"trace[%d] is %s, want %s\", k, m->trace[k], want_trace[k]); fail = msg; }} }}")
        lines.append(f"verdict({vid}, fail);")
    lines.append("}")
    return "\n    ".join(lines)


def replay(specs: dict[str, pathlib.Path], report: dict, t27c: pathlib.Path, work: pathlib.Path) -> dict:
    work.mkdir(parents=True, exist_ok=True)
    s = load_specs(specs["enc"], specs["mach"], specs["tbin"])
    protos, objects, gen_hashes = [], [], {}
    for key in ("enc", "mach", "tbin"):
        rc, out, err = run([str(t27c), "gen-c", str(specs[key])])
        if rc != 0 or not out.strip():
            return {"ok": False, "stage": "generate", "detail": f"{key}: {(err or out).strip()[:300]}", "passed": 0, "failed": 0, "failures": []}
        src = work / f"{key}.c"
        src.write_text(out, encoding="utf-8")
        gen_hashes[key] = sha256(out.encode())
        protos += prototypes(out)
        rc, out2, err = run(["cc", "-std=c11", "-w", "-c", "-x", "c", str(src), "-o", str(work / f"{key}.o")])
        if rc != 0:
            return {"ok": False, "stage": "compile", "detail": f"{key}: {err.strip()[:400]}", "passed": 0, "failed": 0, "failures": [], "generated": gen_hashes}
        objects.append(str(work / f"{key}.o"))
    e, m, b = s["enc"], s["mach"], s["tbin"]
    subs = {"@PROTOTYPES@": "\n".join(protos), "@MEM_BYTES@": str(m["MEMORY_BYTES"]), "@MEM_WORDS@": str(m["MEMORY_WORDS"]), "@TRACE_LIMIT@": str(TRACE_LIMIT),
            "@ENTRY_PC_INIT@": str(m["ENTRY_PC_INIT"]), "@ENTRY_SP_INIT@": str(m["ENTRY_SP_INIT"]), "@LOAD_PC@": str(b["LOAD_PC"]), "@LOAD_SP@": str(b["LOAD_SP"]),
            "@PHI_SCALED@": str(m["PHI_SCALED"]), "@PI_SCALED@": str(m["PI_SCALED"]), "@E_SCALED@": str(m["E_SCALED"]),
            "@OPNAMES@": "\n".join(f"    OPNAME[{val}] = \"{name}\";" for name, val in zip(e["OPCODE_NAMES"], e["OPCODE_VALUES"]))}
    for name, val in zip(e["OPCODE_NAMES"], e["OPCODE_VALUES"]):
        subs[f"@OP_{name}@"] = str(val)
    for key, sk in (("ST_INVALID_REGISTER", "STATUS_INVALID_REGISTER"), ("ST_INVALID_MEMORY", "STATUS_INVALID_MEMORY"), ("ST_DIVISION_BY_ZERO", "STATUS_DIVISION_BY_ZERO"),
                    ("ST_STACK_OVERFLOW", "STATUS_STACK_OVERFLOW"), ("ST_INVALID_OPCODE", "STATUS_INVALID_OPCODE"), ("ST_FETCH", "STATUS_FETCH_OUT_OF_RANGE"),
                    ("ST_BUDGET", "STATUS_BUDGET_EXHAUSTED"), ("ST_RET_UNDERFLOW", "STATUS_RET_UNDERFLOW"), ("ST_HOST_IO", "STATUS_HOST_IO_NOT_REPLAYED"), ("ST_HALTED", "STATUS_HALTED")):
        subs[f"@{key}@"] = str(m[sk])
    for key in ("LOAD_OK", "LOAD_TRUNCATED", "LOAD_INVALID_SECTION", "LOAD_DATA_TOO_LARGE", "LOAD_SECTION_MISSING", "SECTION_CODE", "SECTION_CONSTANTS", "SECTION_DATA"):
        subs[f"@{key}@"] = str(b[key])
    head = DRIVER_HEAD
    for k, v in subs.items():
        head = head.replace(k, v)
    left = re.findall(r"@[A-Z_]+@", head)
    if left:
        return {"ok": False, "stage": "driver", "detail": f"unfilled: {sorted(set(left))[:5]}", "passed": 0, "failed": 0, "failures": []}
    driver = head + "\n".join(driver_block(i, v) for i, v in enumerate(report["vectors"])) + DRIVER_TAIL
    (work / "driver.c").write_text(driver, encoding="utf-8")
    rc, out, err = run(["cc", "-std=c11", "-w", "-c", "-x", "c", str(work / "driver.c"), "-o", str(work / "driver.o")])
    if rc != 0:
        return {"ok": False, "stage": "compile", "detail": "driver: " + err.strip()[:600], "passed": 0, "failed": 0, "failures": [], "generated": gen_hashes}
    rc, out, err = run(["cc", str(work / "driver.o")] + objects + ["-o", str(work / "driver"), "-lm"])
    if rc != 0:
        return {"ok": False, "stage": "link", "detail": err.strip()[:600], "passed": 0, "failed": 0, "failures": [], "generated": gen_hashes}
    rc, out, err = run([str(work / "driver")], timeout=300)
    results = re.findall(r"^\[VEC\] (.+?) : (PASSED|FAILED)(?: \((.*)\))?$", out, re.M)
    failures = [{"id": vid, "detail": d or ""} for vid, v, d in results if v == "FAILED"]
    passed = sum(1 for _, v, _ in results if v == "PASSED")
    summary = re.search(r"^vectors: (\d+) passed, (\d+) failed$", out, re.M)
    if summary is None:
        return {"ok": False, "stage": "runtime", "detail": f"driver exit {rc}, no summary line: {(err or out)[-300:]}", "passed": passed, "failed": len(failures), "failures": failures[:50], "generated": gen_hashes}
    ok = rc == 0 and not failures and passed == len(report["vectors"])
    return {"ok": ok, "stage": "runtime", "detail": "", "passed": passed, "failed": len(failures), "failures": failures[:50], "generated": gen_hashes}


def replay_record(specs: dict[str, pathlib.Path], report: dict, t27c: pathlib.Path) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        r = replay(specs, report, t27c, pathlib.Path(tmp))
    r["at"] = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    r["t27c"] = tool_version([str(t27c), "--version"]) or "t27c"
    r["cc"] = tool_version(["cc", "--version"]) or "cc"
    r["host"] = f"{platform.system()} {platform.machine()}"
    r["spec_sha256"] = {k: sha256(p.read_bytes()) for k, p in specs.items()}
    r["vectors"] = len(report["vectors"])
    return r


# ---------------------------------------------------------------------------
# check
# ---------------------------------------------------------------------------
def check_report(specs: dict[str, pathlib.Path], report: dict) -> list[str]:
    findings = []
    try:
        s = load_specs(specs["enc"], specs["mach"], specs["tbin"])
    except ValueError as e:
        return [str(e)]
    findings += spec_findings(s)
    fresh = build_report(s)
    for key in ("vectors", "status_codes", "load_status_codes", "model", "entry_profiles", "not_replayed"):
        if json.dumps(report.get(key), sort_keys=True) != json.dumps(fresh[key], sort_keys=True):
            findings.append(f"{key}: the report drifts from what `vectors` writes now")
    ids = [v["id"] for v in report.get("vectors", [])]
    if len(set(ids)) != len(ids):
        findings.append("vectors: duplicate ids")
    kinds = {v["kind"] for v in report.get("vectors", [])}
    if kinds != {"program", "loader"}:
        findings.append("vectors: both kinds must be present")
    nonzero = [v for v in report.get("vectors", []) if v["kind"] == "program" and v["expect"]["status"] != 0]
    if len(nonzero) < 7:
        findings.append("vectors: fewer than seven nonzero-status programs (invalid register, invalid opcode, division, stack, fetch, RET, budget)")
    r = report.get("replay")
    if not isinstance(r, dict):
        findings.append("replay: absent; run `run`")
        return findings
    for k, p in specs.items():
        if r.get("spec_sha256", {}).get(k) != sha256(p.read_bytes()):
            findings.append(f"replay: {p.name} changed since the replay; run `run`")
    if r.get("vectors") != len(report.get("vectors", [])):
        findings.append("replay: the vector count changed since the replay; run `run`")
    if not r.get("ok") or r.get("failed", 1) != 0 or r.get("passed") != len(report.get("vectors", [])):
        findings.append(f"replay: {r.get('passed')} passed, {r.get('failed')} failed at stage {r.get('stage')}: {r.get('detail', '')[:200]}")
    return findings


# ---------------------------------------------------------------------------
# --self-check
# ---------------------------------------------------------------------------
def self_check() -> int:
    t27c = t27c_path()
    if t27c is None or shutil.which("cc") is None:
        print("trinity_tri27 --self-check: needs t27c (target/release/t27c) and cc", file=sys.stderr)
        return 2
    ok = True

    def expect(cond: bool, what: str):
        nonlocal ok
        print(f"  {'ok ' if cond else 'BAD'} {what}")
        ok = ok and cond

    w = encode("LDI", 3, 0, 0, -5)
    d = decode(w)
    expect(w == 0xFFF60304 and d["imm"] == -5 and d["dst"] == 3, "model: the immediate form round trips with sign extension")
    expect(decode(encode("DOT", 3, 1, 2))["src2"] == 0 and decode(bundle3_word(3, 1, 2, 4))["cond"] == 4, "model: DOT loses src2, BUNDLE3 carries a third register")
    m = Machine(); m.entry_cli(container(assemble(["LDI t1, 5", "LDI t2, 7", "ADD t3, t1, t2", "HALT"]))); m.run()
    expect(m.status == 0 and m.t[3] == 12 and m.executed == 4 and m.pc == 6, "model: LDI, ADD, HALT as test_golden.zig expects")
    m = Machine(); expect(m.load_tbin(b"2IRT\x02\x01") == LOAD["InvalidVersion"] and Machine().load_tbin(b"2IRT\x01") == LOAD["Truncated"], "model: the loader rejects in order")
    specs = {"enc": SPEC_ENC, "mach": SPEC_MACH, "tbin": SPEC_TBIN}
    s = load_specs()
    expect(not spec_findings(s), "spec: the constants of the three specs agree with the model")
    report = build_report(s)
    expect(len(report["vectors"]) >= 35 and len({x["id"] for x in report["vectors"]}) == len(report["vectors"]), f"vectors: {len(report['vectors'])} unique ids")
    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        r = replay(specs, report, t27c, work / "good")
        expect(r["ok"] and r["failed"] == 0, f"replay: the specs pass every vector ({r['passed']} passed, {r['failed']} failed{', ' + r['detail'][:200] if r['detail'] else ''})")
        bad = json.loads(json.dumps(report))
        target = next(x for x in bad["vectors"] if x["id"] == "ldi-add")
        target["expect"]["registers"]["t3"] = 13
        r = replay(specs, bad, t27c, work / "bad")
        expect(r["failed"] == 1 and r["failures"][0]["id"] == "ldi-add", f"planted: a wrong expected register fails exactly its vector ({r['failures'][0]['id'] if r['failures'] else 'none'})")
        bad = json.loads(json.dumps(report))
        target = next(x for x in bad["vectors"] if x["id"] == "load-bad-magic")
        target["expect"] = {"load_status": 0, "load_status_name": "ok", "pc": 0, "sp": 19682, "memory": []}
        r = replay(specs, bad, t27c, work / "bad2")
        expect(r["failed"] == 1 and r["failures"][0]["id"] == "load-bad-magic", "planted: a rejection declared as acceptance fails its loader vector")
        text = SPEC_MACH.read_text(encoding="utf-8")
        planted = text.replace("pub fn bind_mod(a: i64, b: i64) -> i64 {\n    if (a == 0) { return b; }\n    if (b == 0) { return a; }\n    return mod_pos(a + b, MODULUS);\n}",
                               "pub fn bind_mod(a: i64, b: i64) -> i64 {\n    return mod_pos(a + b, MODULUS);\n}")
        expect(planted != text, "planted: the bind_mod body was found to replace")
        (work / "spec").mkdir()
        pm = work / "spec" / "tri27_machine.t27"
        pm.write_text(planted, encoding="utf-8")
        r = replay({"enc": SPEC_ENC, "mach": pm, "tbin": SPEC_TBIN}, report, t27c, work / "spec")
        ids = {x["id"] for x in r["failures"]}
        expect(r["failed"] >= 1 and ids == {"vsa-scalar"}, f"planted: a bind without the zero identity fails only the vector that binds a zero to a negative value ({sorted(ids)})")
        text = SPEC_ENC.read_text(encoding="utf-8")
        planted = text.replace("pub const IMM_SIGN_BIT : u32 = 16384;", "pub const IMM_SIGN_BIT : u32 = 8192;")
        pe = work / "spec" / "ternary_encoding.t27"
        pe.write_text(planted, encoding="utf-8")
        r = replay({"enc": pe, "mach": SPEC_MACH, "tbin": SPEC_TBIN}, report, t27c, work / "spec2")
        ids = {x["id"] for x in r["failures"]}
        expect(r["failed"] >= 1 and "ldi-negative" in ids and "halt-only" not in ids, f"planted: a wrong sign bit fails the negative-immediate vectors and not the others ({len(ids)} failing)")
        good = dict(report, replay=dict(replay_record(specs, report, t27c)))
        expect(not check_report(specs, good), "check: a fresh report has no finding")
        drifted = json.loads(json.dumps(good))
        drifted["vectors"][0]["note"] = "edited"
        expect(any("drifts" in x for x in check_report(specs, drifted)), "check: an edited vector is reported as drift")
        stale = json.loads(json.dumps(good))
        stale["replay"]["spec_sha256"]["mach"] = "sha256:0"
        expect(any("changed since the replay" in x for x in check_report(specs, stale)), "check: a replay of another spec text is reported as stale")
        failed = json.loads(json.dumps(good))
        failed["replay"]["failed"] = 1
        failed["replay"]["ok"] = False
        expect(any("failed" in x for x in check_report(specs, failed)), "check: a recorded failure is a finding")
    print("trinity_tri27 --self-check:", "ok" if ok else "FAILED")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", nargs="?", choices=["vectors", "run", "check"])
    ap.add_argument("--out", default=str(REPORT))
    ap.add_argument("--report", default=str(REPORT))
    ap.add_argument("--self-check", action="store_true")
    a = ap.parse_args()
    if a.self_check:
        return self_check()
    if a.command is None:
        ap.print_help()
        return 2
    specs = {"enc": SPEC_ENC, "mach": SPEC_MACH, "tbin": SPEC_TBIN}
    for p in specs.values():
        if not p.exists():
            print(f"no spec at {p}", file=sys.stderr)
            return 2
    try:
        s = load_specs()
    except ValueError as e:
        print(f"finding: {e}")
        return 1
    if a.command == "vectors":
        report = build_report(s)
        out = pathlib.Path(a.out)
        if out.exists():
            old = json.loads(out.read_text(encoding="utf-8"))
            if "replay" in old:
                report["replay"] = old["replay"]
        out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        programs = sum(1 for v in report["vectors"] if v["kind"] == "program")
        print(f"wrote {out}: {len(report['vectors'])} vectors ({programs} programs, {len(report['vectors']) - programs} loader)")
        return 0
    rp = pathlib.Path(a.report)
    if not rp.exists():
        print(f"no report at {rp}; run `vectors` first", file=sys.stderr)
        return 2
    report = json.loads(rp.read_text(encoding="utf-8"))
    if a.command == "run":
        t27c = t27c_path()
        if t27c is None or shutil.which("cc") is None:
            print("run: needs t27c (target/release/t27c) and cc", file=sys.stderr)
            return 2
        report["replay"] = replay_record(specs, report, t27c)
        rp.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        r = report["replay"]
        print(f"replay: {r['passed']} passed, {r['failed']} failed of {r['vectors']} vectors (stage {r['stage']}) with {r['t27c']} and {r['cc']}")
        for x in r["failures"][:20]:
            print(f"  FAILED {x['id']}: {x['detail']}")
        if r["detail"] and not r["ok"]:
            print(f"  {r['detail'][:600]}")
        return 0 if r["ok"] else 1
    findings = check_report(specs, report)
    seen = []
    for x in findings:
        if x not in seen:
            seen.append(x)
    for x in seen:
        print(f"finding: {x}")
    r = report.get("replay", {})
    print(f"check: {len(report.get('vectors', []))} vectors, replay {r.get('passed')}/{r.get('vectors')} passed at {r.get('at')}; {len(seen)} finding(s)")
    return 1 if seen else 0


if __name__ == "__main__":
    sys.exit(main())

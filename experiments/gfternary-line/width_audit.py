"""W772: check the ARITHMETIC of every width expression in emitted Verilog.

W771 lost a wave to a probe that assigned NB-1 bits into an NB-bit register.
T384a prescribed checking the arithmetic of every slice width in the emitted
source; this is that check, made mechanical so it cannot be skipped.

WHAT IT VERIFIES, per line:
  reg/wire [H:L] NAME        -- declared width H-L+1, recorded
  NAME <= {A[h1:l1], B[h2:l2], ...}  -- the concatenation's total width must
                                        EQUAL the target's declared width
  assign NAME = {...}        -- same
  NAME[h:l]                  -- h must be < declared width of NAME, l >= 0

It does NOT understand Verilog; it understands the four shapes this project's
generators emit. A pattern it cannot parse is REPORTED, never silently skipped --
an auditor that quietly ignores what it does not recognise is the W771a failure
in a new costume.
"""
import re, sys

DECL = re.compile(r'^\s*(?:reg|wire)\s+(?:signed\s+)?\[\s*(\d+)\s*:\s*(\d+)\s*\]\s*(\w+)')
DECL1 = re.compile(r'^\s*(?:reg|wire)\s+(?:signed\s+)?(\w+)\s*[=;]')
ASSIGN = re.compile(r'^\s*(?:assign\s+)?(\w+)\s*(?:<=|=)\s*\{(.+)\}\s*;')
SLICE = re.compile(r'(\w+)\s*\[\s*(\d+)\s*:\s*(\d+)\s*\]')
BIT = re.compile(r'(\w+)\s*\[\s*(\d+)\s*\]')
LITERAL = re.compile(r"(\d+)'[sbdh]?[bdh]?[0-9a-fA-FxzXZ_]+")
REPL = re.compile(r'\{\s*(\d+)\s*\{')

def audit(path):
    widths={}; problems=[]; unparsed=[]
    lines=open(path).read().splitlines()
    for ln in lines:
        m=DECL.match(ln)
        if m:
            h,l,nm=int(m.group(1)),int(m.group(2)),m.group(3)
            widths[nm]=h-l+1; continue
        m=DECL1.match(ln)
        if m: widths[m.group(1)]=1
    for i,ln in enumerate(lines,1):
        m=ASSIGN.match(ln)
        if not m: continue
        tgt, body = m.group(1), m.group(2)
        if tgt not in widths: continue
        total=0; ok=True
        rest=body
        for r in REPL.finditer(body):     # {N{expr}} replication -- not parsed
            unparsed.append((i,"replication {N{...}}")); ok=False
        # W776: a ternary conditional contributes ONE branch's width, not both,
        # and its condition contributes nothing. The first version summed both
        # branches plus the condition's operands and reported the WORKING die B
        # as broken -- 46 bits where the line is 32. Caught only because the
        # known-good dyadic build was audited alongside the new one.
        # Report rather than mis-count (lesson 872).
        if "?" in body and ":" in body:
            unparsed.append((i, "ternary ?: -- width not inferred")); ok=False
        if re.search(r'[<>]=?|==|!=', body):
            unparsed.append((i, "comparison in concatenation -- width not inferred")); ok=False
        if not ok: continue
        for sm in SLICE.finditer(rest):
            total += int(sm.group(2))-int(sm.group(3))+1
        rest2 = SLICE.sub("", rest)
        for bm in BIT.finditer(rest2):
            total += 1
        rest3 = BIT.sub("", rest2)
        for lm in LITERAL.finditer(rest3):
            total += int(lm.group(1))
        rest4 = LITERAL.sub("", rest3)
        for w in re.findall(r'\b(\w+)\b', rest4):
            if w in widths: total += widths[w]
        if total != widths[tgt]:
            problems.append((i, tgt, widths[tgt], total, ln.strip()[:88]))
    # slice bounds against declared widths
    for i,ln in enumerate(lines,1):
        for sm in SLICE.finditer(ln):
            nm,h,l=sm.group(1),int(sm.group(2)),int(sm.group(3))
            if nm in widths and h >= widths[nm]:
                problems.append((i,nm,widths[nm],h,f"slice [{h}:{l}] out of range"))
    return widths, problems, unparsed

if __name__=="__main__":
    bad=0
    for path in sys.argv[1:]:
        w,p,u = audit(path)
        nm=path.split("/")[-1]
        print(f"  {nm:<22} {len(w):>4} объявлений, {len(p)} расхождений, {len(u)} не разобрано")
        for i,t,decl,got,src in p:
            print(f"      строка {i}: {t} объявлен {decl} бит, выражение даёт {got} -- {src}")
            bad+=1
        for i,why in u[:3]:
            print(f"      строка {i}: НЕ РАЗОБРАНО ({why}) -- проверить вручную")
    sys.exit(1 if bad else 0)


# ---------------------------------------------------------------------------
# W774: THE CROSS-BOUNDARY CHECK.
#
# T392 cost six waves to a mismatch that lives ON the Verilog/Python boundary:
# the design declared `reg [32:0] sr` (a 33-bit JTAG data register) and the
# driver shifted 32 bits. Neither file was wrong on its own, and an auditor that
# stops at the file boundary cannot see it. This one does not stop there.
#
# The design side: the width of the register wired to BSCANE2's TDO, which is the
# DR length the JTAG state machine will clock.
# The driver side: bits sent per DR pass, summed over MPSSE commands --
#   0x39 / 0x19 <lenlo> <lenhi>  -> (len+1) BYTES  = 8*(len+1) bits
#   0x3B / 0x1B <len> <byte>     -> (len+1) BITS
# ---------------------------------------------------------------------------

DR_REG = re.compile(r'^\s*reg\s+\[\s*(\d+)\s*:\s*0\s*\]\s*(\w+)\s*=')
TDO    = re.compile(r'\.TDO\s*\(\s*(\w+)\s*\[')
BYTECMD = re.compile(r'bytes\(\[\s*0x(?:39|19)\s*,\s*(\d+)\s*,\s*(\d+)\s*\]')
BITCMD  = re.compile(r'bytes\(\[\s*(?:CLK_BITS_IO_NEG|CLK_BITS_OUT_NEG|0x3B|0x1B)\s*,\s*(\d+)\s*,')

def dr_width(verilog_path):
    """The declared width of the register BSCANE2 shifts, or None."""
    txt = open(verilog_path).read()
    m = TDO.search(txt)
    if not m: return None, "no BSCANE2 .TDO(reg[...]) found"
    name = m.group(1)
    for ln in txt.splitlines():
        d = DR_REG.match(ln)
        if d and d.group(2) == name:
            return int(d.group(1)) + 1, name
    return None, f"TDO drives {name} but no `reg [N:0] {name} =` declaration"

def driver_bits(py_path, func):
    """Bits a driver function shifts in one DR pass, counted from MPSSE commands."""
    src = open(py_path).read()
    i = src.find(f"def {func}(")
    if i < 0: return None, f"{func} not found"
    j = src.find("\ndef ", i + 1)
    body = src[i: j if j > 0 else len(src)]
    bits = 0
    for m in BYTECMD.finditer(body):
        bits += 8 * ((int(m.group(1)) | (int(m.group(2)) << 8)) + 1)
    for m in BITCMD.finditer(body):
        bits += int(m.group(1)) + 1
    return bits, None

def cross_check(verilog_path, py_path, func):
    w, note = dr_width(verilog_path)
    b, err = driver_bits(py_path, func)
    vn = verilog_path.split("/")[-1]; pn = py_path.split("/")[-1]
    if w is None: return f"  {vn:<16} DR: НЕ ОПРЕДЕЛЁН ({note})", 1
    if b is None: return f"  {vn:<16} драйвер: {err}", 1
    ok = (w == b)
    tag = "OK" if ok else "!! РАСХОЖДЕНИЕ !!"
    return (f"  {vn:<16} DR={w:>3} бит ({note})   {pn}:{func} шлёт {b:>3} бит   {tag}",
            0 if ok else 1)

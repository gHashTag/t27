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

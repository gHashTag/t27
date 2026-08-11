#!/usr/bin/env python3
"""Make the emitted engine Icarus-friendly WITHOUT changing its logic.

Icarus requires declare-before-use. The emitter writes several declarations
after their first use, which yosys accepts and Icarus does not. This hoists
every module-level `wire`/`reg` declaration to the top of the module, splitting
an initialised wire into a declaration plus an `assign` left in place, so the
logic is untouched.
"""
import pathlib, re, sys

p = pathlib.Path(sys.argv[1])
s = p.read_text()

head = re.search(r"^(module\s+\w+\b.*?\n\);\s*\n)", s, re.M | re.S)
if not head:
    head = re.search(r"^(module\s+\w+\b[^;]*?;\s*\n)", s, re.M | re.S)
assert head, f"{p.name}: could not find the module header"

body = s[head.end():]
decls, out = [], []
# Comma-separated declarations count too: `wire pf_overflow, dma_overflow;`
# and `reg mac_valid_q, mac_first_q, mac_last_q;` are both in this emitter, and
# a single-name pattern silently left them where they were.
DECL = re.compile(
    r"^([ \t]*)(wire|reg)((?:\s+signed)?(?:\s*\[[^\]]*\])?)\s+"
    r"([A-Za-z_]\w*(?:\s*,\s*[A-Za-z_]\w*)*)\s*(=\s*[^;]+)?;[ \t]*$")

for line in body.split("\n"):
    m = DECL.match(line)
    if not m:
        out.append(line)
        continue
    indent, kind, width, name, init = m.groups()
    for one in [x.strip() for x in name.split(",")]:
        decls.append(f"{indent}{kind}{width} {one};")
    if init:
        # Keep the driver where it was, as a continuous assignment.
        out.append(f"{indent}assign {name} {init};")
    # A bare declaration disappears from its old position.

p.write_text(s[:head.end()] + "\n".join(decls) + "\n" + "\n".join(out))
print(f"  {p.name}: hoisted {len(decls)} declarations")

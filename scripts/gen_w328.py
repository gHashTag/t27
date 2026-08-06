#!/usr/bin/env python3
"""Batch append W328 test/invariant blocks to all 27 IGLA specs."""

import re
from pathlib import Path

SPECS = (
    list(Path("specs/igla/race").glob("*.t27")) +
    list(Path("specs/igla/coder").glob("*.t27"))
)

def append_w328(path: Path) -> None:
    text = path.read_text()
    m = list(re.finditer(r'// Wave Loop (\d+).*?\n(invariant .*?true)\n', text, re.DOTALL))
    if not m:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    last = m[-1]
    wave = int(last.group(1))
    if wave != 327:
        print(f"SKIP {path}: last wave is {wave}, expected 327")
        return

    block = last.group(0)
    new_block = block.replace("Wave Loop 327", "Wave Loop 328")
    new_block = new_block.replace("w327_", "w328_")
    def inc_depth(match):
        return f"depth_{int(match.group(1)) + 1:03d}"
    new_block = re.sub(r'depth_(\d+)', inc_depth, new_block)
    def inc_arrow(match):
        x, y = match.group(1), match.group(2)
        return f"({y}→{int(y)+1})"
    new_block = re.sub(r'\((\d+)→(\d+)\)', inc_arrow, new_block)

    insert_pos = last.end()
    new_text = text[:insert_pos] + new_block + text[insert_pos:]
    path.write_text(new_text)
    print(f"APPENDED W328 to {path}")

if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w328(spec)

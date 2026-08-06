#!/usr/bin/env python3
"""Batch append W366 test/invariant blocks to all 27 IGLA specs."""

import re
from pathlib import Path

SPECS = (
    list(Path("specs/igla/race").glob("*.t27")) +
    list(Path("specs/igla/coder").glob("*.t27"))
)


def append_w366(path: Path) -> None:
    text = path.read_text()
    m = list(re.finditer(r'// Wave Loop (\d+).*?\n(invariant .*?true)\n', text, re.DOTALL))
    if not m:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    last = m[-1]
    wave = int(last.group(1))
    if wave != 365:
        print(f"SKIP {path}: last wave is {wave}, expected 365")
        return

    block = last.group(0)
    new_block = block.replace("Wave Loop 365", "Wave Loop 366")
    new_block = new_block.replace("w365_", "w366_")
    new_block = new_block.replace("W364 seal regeneration", "W365 seal regeneration")
    new_block = new_block.replace("after W364", "after W365")
    new_block = new_block.replace("WAVE_LOOP_364_COOPERATION.md", "WAVE_LOOP_365_COOPERATION.md")

    insert_pos = last.end()
    new_text = text[:insert_pos] + new_block + text[insert_pos:]
    path.write_text(new_text)
    print(f"APPENDED W366 to {path}")


if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w366(spec)

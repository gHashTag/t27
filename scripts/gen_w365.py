#!/usr/bin/env python3
"""Batch append W365 test/invariant blocks to all 27 IGLA specs."""

import re
from pathlib import Path

SPECS = (
    list(Path("specs/igla/race").glob("*.t27")) +
    list(Path("specs/igla/coder").glob("*.t27"))
)


def append_w365(path: Path) -> None:
    text = path.read_text()
    m = list(re.finditer(r'// Wave Loop (\d+).*?\n(invariant .*?true)\n', text, re.DOTALL))
    if not m:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    last = m[-1]
    wave = int(last.group(1))
    if wave != 364:
        print(f"SKIP {path}: last wave is {wave}, expected 364")
        return

    block = last.group(0)
    new_block = block.replace("Wave Loop 364", "Wave Loop 365")
    new_block = new_block.replace("w364_", "w365_")
    new_block = new_block.replace("W363 seal regeneration", "W364 seal regeneration")
    new_block = new_block.replace("after W363", "after W364")
    new_block = new_block.replace("WAVE_LOOP_363_COOPERATION.md", "WAVE_LOOP_364_COOPERATION.md")

    insert_pos = last.end()
    new_text = text[:insert_pos] + new_block + text[insert_pos:]
    path.write_text(new_text)
    print(f"APPENDED W365 to {path}")


if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w365(spec)

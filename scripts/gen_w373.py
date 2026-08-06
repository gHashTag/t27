#!/usr/bin/env python3
"""Batch append W373 test/invariant blocks to all 27 IGLA specs."""

import re
from pathlib import Path

SPECS = (
    list(Path("specs/igla/race").glob("*.t27")) +
    list(Path("specs/igla/coder").glob("*.t27"))
)


def append_w373(path: Path) -> None:
    text = path.read_text()
    m = list(re.finditer(r'// Wave Loop (\d+).*?\n(invariant .*?true)\n', text, re.DOTALL))
    if not m:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    last = m[-1]
    wave = int(last.group(1))
    if wave != 372:
        print(f"SKIP {path}: last wave is {wave}, expected 372")
        return

    block = last.group(0)
    new_block = block.replace("Wave Loop 372", "Wave Loop 373")
    new_block = new_block.replace("w372_", "w373_")
    new_block = new_block.replace("W371 seal regeneration", "W372 seal regeneration")
    new_block = new_block.replace("after W371", "after W372")
    new_block = new_block.replace("WAVE_LOOP_371_COOPERATION.md", "WAVE_LOOP_372_COOPERATION.md")

    insert_pos = last.end()
    new_text = text[:insert_pos] + new_block + text[insert_pos:]
    path.write_text(new_text)
    print(f"APPENDED W373 to {path}")


if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w373(spec)

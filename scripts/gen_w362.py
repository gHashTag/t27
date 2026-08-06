#!/usr/bin/env python3
"""Batch append W362 test/invariant blocks to all 27 IGLA specs."""

import re
from pathlib import Path

SPECS = (
    list(Path("specs/igla/race").glob("*.t27")) +
    list(Path("specs/igla/coder").glob("*.t27"))
)

RING = {
    "specs/igla/coder": "coder",
    "specs/igla/race": "race",
}


def append_w362(path: Path) -> None:
    text = path.read_text()
    m = list(re.finditer(r'// Wave Loop (\d+).*?\n(invariant .*?true)\n', text, re.DOTALL))
    if not m:
        print(f"SKIP {path}: no Wave Loop block found")
        return

    last = m[-1]
    wave = int(last.group(1))
    if wave != 361:
        print(f"SKIP {path}: last wave is {wave}, expected 361")
        return

    block = last.group(0)
    new_block = block.replace("Wave Loop 361", "Wave Loop 362")
    new_block = new_block.replace("w361_", "w362_")
    new_block = new_block.replace("W360 seal regeneration", "W361 seal regeneration")
    new_block = new_block.replace("after W360", "after W361")
    new_block = new_block.replace("after W359", "after W360")
    new_block = new_block.replace("WAVE_LOOP_360_COOPERATION.md", "WAVE_LOOP_361_COOPERATION.md")

    ring = "race" if "specs/igla/race" in str(path) else "coder"
    if "-- coder depth +1" in new_block:
        new_block = new_block.replace("-- coder depth +1", "-- coder depth +1", 1)
    elif "-- race depth +1" in new_block:
        new_block = new_block.replace("-- race depth +1", "-- race depth +1", 1)
    else:
        # Fallback: replace generic depth marker with ring-specific marker.
        new_block = new_block.replace("-- depth +1", f"-- {ring} depth +1", 1)

    insert_pos = last.end()
    new_text = text[:insert_pos] + new_block + text[insert_pos:]
    path.write_text(new_text)
    print(f"APPENDED W362 to {path}")


if __name__ == "__main__":
    for spec in sorted(SPECS):
        append_w362(spec)

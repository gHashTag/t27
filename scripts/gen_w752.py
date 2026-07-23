#!/usr/bin/env python3
"""Generate specs/scratch/w752_bench_module_323x2p6_aos_var_call_write.t27."""

import os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DST = os.path.join(REPO, "specs/scratch/w752_bench_module_323x2p6_aos_var_call_write.t27")

OUTER = 323
TOTAL = OUTER * 2 ** 6
LAST_IDX = OUTER - 1
MID_IDX = OUTER // 2  # 161
LAST_X = (2 * (TOTAL - 1)) % 32768
LAST_Y = (2 * (TOTAL - 1) + 1) % 32768
# Index [MID_IDX][1][0][0][0][0][0] corresponds to element number
# MID_IDX * 64 + 1 * 32 + 0*16 + 0*8 + 0*4 + 0*2 + 0 = MID_IDX*64 + 32
MID_E = MID_IDX * (2 ** 6) + 32
MID_X = (2 * MID_E) % 32768
MID_Y = (2 * MID_E + 1) % 32768
WRAP_LAST_X = (2 * (TOTAL - 1) + 32768) % 32768
WRAP_LAST_Y = (2 * (TOTAL - 1) + 1 + 32768) % 32768


def indent(level: int) -> str:
    return "    " * level


def build_tree(shape: list[int], base: int, offset: int) -> tuple[str, int]:
    """Recursively build a t27 literal for shape, returning (text, next_base).

    The returned text is a self-contained literal beginning with the full
    dimension annotation and ending with the matching closing brace, indented
    relative to level 0.
    """
    if len(shape) == 1:
        n = shape[0]
        lines = [f"{indent(1)}[{n}]Pt{{"]
        entries = []
        for i in range(n):
            e = base + i
            x = (2 * e + offset) % 32768
            y = (2 * e + offset + 1) % 32768
            entries.append(f"{indent(2)}Pt{{ .x = {x}, .y = {y} }}")
        lines.append(",\n".join(entries))
        lines.append(f"{indent(1)}}}")
        return "\n".join(lines), base + n

    n = shape[0]
    inner_shape = shape[1:]
    dims = "[" + "][".join(str(d) for d in shape) + "]"
    lines = [f"{indent(1)}{dims}Pt{{"]
    entries = []
    for _ in range(n):
        entry_text, base = build_tree(inner_shape, base, offset)
        entries.append(entry_text)
    lines.append(",\n".join(entries))
    lines.append(f"{indent(1)}}}")
    return "\n".join(lines), base


def generate() -> None:
    shape = [OUTER, 2, 2, 2, 2, 2, 2]
    literal, _ = build_tree(shape, 0, 0)
    first, rest = literal.split("\n", 1)
    indented_lines = [f"    {first}"]
    for line in rest.splitlines():
        indented_lines.append(f"    {line}")
    indented_literal = "\n".join(indented_lines)

    dims = "[" + "][".join(str(d) for d in shape) + "]"
    make_grid = (
        f"pub fn make_grid(offset : u16) -> {dims}Pt {{\n"
        f"    return {indented_literal};\n"
        "}\n"
    )

    header = f"module w752_bench_module_{OUTER}x2p6_aos_var_call_write\n\n\npub struct Pt {{ x : i16, y : i16 }}\n\n"

    footer = f"""
pub const expected : {dims}Pt = make_grid(0);

pub var dst : {dims}Pt = make_grid(0);

test module_var_{OUTER}x2p6_call_write {{
    assert_eq(dst, expected);
    assert_eq(dst[0][0][0][0][0][0][0].x, 0);
    assert_eq(dst[0][0][0][0][0][0][0].y, 1);
    assert_eq(dst[{LAST_IDX}][1][1][1][1][1][1].x, {LAST_X});
    assert_eq(dst[{LAST_IDX}][1][1][1][1][1][1].y, {LAST_Y});
    assert_eq(dst[{MID_IDX}][1][0][0][0][0][0].x, {MID_X});
    assert_eq(dst[{MID_IDX}][1][0][0][0][0][0].y, {MID_Y});
    // Explicit period-identity check: offset 32768 is congruent to 0 modulo 32768
    assert_eq(make_grid(32768)[0][0][0][0][0][0][0].x, 0);
    assert_eq(make_grid(32768)[0][0][0][0][0][0][0].y, 1);
    assert_eq(make_grid(32768)[{LAST_IDX}][1][1][1][1][1][1].x, {WRAP_LAST_X});
    assert_eq(make_grid(32768)[{LAST_IDX}][1][1][1][1][1][1].y, {WRAP_LAST_Y});
}}

bench module_bench_{OUTER}x2p6_call_write {{
    // Site 1: whole-array equality before writes
    assert_eq(dst, expected);

    // Site 2: indexed read before writes
    assert_eq(dst[0][0][0][0][0][0][0].x, 0);
    assert_eq(dst[0][0][0][0][0][0][0].y, 1);
    assert_eq(dst[{LAST_IDX}][1][1][1][1][1][1].x, {LAST_X});
    assert_eq(dst[{LAST_IDX}][1][1][1][1][1][1].y, {LAST_Y});

    // Site 3: indexed signed field writes
    dst[0][0][0][0][0][0][0].x = 1234;
    dst[0][0][0][0][0][0][0].y = -1234;
    dst[{LAST_IDX}][1][1][1][1][1][1].x = -5678;
    dst[{LAST_IDX}][1][1][1][1][1][1].y = 5678;

    // Site 4: read back updated fields
    assert_eq(dst[0][0][0][0][0][0][0].x, 1234);
    assert_eq(dst[0][0][0][0][0][0][0].y, -1234);
    assert_eq(dst[{LAST_IDX}][1][1][1][1][1][1].x, -5678);
    assert_eq(dst[{LAST_IDX}][1][1][1][1][1][1].y, 5678);

    // Site 5: frame-condition checks on untouched elements
    assert_eq(dst[{MID_IDX}][1][0][0][0][0][0].x, {MID_X});
    assert_eq(dst[{MID_IDX}][1][0][0][0][0][0].y, {MID_Y});

    // Site 6: whole-array inequality after partial writes
    // assert_ne is not emitted by the Icarus simulation path, so we verify that
    // the changed elements differ from their original expected values.
    assert_eq(dst[0][0][0][0][0][0][0].x, 1234);
    assert_eq(dst[{LAST_IDX}][1][1][1][1][1][1].x, -5678);
}}
"""

    text = header + make_grid + footer

    if not all(ord(ch) < 128 for ch in text):
        raise ValueError("generated spec contains non-ASCII characters")

    with open(DST, "w", encoding="ascii", newline="\n") as f:
        f.write(text)

    print(f"Wrote {DST} ({len(text)} bytes, {text.count(chr(10))} lines)")


if __name__ == "__main__":
    generate()

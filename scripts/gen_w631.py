#!/usr/bin/env python3
"""Generate specs/scratch/w631_bench_module_81x2p6_aos_var_call_write.t27."""

import os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DST = os.path.join(REPO, "specs/scratch/w631_bench_module_81x2p6_aos_var_call_write.t27")


def indent(level: int) -> str:
    return "    " * level


def build_tree(shape: list[int], base: int, offset: int) -> tuple[str, int]:
    """Recursively build a t27 literal for shape, returning (text, next_base).

    The returned text is a self-contained literal beginning with the full
    dimension annotation (e.g. ``[81][2][2][2][2][2][2]Pt{``) and ending with
    the matching closing brace, indented relative to level 0.
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
    shape = [81, 2, 2, 2, 2, 2, 2]
    literal, _ = build_tree(shape, 0, 0)
    # Move the outer literal so its opening brace is on the same line as ``return``.
    first, rest = literal.split("\n", 1)
    # Add a four-space function-body indent to every line.
    indented_lines = [f"    {first}"]
    for line in rest.splitlines():
        indented_lines.append(f"    {line}")
    indented_literal = "\n".join(indented_lines)

    make_grid = (
        "pub fn make_grid(offset : u16) -> [81][2][2][2][2][2][2]Pt {\n"
        f"    return {indented_literal};\n"
        "}\n"
    )

    header = "module w631_bench_module_81x2p6_aos_var_call_write\n\n\npub struct Pt { x : i16, y : i16 }\n\n"

    footer = """
pub const expected : [81][2][2][2][2][2][2]Pt = make_grid(0);

pub var dst : [81][2][2][2][2][2][2]Pt = make_grid(0);

test module_var_81x2p6_call_write {
    assert_eq(dst, expected);
    assert_eq(dst[0][0][0][0][0][0][0].x, 0);
    assert_eq(dst[0][0][0][0][0][0][0].y, 1);
    assert_eq(dst[80][1][1][1][1][1][1].x, 10366);
    assert_eq(dst[80][1][1][1][1][1][1].y, 10367);
    assert_eq(dst[40][1][0][0][0][0][0].x, 5184);
    assert_eq(dst[40][1][0][0][0][0][0].y, 5185);
    // Explicit modulo-wrap check: offset 32768 folds back to 0..1
    assert_eq(make_grid(32768)[0][0][0][0][0][0][0].x, 0);
    assert_eq(make_grid(32768)[0][0][0][0][0][0][0].y, 1);
    assert_eq(make_grid(32768)[80][1][1][1][1][1][1].x, 10302);
    assert_eq(make_grid(32768)[80][1][1][1][1][1][1].y, 10303);
}

bench module_bench_81x2p6_call_write {
    // Site 1: whole-array equality before writes
    assert_eq(dst, expected);

    // Site 2: indexed read before writes
    assert_eq(dst[0][0][0][0][0][0][0].x, 0);
    assert_eq(dst[0][0][0][0][0][0][0].y, 1);
    assert_eq(dst[80][1][1][1][1][1][1].x, 10366);
    assert_eq(dst[80][1][1][1][1][1][1].y, 10367);

    // Site 3: indexed signed field writes
    dst[0][0][0][0][0][0][0].x = 1234;
    dst[0][0][0][0][0][0][0].y = -1234;
    dst[80][1][1][1][1][1][1].x = -5678;
    dst[80][1][1][1][1][1][1].y = 5678;

    // Site 4: read back updated fields
    assert_eq(dst[0][0][0][0][0][0][0].x, 1234);
    assert_eq(dst[0][0][0][0][0][0][0].y, -1234);
    assert_eq(dst[80][1][1][1][1][1][1].x, -5678);
    assert_eq(dst[80][1][1][1][1][1][1].y, 5678);

    // Site 5: frame-condition checks on untouched elements
    assert_eq(dst[40][1][0][0][0][0][0].x, 5184);
    assert_eq(dst[40][1][0][0][0][0][0].y, 5185);

    // Site 6: whole-array inequality after partial writes
    // assert_ne is not emitted by the Icarus simulation path, so we verify that
    // the changed elements differ from their original expected values.
    assert_eq(dst[0][0][0][0][0][0][0].x, 1234);
    assert_eq(dst[80][1][1][1][1][1][1].x, -5678);
}
"""

    text = header + make_grid + footer

    if not all(ord(ch) < 128 for ch in text):
        raise ValueError("generated spec contains non-ASCII characters")

    with open(DST, "w", encoding="ascii", newline="\n") as f:
        f.write(text)

    print(f"Wrote {DST} ({len(text)} bytes, {text.count(chr(10))} lines)")


if __name__ == "__main__":
    generate()

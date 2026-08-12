#!/usr/bin/env python3
"""Fail when a port and the signal driving it name different quantities.

Wave 656. Prop. 122a found a defect no property covers, because each side of it
is internally consistent:

    bitnet_engine_top.sv:351   .length(reg_neurons)
    dma_controller.sv:8        // One beat = 8 bytes (64-bit). length is byte-count.
    bitnet_engine_top.sv:124   wire [15:0] neurons_per_layer = reg_neurons[15:0];

One register serves as a neuron count in one place and a byte-count DMA length in
another. `dma_controller` is right about bytes; `bitnet_engine_top` is right
about neurons; the connection between them is wrong, and nothing looks at
connections. For N neurons the input DMA moves N BYTES -- ceil(N/8) words --
where the readiness gate demands N, which may block layer 0 entirely.

A general units system would be the right answer and this is not one. Exactly one
module in the bundle documents its units at all, so a gate requiring declared
units everywhere would check one port and pass. Instead this reads the names,
which the emitters write consistently: a port called `length` driven by a signal
called `reg_neurons` is a quantity mismatch visible without any annotation.

VOCABULARY, not inference. Each family below is a quantity this design counts in.
A connection whose port and signal fall in DIFFERENT families is reported. A
connection where either side is unrecognised is not -- silence here means "no
opinion", and the summary says how many were skipped so that silence is
measurable (Prop. 116's lesson).

Usage:  python3 formal/units_scan.py [--self-test]
COVERAGE. Measured, and it is the reason Prop. 194 exists: of 163 port
connections across 13 files, this gate compares **41** and skips **122 as
unrecognised** -- it is blind to 75% of what it looks at. A quantity is inferred
from the port NAME against the hand-written `FAMILIES` table above, so any port
whose name is not in that table is not compared, and a defect there is
undetectable by construction, not merely unfound. The gate has exited 0 every
wave since it landed; that green means "no unit disagreement among the quarter of
connections whose names we recognise", and nothing more. Widening `FAMILIES` is a
shape search (Prop. 193) and cannot bound this residue -- only a design-side
convention that names quantities, or a type annotation, could.

"""

import pathlib
import re
import sys

# comment-scan: strips comments before matching; see strip() below.

FAMILIES = {
    "bytes":   ("length", "byte", "bytes", "nbytes", "burst_bytes"),
    "neurons": ("neuron", "neurons", "num_neurons", "neurons_per_layer"),
    # chunk and word are ONE family. Wave 656: separating them produced two
    # false findings on the first run -- `.input_chunk(activation_word)` and
    # `.weight_chunk(weight_word)` -- because in this design a chunk IS a
    # 54-bit packed word. The vocabulary encoded a distinction the design does
    # not make, which is over-detection for the sixth consecutive wave.
    "words":   ("chunk", "chunks", "num_chunks", "chunk_id",
                "word", "words", "num_words", "word_index", "wr_word"),
    "layers":  ("layer", "layers", "num_layers"),
    "beats":   ("beat", "beats", "arlen", "awlen", "burst_count"),
    # Wave 657: addresses are a quantity too, and an address driven by a count
    # (or vice versa) is the same class of defect as Prop. 122a. Added after
    # enumerating what the scan was skipping -- see Prop. 124 for why that
    # enumeration mostly found NON-quantities.
    "addrs":   ("addr", "address", "araddr", "awaddr", "rd_addr", "wr_addr",
                "src_addr", "dst_addr", "bram_addr", "local_addr", "chunk_addr",
                "act_wr_addr", "read_addr", "write_addr", "buf_read_addr",
                "buf_write_addr", "mem_addr"),
}

# The module name cannot be a keyword. Wave 656: `else if (...)` parsed as a
# module named `else`, so a finding was reported against the wrong line.
KEYWORDS = {"else", "if", "begin", "end", "always", "assign", "wire", "reg",
            "case", "for", "while", "initial", "return", "input", "output"}
# Known-open, declared rather than silenced. Prop. 122a is a real defect that
# has NOT been fixed, because the repair is a design decision (is the length
# wrong, or the gate that consumes it?). Following the expected-refutation
# convention of Prop. 26: a known finding is listed with its reason, and
# anything NOT on this list fails the build. Removing an entry here must
# coincide with fixing the defect, or the gate goes red -- which is the point.
KNOWN_OPEN = {
    ("bitnet_engine_top.sv", "dma_controller", "length", "reg_neurons"):
        "Prop. 122a: the engine passes a neuron count to a byte-count DMA "
        "length. Real, unfixed; the repair is a design decision recorded in "
        "issue #2092.",
}

INST = re.compile(r"^\s*(\w+)\s+(\w+)\s*\((.*?)\);", re.M | re.S)
CONN = re.compile(r"\.(\w+)\s*\(\s*([^)]*?)\s*\)")


def strip(text):
    return re.sub(r"//[^\n]*", "", text)


def family(name):
    """Which quantity family a name belongs to, or None."""
    toks = set(re.split(r"[_\W]+", name.lower()))
    hits = {fam for fam, words in FAMILIES.items() if toks & set(words)}
    # A name in two families ("word_index" is words; "burst_bytes" is bytes) is
    # ambiguous only if the families genuinely differ; take None rather than
    # guess, since a wrong family produces a false finding.
    return next(iter(hits)) if len(hits) == 1 else None


HEAD = re.compile(r"^[ \t]*([A-Za-z_]\w*)[ \t]+([A-Za-z_]\w*)[ \t]*\(", re.M)


def instantiations(src):
    r"""(module, instance, body) with the body matched by paren DEPTH.

    Wave 656: the first version captured the body with a non-greedy `(.*?)\);`,
    which stops at the first `);` and cannot survive a nested parenthesis. The
    engine's DMA instantiation opens with
    `.start(reg_ctrl[1] && !reg_ctrl[0] && ...)`, so the ONE connection this
    gate was written to catch -- `.length(reg_neurons)` -- was never parsed at
    all. Eleven instantiations were read, `dma_controller` was not among them,
    and the gate reported a clean tree.

    The `compared > 0` floor did not help: twenty other connections were
    compared, so the total was healthy while the subject was missing. A floor on
    a total says nothing about coverage of the thing you care about.
    """
    for m in HEAD.finditer(src):
        module, inst = m.group(1), m.group(2)
        if module in KEYWORDS or inst in KEYWORDS:
            continue
        depth, i = 1, m.end()
        while i < len(src) and depth:
            if src[i] == "(":
                depth += 1
            elif src[i] == ")":
                depth -= 1
            i += 1
        yield module, inst, src[m.end():i - 1], m.start()


def check_file(path):
    """(findings, compared, skipped) for one emitted file."""
    src = strip(path.read_text())
    bad, compared, skipped, known = [], 0, 0, []
    for module, inst, body, at in instantiations(src):
        for port, sig in CONN.findall(body):
            sig = re.sub(r"\s+", "", sig)
            if not sig or not re.match(r"^[A-Za-z_]\w*$", sig):
                continue
            pf, sf = family(port), family(sig)
            if pf is None or sf is None:
                skipped += 1
                continue
            compared += 1
            if pf != sf:
                key = (path.name, module, port, sig)
                if key in KNOWN_OPEN:
                    known.append(f"{path.name}: {module}.{port} <- {sig} "
                                 f"({KNOWN_OPEN[key]})")
                    continue
                n = src[:at].count("\n") + 1
                bad.append(
                    f"{path.name}:{n}: `{module} {inst}` connects port `{port}` "
                    f"(a {pf} count) to `{sig}` (a {sf} count). The two sides of "
                    "this connection disagree about what is being counted, and "
                    "each module is internally consistent, so no property covers "
                    "it. See Prop. 123.")
    return bad, compared, skipped, known


def scan(root):
    files = sorted((root / "build" / "rtl").glob("*.sv"))
    if not files:
        print(f"::error::units_scan found no RTL under {root}/build/rtl -- emit "
              "the bundle before running this gate")
        return 1
    bad, compared, skipped, allknown = [], 0, 0, []
    modules_seen = []
    for f in files:
        b, c, s, k = check_file(f)
        modules_seen += [m for m, _i, _b, _a in instantiations(strip(f.read_text()))]
        allknown += k
        bad += b
        compared += c
        skipped += s
    for k in allknown:
        print(f"::warning::known-open {k}")
    for b in bad:
        print(f"::error::{b}")
    # Silence must be measurable: a vocabulary that recognised nothing would
    # report a clean sweep.
    # WITNESS: the specific connection this gate exists for must have been
    # parsed. Wave 656 shipped a version that never saw it -- eleven
    # instantiations read, dma_controller absent, tree clean -- while the
    # `compared > 0` floor passed on twenty other connections. A floor on a
    # total says nothing about coverage of the thing you care about, so name it.
    witness = any("dma_controller" in m for m in modules_seen)
    if not witness:
        print("::error::units_scan never parsed the `dma_controller` "
              "instantiation, which is the connection it was written for "
              "(Prop. 122a). It examined other connections and would have "
              "reported a clean tree. See Prop. 124.")
        return 1

    if compared == 0:
        print(f"::error::units_scan compared 0 connections across {len(files)} "
              f"files ({skipped} skipped as unrecognised) -- it checked nothing, "
              "so its silence means nothing.")
        return 1
    print(f"units scan: {len(files)} files, {compared} connections compared, "
          f"{skipped} skipped as unrecognised, {len(allknown)} known-open, "
          f"{len(bad)} new disagreements")
    return 1 if bad else 0


def self_test():
    import tempfile
    CASES = [
        ("a connection whose sides agree",
         "module top;\n  dma_controller u (.length(byte_count), .clk(clk));\n"
         "endmodule\n", 0),
        ("the Prop. 122a defect: a neuron count driving a byte length",
         "module top;\n  dma_controller u (.length(reg_neurons), .clk(clk));\n"
         "endmodule\n", 1),
        # Wave 656: this case USED to expect a finding, encoding a distinction
        # the design does not make -- a chunk is a 54-bit word here. Kept,
        # inverted, as the regression that keeps the two in one family.
        ("chunks driving a word port -- the same quantity here",
         "module top;\n  m u (.num_words(num_chunks));\nendmodule\n", 0),
        ("a control keyword is not an instantiation",
         "module top;\n  always @(*) else if (length == reg_neurons) x = 1;\n"
         "endmodule\n", 0),
        ("an unrecognised name on one side is not judged",
         "module top;\n  m u (.length(foo_bar));\nendmodule\n", 0),
        ("a commented-out connection is not read",
         "module top;\n  // m u (.length(reg_neurons));\n  m v (.clk(clk));\n"
         "endmodule\n", 0),
    ]
    bad = []
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td)
        for name, text, want in CASES:
            f = d / "zz.sv"
            f.write_text(text)
            found, _c, _s, _k = check_file(f)
            ok = len(found) == want
            print(f"  {'ok  ' if ok else 'FAIL'} {name}: {len(found)} finding(s), "
                  f"want {want}")
            if not ok:
                bad.append(name)

        (d / "build" / "rtl").mkdir(parents=True)
        (d / "build" / "rtl" / "zz.sv").write_text("module m; endmodule\n")
        rc = scan(d)
        print(f"  {'ok  ' if rc else 'FAIL'} a tree with nothing recognisable "
              f"fails rather than passing silently: exit {rc}")
        if rc == 0:
            bad.append("the zero-compare guard let an empty tree pass")

    for b in bad:
        print(f"::error::units_scan self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else scan(r))

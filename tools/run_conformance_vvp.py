#!/usr/bin/env python3
"""Execute conformance vectors against generated RTL -- the first consumer
that RUNS them (#2241).

The checked-in conformance/fpga_*.json corpus was, until this tool, displayed
but never executed: the CI job echoed a pre-recorded 'verdict' field and its
compile loop never invoked vvp. This runner generates a testbench per module
that calls the generated Verilog FUNCTIONS hierarchically (the same style the
historical KATs used), applies each vector case, and fails loudly on any
mismatch.

v1 scope, deliberately narrow and growable (the formal-v1 doctrine: a thin
gate that is REAL beats a broad one that is vacuous): the registry below maps
vector groups whose case shape is understood to a call template. Groups not in
the registry are counted and reported as NOT EXECUTED -- visible debt, never
silent coverage.

Exit codes: 0 = every executed case passed; 1 = any case failed or a mapped
group failed to build; 2 = usage/environment error.
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

# trit (-1|0|1) -> 2-bit field encoding used by the packed word
TRIT_ENC = {0: 0, 1: 1, -1: 2}


def tb_case_extract_trit(c):
    exp = c["expected_trit"]
    return (1, (
        f'    r = dut.extract_trit(32\'d{c["word_raw"]}, 32\'d{c["index"]});\n'
        f'    if ($signed(r) !== {exp}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {exp}", $signed(r));\n'
        f'      fails = fails + 1;\n'
        f'    end else $display("PASS {c["id"]}");\n'
    ))


def tb_case_mac_multiply(c):
    a = TRIT_ENC[c["a_trit0"]]
    b = TRIT_ENC[c["b_trit0"]]
    exp = c["expected_trit0"]
    return (2, (
        f'    r = dut.mac_multiply(32\'d{a}, 32\'d{b}, 8\'d0);\n'
        f'    r = dut.extract_trit(r, 32\'d0);\n'
        f'    if ($signed(r) !== {exp}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {exp}", $signed(r));\n'
        f'      fails = fails + 1;\n'
        f'    end else $display("PASS {c["id"]}");\n'
    ))


def tb_case_mac_invalid_unit(c):
    exp = c["expected_raw"]
    return (1, (
        f'    r = dut.mac_multiply(32\'d1, 32\'d1, 8\'d{c["unit"]});\n'
        f'    if (r !== 32\'d{exp}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {exp}", r);\n'
        f'      fails = fails + 1;\n'
        f'    end else $display("PASS {c["id"]}");\n'
    ))



def _word_from_trits(trits):
    raw = 0
    for i, t in enumerate(trits):
        raw |= TRIT_ENC[t] << (2 * i)
    return raw


def tb_case_pack_trit(c):
    exp = c["expected_packed"]
    return (1, (
        f'    r = dut.pack_trit({c["trit"]}, 32\'d{c["index"]});\n'
        f'    if (r !== 32\'d{exp}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {exp}", r);\n'
        f'      fails = fails + 1;\n'
        f'    end else $display("PASS {c["id"]}");\n'
    ))


def tb_case_mac_cycle(c):
    a = _word_from_trits(c["a_trits"])
    b = _word_from_trits(c["b_trits"])
    exp = c["expected_acc"]
    init = c["initial_acc"]
    return (2, (
        f'    r = dut.mac_cycle(32\'d{a}, 32\'d{b}, 8\'d0, {init});\n'
        f'    if ($signed(r) !== {exp}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {exp}", $signed(r));\n'
        f'      fails = fails + 1;\n'
        f'    end else $display("PASS {c["id"]}");\n'
    ))


def tb_case_mac_status(c):
    exp = c["expected"]
    # "initially" semantics need FRESH state -- stage 0 runs before any
    # state-writing op; "after operation" reads run last (stage 3). The first
    # unstaged version ran ops first and read DONE where the vector said
    # READY: a TB sequencing bug, found by the gate itself.
    stage = 0 if "initial" in c["id"] else 3
    return (stage, (
        f'    r = dut.mac_status_read(8\'d{c["unit"]});\n'
        f'    if (r !== 32\'d{exp}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {exp}", r);\n'
        f'      fails = fails + 1;\n'
        f'    end else $display("PASS {c["id"]}");\n'
    ))



def tb_case_mac_reset(c):
    # Stage 3: a stateful op has already dirtied unit state; reset then read.
    if "units_checked" in c:
        exp = c["expected_acc"]
        lines = ['    dummy = dut.mac_reset_all_ret();\n'] if False else []
        body = ""
        for u in c["units_checked"]:
            body += (
                f'    r = dut.mac_cycle(32\'d1, 32\'d1, 8\'d{u}, 0);\n'
                f'    r = dut.mac_reset(8\'d{u});\n'
                f'    r = dut.mac_acc_read(8\'d{u});\n'
                f'    if ($signed(r) !== {exp}) begin\n'
                f'      $display("FAIL {c["id"]}_u{u}: got %0d want {exp}", $signed(r));\n'
                f'      fails = fails + 1;\n'
                f'    end else $display("PASS {c["id"]}_u{u}");\n'
            )
        return (3, body)
    if "expected_status_after_reset" in c:
        exp = c["expected_status_after_reset"]
        return (3, (
            f'    r = dut.mac_cycle(32\'d1, 32\'d1, 8\'d{c["unit"]}, 0);\n'
            f'    r = dut.mac_reset(8\'d{c["unit"]});\n'
            f'    r = dut.mac_status_read(8\'d{c["unit"]});\n'
            f'    if (r !== 32\'d{exp}) begin\n'
            f'      $display("FAIL {c["id"]}: got %0d want {exp}", r);\n'
            f'      fails = fails + 1;\n'
            f'    end else $display("PASS {c["id"]}");\n'
        ))
    exp = c["expected_acc_after_reset"]
    return (3, (
        f'    r = dut.mac_cycle(32\'d1, 32\'d1, 8\'d{c["unit"]}, 5);\n'
        f'    r = dut.mac_reset(8\'d{c["unit"]});\n'
        f'    r = dut.mac_acc_read(8\'d{c["unit"]});\n'
        f'    if ($signed(r) !== {exp}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {exp}", $signed(r));\n'
        f'      fails = fails + 1;\n'
        f'    end else $display("PASS {c["id"]}");\n'
    ))


# module -> (top, vectors file, {group: case renderer})
REGISTRY = {
    "mac": (
        "ZeroDSP_MAC",
        "conformance/fpga_mac_vectors.json",
        {
            "extract_trit": tb_case_extract_trit,
            "mac_multiply": tb_case_mac_multiply,
            "mac_invalid_unit": tb_case_mac_invalid_unit,
            "pack_trit": tb_case_pack_trit,
            "mac_cycle": tb_case_mac_cycle,
            "mac_status": tb_case_mac_status,
            "mac_reset": tb_case_mac_reset,
        },
    ),
}


def run_module(name, verilog_path, workdir):
    top, vec_path, groups = REGISTRY[name]
    data = json.load(open(os.path.join(ROOT, vec_path)))
    vectors = data["vectors"]

    stages = {0: [], 1: [], 2: [], 3: []}
    executed = 0
    for gname, render in groups.items():
        cases = vectors.get(gname, {}).get("cases", [])
        for c in cases:
            stage, text = render(c)
            stages[stage].append(text)
            executed += 1
    body = stages[0] + stages[1] + stages[2] + stages[3]
    skipped = [g for g in vectors if g not in groups]

    tb = (
        "`timescale 1ns/1ps\n"
        "module conformance_tb;\n"
        "  reg clk = 0; reg rst_n = 1; reg en = 1; wire ready;\n"
        f"  {top} dut(.clk(clk), .rst_n(rst_n), .en(en), .ready(ready));\n"
        "  reg [31:0] r;\n"
        "  integer fails = 0;\n"
        "  initial begin\n"
        + "".join(body)
        + '    if (fails == 0) $display("CONFORMANCE OK: all executed cases passed");\n'
        '    else $display("CONFORMANCE FAILED: %0d case(s)", fails);\n'
        "    $finish(fails == 0 ? 0 : 1);\n"
        "  end\n"
        "endmodule\n"
    )
    tb_path = os.path.join(workdir, f"{name}_conformance_tb.v")
    open(tb_path, "w").write(tb)

    vvp_path = os.path.join(workdir, f"{name}_conformance.vvp")
    build = subprocess.run(
        ["iverilog", "-g2012", "-DSIMULATION", "-o", vvp_path, verilog_path, tb_path],
        capture_output=True, text=True,
    )
    if build.returncode != 0:
        print(build.stderr.strip())
        print(f"{name}: TB BUILD FAILED")
        return 1, executed, skipped
    sim = subprocess.run(["vvp", vvp_path], capture_output=True, text=True)
    print(sim.stdout.strip())
    if sim.stderr.strip():
        print(sim.stderr.strip())
    failed = ("CONFORMANCE OK" not in sim.stdout) or sim.returncode not in (0,)
    return (1 if failed else 0), executed, skipped


def main():
    if len(sys.argv) < 3:
        print("usage: run_conformance_vvp.py <module> <generated.v> [workdir]")
        return 2
    name, verilog_path = sys.argv[1], sys.argv[2]
    workdir = sys.argv[3] if len(sys.argv) > 3 else "/tmp"
    if name not in REGISTRY:
        print(f"{name}: not in the executed-vector registry (v1 covers: {sorted(REGISTRY)})")
        return 2
    rc, executed, skipped = run_module(name, verilog_path, workdir)
    print(f"executed cases: {executed}; groups not yet executed: {len(skipped)} {sorted(skipped)}")
    return rc


if __name__ == "__main__":
    sys.exit(main())

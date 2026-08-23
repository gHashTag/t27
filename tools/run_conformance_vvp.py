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
import tempfile
import shutil
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



def tb_case_spi_prescaler(c):
    """The executable half of a prescaler vector.

    The case gives (clk_freq, target_freq, expected_divisor). No spec function
    maps frequencies to a prescaler code -- that arithmetic is the vector's own
    derivation -- so what executes is the round trip the SPEC does own:
    spi_set_prescaler(code) then spi_get_prescaler_div() == expected_divisor.
    That chain is worth executing: it was a `match` the parser silently dropped
    until #1941, so the whole dispatch was once an unimplemented stub.
    """
    d = c["expected_divisor"]
    code = d.bit_length() - 2  # 2->0, 4->1, 8->2, ... 256->7
    return (1, (
        f'    r = dut.spi_set_prescaler(8\'d{code});\n'
        f'    r = dut.spi_get_prescaler_div(1\'b0);\n'
        f'    if (r !== 32\'d{d}) begin\n'
        f'      $display("FAIL {c["id"]}: got %0d want {d}", r);\n'
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
    "spi": (
        "SPI_Master",
        "conformance/fpga_spi.json",
        {
            "spi_prescaler": tb_case_spi_prescaler,
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
    try:
        build = subprocess.run(
            ["iverilog", "-g2012", "-DSIMULATION", "-o", vvp_path, verilog_path, tb_path],
            capture_output=True, text=True,
        )
    except OSError as e:
        # T117: absent is not failed. Without this the call raises
        # FileNotFoundError and this gate dies with a traceback -- red, and not
        # one word saying whether the RTL is wrong or the simulator is missing.
        print(f"{name}: iverilog is not on PATH ({e.strerror})")
        print(f"{name}: NOTHING WAS EXECUTED -- the simulator is absent, which is")
        print("not the same as vectors that failed.")
        return 2, 0, skipped
    if build.returncode != 0:
        print(build.stderr.strip())
        print(f"{name}: TB BUILD FAILED")
        return 1, executed, skipped
    try:
        sim = subprocess.run(["vvp", vvp_path], capture_output=True, text=True)
    except OSError as e:
        print(f"{name}: vvp is not on PATH ({e.strerror})")
        print(f"{name}: NOTHING WAS EXECUTED -- the simulator is absent.")
        return 2, 0, skipped
    print(sim.stdout.strip())
    if sim.stderr.strip():
        print(sim.stderr.strip())
    failed = ("CONFORMANCE OK" not in sim.stdout) or sim.returncode not in (0,)
    return (1 if failed else 0), executed, skipped


def self_check():
    """Run the WHOLE program once per verdict it can reach without a simulator.

    T117. This gate ran in CI with no negative control in any form -- invisible
    to `tri gates sweep` until selection moved from names to properties, because
    it is called `run_*` rather than `check_*`.

    Every case here spawns the real program and asserts the exit code AND the
    message. Three of its exits are 2 and two are 1, so the code alone cannot
    say which branch spoke, and this file's own subject is a gate that reported
    a vacuous pass.

    NOT COVERED, said rather than left to be inferred: the passing path and the
    real per-case mismatch, both of which need iverilog and a built .v. The
    "nothing was executed" verdict is reachable only through a registry module
    whose vectors are gone, which means planting a corpus -- worth building and
    not built here.
    """
    ok = True

    def case(label, argv, want, expect, absent, env=None):
        nonlocal ok
        r = subprocess.run([sys.executable, os.path.abspath(__file__), *argv],
                           capture_output=True, text=True,
                           env=env or os.environ.copy())
        out = r.stdout + r.stderr
        missing = [s for s in expect if s not in out]
        leaked = [s for s in absent if s in out]
        good = r.returncode == want and not missing and not leaked
        print(f"  {label:<40} " + (f"exit {want}, right branch" if good
                                   else "CONTROL FAILED"))
        if not good:
            ok = False
            print(f"       exit {r.returncode!r} (want {want!r})")
            if missing:
                print(f"       the branch never said: {missing!r}")
            if leaked:
                print(f"       neighbouring marker leaked: {leaked!r}")
            print(f"       said {out[:300]!r}")

    case("no arguments is usage, not a pass", [], 2,
         ["usage: run_conformance_vvp.py"],
         ["CONFORMANCE OK", "NOTHING WAS EXECUTED", "TB BUILD FAILED", "Traceback"])

    case("a module outside the registry is refused",
         ["not_a_real_module", "/nonexistent.v"], 2,
         ["not in the executed-vector registry"],
         ["CONFORMANCE OK", "TB BUILD FAILED", "Traceback"])

    name = sorted(REGISTRY)[0]
    with tempfile.TemporaryDirectory() as td:
        # A registry module with a .v that cannot build: the TB-BUILD branch,
        # reached without needing correct RTL. iverilog must be PRESENT for this
        # one, or it would be measuring the case below instead.
        if shutil.which("iverilog"):
            bad_v = os.path.join(td, "not_verilog.v")
            open(bad_v, "w").write("this is not verilog at all\n")
            case("unbuildable RTL is a build failure",
                 [name, bad_v, td], 1,
                 [f"{name}: TB BUILD FAILED"],
                 ["CONFORMANCE OK", "is not on PATH", "Traceback"])
        else:
            print("  unbuildable RTL is a build failure     UNRUN (no iverilog on PATH)")
            ok = False

        # And the same world with the simulator removed. Before this commit the
        # call raised FileNotFoundError: red, with nothing said about whether
        # the RTL was wrong or the tool was missing.
        stripped = os.environ.copy()
        stripped["PATH"] = os.pathsep.join(
            d for d in stripped.get("PATH", "").split(os.pathsep)
            if d and not os.path.exists(os.path.join(d, "iverilog")))
        case("a missing simulator is named, not a failure",
             [name, os.path.join(td, "whatever.v"), td], 2,
             ["iverilog is not on PATH", "NOTHING WAS EXECUTED"],
             ["TB BUILD FAILED", "CONFORMANCE OK", "Traceback"], env=stripped)

    print(f"  self-check: four verdicts reached, each by its own message; the passing "
          f"path and per-case mismatch are NOT covered here = {ok}")
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
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
    # T67: zero executed cases is a FAILURE, not a pass. A testbench with no
    # cases prints "CONFORMANCE OK: all executed cases passed" -- vacuously
    # true and indistinguishable from a real run. Emptying a vector file made
    # this whole job green end to end while the vectors it names ceased to
    # exist. `.claude/skills/ci-gates/SKILL.md` §1: a gate that cannot fail
    # reads as coverage and is worse than none.
    if rc == 0 and executed == 0:
        print()
        print(f"{name}: NOTHING WAS EXECUTED. 'all executed cases passed' over an")
        print("empty set is not a pass -- the vectors this module names are gone,")
        print("unparseable, or no longer match the registry's call templates.")
        return 1
    return rc


if __name__ == "__main__":
    sys.exit(main())

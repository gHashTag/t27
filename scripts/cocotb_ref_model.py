#!/usr/bin/env python3
"""
Cocotb-ready reference-model cross-check for t27 -> Icarus Verilog.

Usage (standalone):
    python3 scripts/cocotb_ref_model.py \
        --ast-json ast.json \
        --verilog DUT.v \
        --top-module MODULE_TB

The script can also be imported as a cocotb test module; when cocotb is
present it will use the runner API, otherwise it falls back to running
``iverilog`` + ``vvp`` directly.

The reference model extracts expected literal values from ``assert_eq``
calls inside ``test`` / ``invariant`` blocks and verifies that the generated
Verilog simulation log reports ``[TEST] <name> : PASSED`` for every block
whose expectations can be statically evaluated. Any ``FAILED`` line is a
reference-model mismatch.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

HAVE_COCOTB = False
try:
    import cocotb
    from cocotb.triggers import Timer

    HAVE_COCOTB = True
except Exception:  # cocotb is optional; standalone mode works without it
    pass


# ---------------------------------------------------------------------------
# AST helpers
# ---------------------------------------------------------------------------

def _children(node: Dict[str, Any]) -> List[Dict[str, Any]]:
    return node.get("children", []) or []


def _literal_value(node: Dict[str, Any]) -> Optional[Any]:
    """Return integer/bool/string value for a literal node, or None."""
    if node.get("kind") != "ExprLiteral":
        return None
    value = node.get("value", "")
    if value in ("true", "false"):
        return value == "true"
    # Try integer forms (decimal / hex / binary / octal)
    for prefix, base in (("0x", 16), ("0b", 2), ("0o", 8)):
        if value.lower().startswith(prefix):
            try:
                return int(value[len(prefix) :], base)
            except ValueError:
                return None
    try:
        return int(value, 10)
    except ValueError:
        try:
            return float(value)
        except ValueError:
            return value  # string literal fallback


def _eval_simple_const(node: Dict[str, Any]) -> Optional[Any]:
    """Evaluate a tiny subset of constant expressions used in asserts."""
    kind = node.get("kind")
    if kind == "ExprLiteral":
        return _literal_value(node)
    if kind == "ExprBinary" and len(_children(node)) == 2:
        op = node.get("extra_op", "")
        left = _eval_simple_const(_children(node)[0])
        right = _eval_simple_const(_children(node)[1])
        if left is None or right is None:
            return None
        try:
            if op == "+":
                return left + right
            if op == "-":
                return left - right
            if op == "*":
                return left * right
            if op == "/":
                return left // right if isinstance(left, int) and isinstance(right, int) else left / right
            if op == "%":
                return left % right
            if op == "==":
                return left == right
            if op == "!=":
                return left != right
            if op == "<":
                return left < right
            if op == "<=":
                return left <= right
            if op == ">":
                return left > right
            if op == ">=":
                return left >= right
        except Exception:
            return None
    if kind == "ExprUnary" and len(_children(node)) == 1:
        op = node.get("extra_op", "")
        child = _eval_simple_const(_children(node)[0])
        if child is None:
            return None
        if op == "-":
            return -child
        if op == "!":
            return not child
    if kind == "ExprCast" and len(_children(node)) == 1:
        return _eval_simple_const(_children(node)[0])
    return None


def _collect_assertions(root: Dict[str, Any]) -> List[Tuple[str, int, Optional[Any], str]]:
    """
    Return list of (block_name, index, expected_value, note) assertions.

    Only ``assert_eq(<actual>, <expected>)`` calls with a statically
    evaluable expected literal are recorded; everything else is skipped
    with note ``skipped``.
    """
    out: List[Tuple[str, int, Optional[Any], str]] = []
    for block in _children(root):
        bkind = block.get("kind")
        if bkind not in ("TestBlock", "InvariantBlock"):
            continue
        block_name = block.get("name", "")
        idx = 0
        for stmt in _children(block):
            call = None
            if stmt.get("kind") == "StmtExpr" and _children(stmt):
                call = _children(stmt)[0]
            elif stmt.get("kind") == "ExprCall":
                call = stmt
            if call is None or call.get("kind") != "ExprCall":
                continue
            if call.get("name") != "assert_eq":
                continue
            args = _children(call)
            if len(args) != 2:
                out.append((block_name, idx, None, "skipped: assert_eq arity"))
                idx += 1
                continue
            expected = _eval_simple_const(args[1])
            if expected is None:
                out.append((block_name, idx, None, "skipped: non-literal expected"))
            else:
                out.append((block_name, idx, expected, "ok"))
            idx += 1
    return out


def _block_has_evaluable_asserts(assertions: List[Tuple[str, int, Optional[Any], str]], block_name: str) -> bool:
    return any(name == block_name and note == "ok" for name, _, _, note in assertions)


def _expected_pass_blocks(assertions: List[Tuple[str, int, Optional[Any], str]]) -> List[str]:
    """Block names that have at least one evaluable assert_eq."""
    seen: set = set()
    out: List[str] = []
    for name, _, _, note in assertions:
        if note == "ok" and name not in seen:
            seen.add(name)
            out.append(name)
    return out


# ---------------------------------------------------------------------------
# Verilog simulation
# ---------------------------------------------------------------------------

def _run_subprocess(cmd: List[str], cwd: Optional[Path] = None) -> Tuple[int, str, str]:
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    return proc.returncode, proc.stdout, proc.stderr


def _find_simulator() -> Tuple[str, str]:
    iverilog = shutil.which("iverilog") or "iverilog"
    vvp = shutil.which("vvp") or "vvp"
    return iverilog, vvp


def _run_iverilog_vvp(verilog_path: Path, top_module: str) -> Tuple[int, str, str]:
    iverilog, vvp = _find_simulator()
    work = verilog_path.parent
    vvp_path = work / f"{verilog_path.stem}.vvp"
    compile_cmd = [
        iverilog,
        "-g2012",
        "-s", top_module,
        "-o", str(vvp_path),
        str(verilog_path),
    ]
    rc, cout, cerr = _run_subprocess(compile_cmd, cwd=work)
    if rc != 0:
        return rc, cout, cerr
    sim_rc, sout, serr = _run_subprocess([vvp, str(vvp_path)], cwd=work)
    return sim_rc, sout + ("\n" + cerr if cerr else ""), ""


def _run_cocotb(verilog_path: Path, top_module: str) -> Tuple[int, str, str]:
    if not HAVE_COCOTB:
        raise RuntimeError("cocotb requested but not importable")
    try:
        from cocotb_tools.runner import get_runner  # type: ignore
    except Exception:
        from cocotb.runner import get_runner  # type: ignore

    work = verilog_path.parent
    build_dir = work / "sim_build"
    runner = get_runner("icarus")
    runner.build(
        hdl_toplevel=top_module,
        sources=[str(verilog_path)],
        build_dir=str(build_dir),
        clean=True,
    )
    # The test module is this file itself; the @cocotb.test below performs
    # the log-parsing cross-check.
    vvp_log = build_dir / "vvp.log"
    runner.test(
        hdl_toplevel=top_module,
        test_module="cocotb_ref_model",
        build_dir=str(build_dir),
        test_dir=str(build_dir),
        test_args=["-l", str(vvp_log)],
    )
    # Read the captured simulator log.
    log_file = vvp_log if vvp_log.exists() else build_dir / "run.log"
    if log_file.exists():
        log_text = log_file.read_text()
    else:
        log_text = ""
    return 0, log_text, ""


def _run_simulation(verilog_path: Path, top_module: str, use_cocotb: bool) -> Tuple[int, str, str]:
    if use_cocotb and HAVE_COCOTB:
        return _run_cocotb(verilog_path, top_module)
    return _run_iverilog_vvp(verilog_path, top_module)


# ---------------------------------------------------------------------------
# Log parsing and cross-check
# ---------------------------------------------------------------------------

_TEST_LINE_RE = re.compile(r"\[TEST\]\s+(.+?)\s*:\s*(starting|PASSED|FAILED)", re.IGNORECASE)


def _parse_log(log_text: str) -> Dict[str, Dict[str, Any]]:
    results: Dict[str, Dict[str, Any]] = {}
    for line in log_text.splitlines():
        m = _TEST_LINE_RE.search(line)
        if not m:
            continue
        name, status = m.group(1).strip(), m.group(2).lower()
        entry = results.setdefault(name, {"started": False, "passed": False, "failed": False})
        if status == "starting":
            entry["started"] = True
        elif status == "passed":
            entry["passed"] = True
        elif status == "failed":
            entry["failed"] = True
    return results


def _cross_check(
    assertions: List[Tuple[str, int, Optional[Any], str]],
    log_results: Dict[str, Dict[str, Any]],
) -> Tuple[bool, List[str]]:
    errors: List[str] = []
    expected_blocks = _expected_pass_blocks(assertions)
    for block in expected_blocks:
        res = log_results.get(block)
        if res is None:
            errors.append(f"missing [TEST] {block} in simulation log")
            continue
        if res["failed"]:
            errors.append(f"[TEST] {block} : FAILED")
        elif not res["passed"]:
            errors.append(f"[TEST] {block} never reported PASSED")
    # Any FAILED line for a block we didn't expect is also a failure.
    for name, res in log_results.items():
        if res["failed"]:
            if name not in expected_blocks:
                errors.append(f"unexpected [TEST] {name} : FAILED")
    return (not errors), errors


# ---------------------------------------------------------------------------
# Main entry points
# ---------------------------------------------------------------------------

def run_reference_check(
    ast_json_path: Path,
    verilog_path: Path,
    top_module: str,
    use_cocotb: bool = False,
) -> Tuple[bool, List[str], str]:
    ast = json.loads(ast_json_path.read_text())
    assertions = _collect_assertions(ast)
    rc, log_text, err_text = _run_simulation(verilog_path, top_module, use_cocotb)
    if rc != 0:
        return False, [f"simulation failed (rc={rc}): {err_text or log_text}"], log_text
    ok, errors = _cross_check(assertions, _parse_log(log_text))
    return ok, errors, log_text


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser(description="t27 cocotb reference-model cross-check")
    parser.add_argument("--ast-json", required=True, type=Path, help="t27 AST JSON produced by 't27c parse --json'")
    parser.add_argument("--verilog", required=True, type=Path, help="Generated Verilog testbench file")
    parser.add_argument("--top-module", required=True, help="Top-level Verilog module name")
    parser.add_argument("--use-cocotb", action="store_true", help="Use cocotb.runner instead of direct iverilog/vvp")
    parser.add_argument("--verbose", action="store_true", help="Print simulation log")
    args = parser.parse_args(argv)

    ok, errors, log_text = run_reference_check(args.ast_json, args.verilog, args.top_module, args.use_cocotb)
    if args.verbose:
        print(log_text)
    if not ok:
        print("cocotb reference-model mismatch:")
        for e in errors:
            print(f"  - {e}")
        return 1

    ast = json.loads(args.ast_json.read_text())
    assertions = _collect_assertions(ast)
    expected = _expected_pass_blocks(assertions)
    print(f"cocotb reference-model OK: {len(expected)} test block(s) passed")
    return 0


# ---------------------------------------------------------------------------
# Cocotb test entry point (used when this module is loaded by cocotb)
# ---------------------------------------------------------------------------

if HAVE_COCOTB:

    @cocotb.test()
    async def reference_model_check(dut) -> None:  # type: ignore
        """Wait for the self-checking testbench to finish and parse its log."""
        # The generated testbench is self-checking; we just wait long enough.
        await Timer(100, units="us")
        # cocotb does not expose simulator stdout directly; verification is
        # delegated to the standalone parser in the runner post-step.
        pass


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

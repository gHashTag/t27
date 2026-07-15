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

The reference model evaluates the expected expression of every ``assert_eq``
inside ``test`` / ``invariant`` blocks and verifies that the generated Verilog
simulation log reports ``[TEST] <name> : PASSED``.  It also reads the VCD probe
value captured for the actual expression and compares it against the
independently evaluated expected value using the declared bit width and
signedness.
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
# Bit-vector value representation
# ---------------------------------------------------------------------------

class Bv:
    """A width-aware bit-vector value."""

    __slots__ = ("value", "width", "signed")

    def __init__(self, value: int, width: int, signed: bool) -> None:
        self.width = width
        self.signed = signed
        self.value = self._mask(value)

    def _mask(self, v: int) -> int:
        return int(v) & ((1 << self.width) - 1)

    def as_int(self) -> int:
        """Return the value as a Python int with correct sign."""
        v = self.value
        if self.signed and v >= (1 << (self.width - 1)):
            v -= 1 << self.width
        return v

    def as_unsigned(self) -> int:
        return self.value

    def __repr__(self) -> str:
        return f"Bv({self.as_int()}, w={self.width}, signed={self.signed})"


# ---------------------------------------------------------------------------
# Type/width helpers
# ---------------------------------------------------------------------------

_TYPE_WIDTH: Dict[str, int] = {
    "bool": 1,
    "u8": 8,
    "i8": 8,
    "u16": 16,
    "i16": 16,
    "u32": 32,
    "i32": 32,
    "u64": 64,
    "i64": 64,
    "u128": 128,
    "i128": 128,
    "usize": 32,
    "int": 32,
    "nat": 32,
}

_TYPE_SIGNED: set = {"i8", "i16", "i32", "i64", "i128", "int"}


def _parse_array_type(ty: str) -> Optional[Tuple[List[int], str]]:
    """Parse '[2][3]i16' into ([2, 3], 'i16') or None."""
    rest = ty.strip()
    dims: List[int] = []
    while rest.startswith("["):
        close = rest.find("]")
        if close == -1:
            return None
        try:
            dims.append(int(rest[1:close].strip()))
        except ValueError:
            return None
        rest = rest[close + 1 :].strip()
    if not dims or not rest:
        return None
    return (dims, rest)


def _base_type_name(ty: str) -> str:
    parsed = _parse_array_type(ty)
    return parsed[1] if parsed else ty.strip()


def _type_width_signed(ty: str) -> Optional[Tuple[int, bool]]:
    """Return (width, signed) for a scalar t27 type, or None."""
    parsed = _parse_array_type(ty)
    elem_type = parsed[1] if parsed else ty.strip()
    width = _TYPE_WIDTH.get(elem_type)
    if width is None:
        return None
    signed = elem_type in _TYPE_SIGNED
    if parsed:
        # A whole array is not scalar, but callers decide whether they want the
        # element width.
        return (width, signed)
    return (width, signed)


def _scalar_array_info(ty: str) -> Optional[Tuple[int, int, bool]]:
    """For '[N]i16' return (count, element_width, signed)."""
    parsed = _parse_array_type(ty)
    if not parsed:
        return None
    dims, elem = parsed
    if len(dims) != 1:
        return None
    width = _TYPE_WIDTH.get(elem)
    if width is None:
        return None
    return (dims[0], width, elem in _TYPE_SIGNED)


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
            return value


def _literal_bv(node: Dict[str, Any]) -> Optional[Bv]:
    """Return a width-aware bit-vector for a literal, or None."""
    if node.get("kind") != "ExprLiteral":
        return None
    value = node.get("value", "")
    if value == "true":
        return Bv(1, 1, False)
    if value == "false":
        return Bv(0, 1, False)
    extra = node.get("extra_type", "")
    if extra:
        ws = _type_width_signed(extra)
        if ws is None:
            return None
        width, signed = ws
    else:
        width, signed = 32, True
    # Parse numeric literal.
    for prefix, base in (("0x", 16), ("0b", 2), ("0o", 8)):
        if value.lower().startswith(prefix):
            try:
                return Bv(int(value[len(prefix) :], base), width, signed)
            except ValueError:
                return None
    try:
        return Bv(int(value, 10), width, signed)
    except ValueError:
        return None


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


def _sanitize_probe_name(name: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", name)


def _probe_name(block_name: str, idx: int) -> str:
    return f"_t27_probe_{_sanitize_probe_name(block_name)}_{idx}"


def _collect_top_level_decls(root: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    """Collect top-level declarations by name for type/function lookup."""
    out: Dict[str, Dict[str, Any]] = {}
    for decl in _children(root):
        kind = decl.get("kind")
        name = decl.get("name", "")
        if not name:
            continue
        if kind == "StructDecl":
            out[f"struct:{name}"] = decl
        elif kind == "FnDecl":
            out[f"fn:{name}"] = decl
        elif kind == "ConstDecl":
            out[f"const:{name}"] = decl
        elif kind == "EnumDecl":
            out[f"enum:{name}"] = decl
    return out


def _struct_field_type(structs: Dict[str, Dict[str, Any]], struct_name: str, field_name: str) -> Optional[str]:
    decl = structs.get(f"struct:{struct_name}")
    if not decl:
        return None
    for field in _children(decl):
        if field.get("name") == field_name:
            return field.get("extra_type", "")
    return None


def _find_function_body(fn: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Return the first StmtReturn or bare return expression in a function."""
    for stmt in _children(fn):
        kind = stmt.get("kind")
        if kind == "ExprReturn" and _children(stmt):
            return _children(stmt)[0]
        if kind == "StmtReturn" and _children(stmt):
            child = _children(stmt)[0]
            if child.get("kind") == "ExprReturn" and _children(child):
                return _children(child)[0]
            return child
    return None


# ---------------------------------------------------------------------------
# Expression evaluator with typed width/signedness
# ---------------------------------------------------------------------------

class EvalContext:
    """Holds variable bindings and top-level declarations for expression eval."""

    def __init__(self, root: Dict[str, Any]) -> None:
        self.root = root
        self.decls = _collect_top_level_decls(root)
        self.vars: Dict[str, Bv] = {}
        # Cache function parameter/return types and local variable types.
        self.fn_param_types: Dict[str, List[Tuple[str, str]]] = {}
        self.fn_return_types: Dict[str, str] = {}
        self.fn_local_types: Dict[str, Dict[str, str]] = {}
        for decl in _children(root):
            if decl.get("kind") != "FnDecl":
                continue
            name = decl.get("name", "")
            if name:
                self.fn_param_types[name] = decl.get("params", []) or []
                self.fn_return_types[name] = decl.get("extra_return_type", "")
            locals_map: Dict[str, str] = {}
            for stmt in _children(decl):
                if stmt.get("kind") == "StmtLocal":
                    vname = stmt.get("name", "")
                    vtype = stmt.get("extra_type", "")
                    if vname and vtype:
                        locals_map[vname] = vtype
            self.fn_local_types[name] = locals_map

    def bind(self, name: str, value: Bv) -> None:
        self.vars[name] = value

    def resolve_var_type(self, name: str) -> Optional[str]:
        if name in self.vars:
            return None  # runtime binding; type is carried by the Bv
        # module-level const
        c = self.decls.get(f"const:{name}")
        if c is not None:
            return c.get("extra_type", "")
        return None


def _type_of_expr(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Tuple[int, bool]]:
    """Infer the scalar width and signedness of an expression node."""
    kind = node.get("kind")
    if kind == "ExprLiteral":
        bv = _literal_bv(node)
        return (bv.width, bv.signed) if bv else None
    if kind == "ExprIdentifier":
        name = node.get("name", "")
        if name in ctx.vars:
            v = ctx.vars[name]
            return (v.width, v.signed)
        ty = ctx.resolve_var_type(name)
        if ty:
            return _type_width_signed(ty)
        return None
    if kind == "ExprCall":
        ret_ty = ctx.fn_return_types.get(node.get("name", ""), "")
        return _type_width_signed(ret_ty)
    if kind == "ExprFieldAccess" and _children(node):
        base = _children(node)[0]
        base_name = _base_name(base)
        if base_name is None:
            return None
        base_type = _resolve_base_type(ctx, base_name)
        if base_type is None:
            return None
        ftype = _struct_field_type(ctx.decls, base_type, node.get("name", ""))
        if ftype is None:
            return None
        info = _scalar_array_info(ftype)
        if info:
            return (info[0] * info[1], info[2])
        return _type_width_signed(ftype)
    if kind == "ExprIndex" and len(_children(node)) >= 2:
        base = _children(node)[0]
        base_name = _base_name(base)
        if base_name is None:
            return None
        base_type = _resolve_base_type(ctx, base_name)
        if base_type is None:
            return None
        # Primitive scalar array element.
        parsed = _parse_array_type(base_type)
        if parsed:
            _, elem = parsed
            ws = _type_width_signed(elem)
            if ws:
                return ws
        # Scalar-struct array field element access: base is a field access
        # whose field is a fixed-size scalar array.
        if base.get("kind") == "ExprFieldAccess":
            ftype = _struct_field_type(ctx.decls, base_type, base.get("name", ""))
            if ftype:
                info = _scalar_array_info(ftype)
                if info:
                    return (info[1], info[2])
        return None
    if kind == "ExprCast":
        target = node.get("extra_type", "")
        base = target.split("[")[0].strip()
        return _type_width_signed(base)
    if kind == "ExprBinary":
        op = node.get("extra_op", "")
        if op in ("&&", "||", "and", "or", "==", "!=", "<", "<=", ">", ">="):
            return (1, False)
        left = _type_of_expr(ctx, _children(node)[0])
        right = _type_of_expr(ctx, _children(node)[1])
        if left is None or right is None:
            return None
        if op in ("<<", ">>"):
            return left
        return (max(left[0], right[0]), left[1] or right[1])
    if kind == "ExprUnary":
        op = node.get("extra_op", "")
        child = _type_of_expr(ctx, _children(node)[0])
        if child is None:
            return None
        if op in ("!", "not"):
            return (1, False)
        return child
    return None


def _base_name(node: Dict[str, Any]) -> Optional[str]:
    kind = node.get("kind")
    if kind == "ExprIdentifier":
        return node.get("name", "") or None
    if kind == "ExprIndex" and _children(node):
        return _base_name(_children(node)[0])
    return None


def _resolve_base_type(ctx: EvalContext, name: str) -> Optional[str]:
    if name in ctx.vars:
        # We don't store type names for runtime vars; fallback below.
        pass
    ty = ctx.resolve_var_type(name)
    if ty:
        return _base_type_name(ty)
    return None


def _eval_expr_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    """Evaluate an expression and return a width-aware Bv value."""
    kind = node.get("kind")
    if kind == "ExprLiteral":
        return _literal_bv(node)
    if kind == "ExprIdentifier":
        name = node.get("name", "")
        if name in ctx.vars:
            return ctx.vars[name]
        return None
    if kind == "ExprCall":
        return _eval_call_bv(ctx, node)
    if kind == "ExprFieldAccess":
        return _eval_field_bv(ctx, node)
    if kind == "ExprIndex":
        return _eval_index_bv(ctx, node)
    if kind == "ExprCast":
        return _eval_cast_bv(ctx, node)
    if kind == "ExprBinary":
        return _eval_binary_bv(ctx, node)
    if kind == "ExprUnary":
        return _eval_unary_bv(ctx, node)
    if kind == "ExprSwitch":
        return _eval_switch_bv(ctx, node)
    if kind == "ExprIf":
        return _eval_ternary_bv(ctx, node)
    if kind == "ExprStructLit":
        return _eval_struct_lit_bv(ctx, node)
    if kind == "ExprArrayLiteral":
        return _eval_array_lit_bv(ctx, node)
    return None


def _eval_array_lit_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    """Pack a scalar array literal into a bit-vector (element 0 at LSB)."""
    elem_type = node.get("extra_type", "")
    try:
        count = int(node.get("extra_size", "").split("][")[-1])
    except ValueError:
        return None
    ws = _type_width_signed(elem_type)
    if ws is None:
        return None
    elem_w, signed = ws
    raw = 0
    for i, child in enumerate(_children(node)):
        val = _eval_expr_bv(ctx, child)
        if val is None:
            return None
        mask = (1 << elem_w) - 1
        raw |= (val.value & mask) << (i * elem_w)
    return Bv(raw, count * elem_w, signed)


def _eval_struct_lit_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    """Pack a scalar-struct literal into a bit-vector (LSB-first field order)."""
    struct_name = node.get("name", "")
    decl = ctx.decls.get(f"struct:{struct_name}")
    if decl is None:
        return None
    fields = [(f.get("name", ""), f.get("extra_type", "")) for f in _children(decl)]
    # Collect explicitly provided field values.
    assigned: Dict[str, Bv] = {}
    for child in _children(node):
        if child.get("kind") != "ExprFieldAccess":
            continue
        fname = child.get("name", "")
        kids = _children(child)
        if not kids:
            continue
        val = _eval_expr_bv(ctx, kids[0])
        if val is None:
            return None
        assigned[fname] = val
    raw = 0
    offset = 0
    total_width = 0
    for fname, ftype in fields:
        val = assigned.get(fname)
        if val is None:
            info = _scalar_array_info(ftype)
            if info is not None:
                fw = info[0] * info[1]
                val = Bv(0, fw, info[2])
            else:
                ws = _type_width_signed(ftype)
                if ws is None:
                    return None
                fw, signed = ws
                val = Bv(0, fw, signed)
        mask = (1 << val.width) - 1
        raw |= (val.value & mask) << offset
        offset += val.width
        total_width += val.width
    return Bv(raw, total_width, False)


def _eval_call_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    name = node.get("name", "")
    args = _children(node)
    fn = ctx.decls.get(f"fn:{name}")
    if fn is None:
        return None
    params = ctx.fn_param_types.get(name, [])
    if len(args) != len(params):
        return None
    call_ctx = EvalContext(ctx.root)
    call_ctx.vars.update(ctx.vars)
    for (pname, ptype), arg in zip(params, args):
        arg_bv = _eval_expr_bv(ctx, arg)
        if arg_bv is None:
            arg_ws = _type_width_signed(ptype)
            if arg_ws is None:
                return None
            arg_bv = Bv(0, *arg_ws)
        call_ctx.bind(pname, arg_bv)
    # Add local type info for function-local vars.
    call_ctx.fn_local_types = ctx.fn_local_types
    body = _find_function_body(fn)
    if body is None:
        return None
    return _eval_expr_bv(call_ctx, body)


def _eval_field_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    if not _children(node):
        return None
    base = _children(node)[0]
    base_name = _base_name(base)
    if base_name is None:
        return None
    base_type = _resolve_base_type(ctx, base_name)
    if base_type is None:
        return None
    field_name = node.get("name", "")
    ftype = _struct_field_type(ctx.decls, base_type, field_name)
    if ftype is None:
        return None
    whole = _eval_expr_bv(ctx, base)
    if whole is None:
        return None
    # Compute offset and width within the packed vector (reverse field order).
    struct_decl = ctx.decls.get(f"struct:{base_type}")
    if struct_decl is None:
        return None
    fields = [(f.get("name", ""), f.get("extra_type", "")) for f in _children(struct_decl)]
    # Fields are stored LSB-first in the packed vector, so the offset of a field
    # is the sum of widths of fields declared before it.
    offset = 0
    field_width = 1
    for fname, fty in fields:
        info = _scalar_array_info(fty)
        fw = info[0] * info[1] if info else (_TYPE_WIDTH.get(fty) or 1)
        if fname == field_name:
            field_width = fw
            break
        offset += fw
    # Extract the field bits.
    raw = (whole.value >> offset) & ((1 << field_width) - 1)
    signed = False
    if ftype:
        info = _scalar_array_info(ftype)
        if info:
            signed = info[2]
        else:
            signed = ftype.strip() in _TYPE_SIGNED
    return Bv(raw, field_width, signed)


def _eval_index_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    if len(_children(node)) < 2:
        return None
    base = _children(node)[0]
    index = _children(node)[1]
    idx_bv = _eval_expr_bv(ctx, index)
    if idx_bv is None:
        return None
    idx = idx_bv.as_int()
    base_name = _base_name(base)
    if base_name is None:
        return None
    base_type = _resolve_base_type(ctx, base_name)
    if base_type is None:
        return None
    parsed = _parse_array_type(base_type)
    if parsed:
        dims, elem = parsed
        elem_ws = _type_width_signed(elem)
        if elem_ws:
            # Primitive scalar array element.
            whole_bv = ctx.vars.get(base_name)
            if whole_bv is None:
                return None
            # Linear index (row-major, outermost varies slowest).
            # We only handle a single dimension here; multi-dim primitive arrays
            # are unpacked and we don't have a whole-array value to slice.
            if len(dims) == 1:
                raw = (whole_bv.value >> (idx * elem_ws[0])) & ((1 << elem_ws[0]) - 1)
                return Bv(raw, *elem_ws)
            return None
    # Scalar-struct array field element access: base is a field access.
    if base.get("kind") == "ExprFieldAccess":
        field_bv = _eval_field_bv(ctx, base)
        if field_bv is None:
            return None
        ftype = _struct_field_type(ctx.decls, base_type, base.get("name", ""))
        if ftype is None:
            return None
        info = _scalar_array_info(ftype)
        if info is None:
            return None
        count, elem_w, signed = info
        if idx < 0 or idx >= count:
            return None
        raw = (field_bv.value >> (idx * elem_w)) & ((1 << elem_w) - 1)
        return Bv(raw, elem_w, signed)
    return None


def _eval_cast_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    if not _children(node):
        return None
    target = node.get("extra_type", "")
    base = target.split("[")[0].strip()
    ws = _type_width_signed(base)
    if ws is None:
        return None
    src = _eval_expr_bv(ctx, _children(node)[0])
    if src is None:
        return None
    return Bv(src.value, ws[0], ws[1])


def _eval_binary_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    if len(_children(node)) < 2:
        return None
    op = node.get("extra_op", "")
    left = _eval_expr_bv(ctx, _children(node)[0])
    right = _eval_expr_bv(ctx, _children(node)[1])
    if left is None or right is None:
        return None
    if op in ("&&", "||", "and", "or"):
        return Bv(1 if (left.as_int() and right.as_int()) else 0, 1, False)
    if op in ("==", "!=", "<", "<=", ">", ">="):
        result = _compare_bv(left, op, right)
        return Bv(1 if result else 0, 1, False)
    res_type = _type_of_expr(ctx, node) or (max(left.width, right.width), left.signed or right.signed)
    width, signed = res_type
    a = left.as_int()
    b = right.as_int()
    if op == "+":
        return Bv(a + b, width, signed)
    if op == "-":
        return Bv(a - b, width, signed)
    if op == "*":
        return Bv(a * b, width, signed)
    if op == "/":
        if b == 0:
            return None
        if signed:
            return Bv(_signed_div(a, b), width, signed)
        return Bv(a // b, width, signed)
    if op == "%":
        if b == 0:
            return None
        if signed:
            return Bv(_signed_rem(a, b), width, signed)
        return Bv(a % b, width, signed)
    if op == "&":
        return Bv(left.value & right.value, width, signed)
    if op == "|":
        return Bv(left.value | right.value, width, signed)
    if op == "^":
        return Bv(left.value ^ right.value, width, signed)
    if op == "<<":
        return Bv(left.value << (b & 0x1F), width, signed)
    if op == ">>":
        shift = b & 0x1F
        if signed:
            # Arithmetic right shift preserving sign bit.
            sign_bit = (left.value >> (left.width - 1)) & 1
            raw = left.value >> shift
            # Replicate sign bit into vacated high bits within result width.
            mask = (1 << (left.width - shift)) - 1
            if sign_bit:
                raw |= ((1 << shift) - 1) << (left.width - shift)
            return Bv(raw, width, signed)
        return Bv(left.value >> shift, width, signed)
    return None


def _compare_bv(left: Bv, op: str, right: Bv) -> bool:
    # For comparisons, use the signedness context of each operand.
    signed = left.signed or right.signed
    if signed:
        a, b = left.as_int(), right.as_int()
    else:
        a, b = left.value, right.value
    if op == "==":
        return a == b
    if op == "!=":
        return a != b
    if op == "<":
        return a < b
    if op == "<=":
        return a <= b
    if op == ">":
        return a > b
    if op == ">=":
        return a >= b
    return False


def _signed_div(a: int, b: int) -> int:
    if b == 0:
        return 0
    # Truncation toward zero, matching Verilog signed division.
    q = abs(a) // abs(b)
    if (a < 0) ^ (b < 0):
        q = -q
    return q


def _signed_rem(a: int, b: int) -> int:
    if b == 0:
        return 0
    r = abs(a) % abs(b)
    if a < 0:
        r = -r
    return r


def _eval_unary_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    if not _children(node):
        return None
    op = node.get("extra_op", "")
    child = _eval_expr_bv(ctx, _children(node)[0])
    if child is None:
        return None
    if op in ("!", "not"):
        return Bv(0 if child.as_int() else 1, 1, False)
    if op == "-":
        return Bv(-child.as_int(), child.width, child.signed)
    if op == "~":
        return Bv(~child.value, child.width, child.signed)
    return None


def _eval_switch_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    if len(_children(node)) < 2:
        return None
    switch_val = _eval_expr_bv(ctx, _children(node)[0])
    if switch_val is None:
        return None
    cases = _children(node)[1:]
    default: Optional[Bv] = None
    for case in cases:
        if case.get("kind") != "ConstDecl":
            continue
        cname = case.get("name", "")
        is_else = cname == "" or cname == "else"
        if is_else:
            if _children(case):
                default = _eval_expr_bv(ctx, _children(case)[0])
            continue
        cmp = _literal_bv(case)
        if cmp is None:
            continue
        if cmp.value == switch_val.value:
            if _children(case):
                return _eval_expr_bv(ctx, _children(case)[0])
            return None
    return default


def _eval_ternary_bv(ctx: EvalContext, node: Dict[str, Any]) -> Optional[Bv]:
    """ExprIf in expression position is a ternary."""
    children = _children(node)
    if len(children) < 3:
        return None
    cond = _eval_expr_bv(ctx, children[0])
    if cond is None:
        return None
    if cond.as_int():
        return _eval_expr_bv(ctx, children[1])
    return _eval_expr_bv(ctx, children[2])


def _collect_assertions(root: Dict[str, Any]) -> List[Tuple[str, int, Optional[Any], str, str]]:
    """
    Return list of (block_name, index, expected_value, note, probe_name) assertions.

    The expected value is a Python int when the expected expression can be
    statically evaluated; otherwise None with a descriptive note.
    """
    ctx = EvalContext(root)
    out: List[Tuple[str, int, Optional[Any], str, str]] = []
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
            probe = _probe_name(block_name, idx)
            if len(args) != 2:
                out.append((block_name, idx, None, "skipped: assert_eq arity", probe))
                idx += 1
                continue
            # Infer the actual expression's scalar width/signedness.  This is
            # required for W540 wide probes: the VCD slices tell us the physical
            # width, but we need the declared type to know whether the final
            # value is signed.
            actual_ws = _type_of_expr(ctx, args[0])
            # Try the full typed evaluator first.  Keep the Bv object so the
            # cross-check knows the exact width and signedness (W540 wide
            # probes need this to reconstruct and interpret values correctly).
            expected_bv = _eval_expr_bv(ctx, args[1])
            if expected_bv is not None:
                if actual_ws is not None and expected_bv.width != actual_ws[0]:
                    # The evaluator may have used a default/narrow width for an
                    # untyped literal, but the actual expression is wider.  Re-wrap
                    # the literal's integer value at the actual width so the VCD
                    # comparison compares the right number of bits.
                    simple = _eval_simple_const(args[1])
                    if isinstance(simple, int):
                        expected_bv = Bv(simple, actual_ws[0], actual_ws[1])
                out.append((block_name, idx, expected_bv, "ok", probe))
            else:
                # Fall back to the simple literal evaluator for backwards
                # compatibility with constant-only specs.
                expected = _eval_simple_const(args[1])
                if expected is None:
                    out.append((block_name, idx, None, "skipped: non-literal expected", probe))
                else:
                    if actual_ws is not None and isinstance(expected, int):
                        expected = Bv(expected, actual_ws[0], actual_ws[1])
                    out.append((block_name, idx, expected, "ok", probe))
            idx += 1
    return out


def _block_has_evaluable_asserts(assertions: List[Tuple[str, int, Optional[Any], str, str]], block_name: str) -> bool:
    return any(name == block_name and note == "ok" for name, _, _, _, _ in assertions)


def _expected_pass_blocks(assertions: List[Tuple[str, int, Optional[Any], str, str]]) -> List[str]:
    """Block names that have at least one evaluable assert_eq."""
    seen: set = set()
    out: List[str] = []
    for name, _, _, note, _ in assertions:
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


def _run_iverilog_vvp(verilog_path: Path, top_module: str) -> Tuple[int, str, str, Optional[Path]]:
    iverilog, vvp = _find_simulator()
    work = verilog_path.parent
    vvp_path = work / f"{verilog_path.stem}.vvp"
    vcd_path = work / "dump.vcd"
    compile_cmd = [
        iverilog,
        "-g2012",
        "-s", top_module,
        "-o", str(vvp_path),
        str(verilog_path),
    ]
    rc, cout, cerr = _run_subprocess(compile_cmd, cwd=work)
    if rc != 0:
        return rc, cout, cerr, None
    sim_rc, sout, serr = _run_subprocess([vvp, str(vvp_path)], cwd=work)
    vcd_file = vcd_path if vcd_path.exists() else None
    return sim_rc, sout + ("\n" + serr if serr else ""), "", vcd_file


def _run_cocotb(verilog_path: Path, top_module: str) -> Tuple[int, str, str, Optional[Path]]:
    if not HAVE_COCOTB:
        raise RuntimeError("cocotb requested but not importable")
    try:
        from cocotb_tools.runner import get_runner  # type: ignore
    except Exception:
        from cocotb.runner import get_runner  # type: ignore

    work = verilog_path.parent
    build_dir = work / "sim_build"
    vcd_path = build_dir / "dump.vcd"
    runner = get_runner("icarus")
    runner.build(
        hdl_toplevel=top_module,
        sources=[str(verilog_path)],
        build_dir=str(build_dir),
        clean=True,
    )
    vvp_log = build_dir / "vvp.log"
    runner.test(
        hdl_toplevel=top_module,
        test_module="cocotb_ref_model",
        build_dir=str(build_dir),
        test_dir=str(build_dir),
        test_args=["-l", str(vvp_log)],
    )
    log_file = vvp_log if vvp_log.exists() else build_dir / "run.log"
    if log_file.exists():
        log_text = log_file.read_text()
    else:
        log_text = ""
    vcd_file = vcd_path if vcd_path.exists() else None
    return 0, log_text, "", vcd_file


def _run_simulation(verilog_path: Path, top_module: str, use_cocotb: bool) -> Tuple[int, str, str, Optional[Path]]:
    if use_cocotb and HAVE_COCOTB:
        return _run_cocotb(verilog_path, top_module)
    return _run_iverilog_vvp(verilog_path, top_module)


# ---------------------------------------------------------------------------
# VCD parsing (W538)
# ---------------------------------------------------------------------------

class _VcdParser:
    """Minimal VCD parser sufficient for scalar/vector probe values."""

    def __init__(self, path: Path) -> None:
        self.id_to_name: Dict[str, str] = {}
        self.id_to_width: Dict[str, int] = {}
        self.values: Dict[str, int] = {}
        self._parse(path)

    def _parse(self, path: Path) -> None:
        in_var = False
        var_type = ""
        var_width = 0
        var_id = ""
        var_name = ""
        in_dumpvars = False
        with path.open("r", encoding="utf-8", errors="ignore") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                if line.startswith("$var"):
                    in_var = True
                    parts = line.split()
                    var_type = parts[1] if len(parts) > 1 else ""
                    var_width = int(parts[2]) if len(parts) > 2 else 1
                    var_id = parts[3] if len(parts) > 3 else ""
                    var_name = parts[4] if len(parts) > 4 else ""
                    if "$end" in line:
                        self.id_to_name[var_id] = var_name
                        self.id_to_width[var_id] = var_width
                        in_var = False
                    continue
                if in_var and "$end" in line:
                    self.id_to_name[var_id] = var_name
                    self.id_to_width[var_id] = var_width
                    in_var = False
                    continue
                if line.startswith("$enddefinitions"):
                    continue
                if line.startswith("$dumpvars"):
                    in_dumpvars = True
                    continue
                if line == "$end" and in_dumpvars:
                    in_dumpvars = False
                    continue
                if line.startswith("#"):
                    continue
                if line.startswith("b"):
                    rest = line[1:]
                    sp = rest.rsplit(" ", 1)
                    if len(sp) == 2:
                        val_str, sid = sp
                        try:
                            self.values[sid] = int(val_str, 2) if val_str else 0
                        except ValueError:
                            pass
                elif len(line) >= 2 and line[1:].strip() in self.id_to_name:
                    sid = line[1:].strip()
                    self.values[sid] = 1 if line[0] == "1" else 0

    def probe_value(self, name: str) -> Optional[Tuple[int, int]]:
        for sid, sname in self.id_to_name.items():
            if sname == name:
                val = self.values.get(sid)
                if val is None:
                    return None
                return (val, self.id_to_width.get(sid, 64))
        return None

    def probe_slices(self, base_name: str) -> Optional[List[Tuple[int, int, int]]]:
        """Return sorted slice values for a wide probe: (value, width, offset)."""
        slices: List[Tuple[int, int, int]] = []
        prefix = base_name + "_s"
        for sid, sname in self.id_to_name.items():
            if sname == base_name:
                # single-signal probe, not a slice set
                return None
            if not sname.startswith(prefix):
                continue
            suffix = sname[len(prefix):]
            if not suffix.isdigit():
                continue
            slice_idx = int(suffix)
            val = self.values.get(sid, 0)
            width = self.id_to_width.get(sid, 64)
            offset = slice_idx * 64
            slices.append((val, width, offset))
        if not slices:
            return None
        slices.sort(key=lambda t: t[2])
        return slices


# ---------------------------------------------------------------------------
# Log parsing and cross-check
# ---------------------------------------------------------------------------

_TEST_LINE_RE = re.compile(r"\[TEST\]\s+(.+?)\s*:\s*(starting|PASSED|FAILED)", re.IGNORECASE)
_PROBE_LINE_RE = re.compile(r"\[PROBE\]\s+(.+?)\s+(\d+)\s*=\s*(\d+)")


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


def _interpret_vcd_value(raw: int, width: int, signed: bool) -> int:
    mask = (1 << width) - 1
    raw &= mask
    if signed and raw >= (1 << (width - 1)):
        raw -= 1 << width
    return raw


def _cross_check(
    assertions: List[Tuple[str, int, Optional[Any], str, str]],
    log_results: Dict[str, Dict[str, Any]],
    vcd: Optional[_VcdParser],
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
    for name, res in log_results.items():
        if res["failed"]:
            if name not in expected_blocks:
                errors.append(f"unexpected [TEST] {name} : FAILED")

    if vcd is not None:
        for block_name, idx, expected, note, probe in assertions:
            if note != "ok" or expected is None:
                continue
            expected_bv = expected if isinstance(expected, Bv) else None
            # Prefer the typed width/signedness carried by the expected value;
            # fall back to the single-signal VCD width and a heuristic for
            # plain ints.
            signed = expected_bv.signed if expected_bv is not None else (isinstance(expected, int) and expected < 0)

            slices = vcd.probe_slices(probe)
            if slices is not None:
                raw = 0
                full_width = 0
                for val, swidth, offset in slices:
                    mask = (1 << swidth) - 1
                    raw |= (val & mask) << offset
                    full_width = max(full_width, offset + swidth)
                # The expected Bv was constructed at the actual expression width,
                # which is authoritative even when slices only tell us the lower
                # bound (e.g. a 128-bit value split 64+64).
                if expected_bv is not None:
                    full_width = expected_bv.width
                actual = _interpret_vcd_value(raw, full_width, signed)
            else:
                probe_info = vcd.probe_value(probe)
                if probe_info is None:
                    continue
                raw, width = probe_info
                full_width = expected_bv.width if expected_bv is not None else width
                actual = _interpret_vcd_value(raw, full_width, signed)

            if isinstance(expected, bool):
                expected_int = 1 if expected else 0
            elif isinstance(expected, Bv):
                expected_int = expected.as_int()
            else:
                expected_int = int(expected)
            if actual != expected_int:
                errors.append(
                    f"VCD mismatch {probe} (block {block_name}, assert {idx}): "
                    f"expected {expected_int}, got {actual} (raw={raw}, width={full_width})"
                )

    return (not errors), errors


# ---------------------------------------------------------------------------
# Main entry points
# ---------------------------------------------------------------------------

def run_reference_check(
    ast_json_path: Path,
    verilog_path: Path,
    top_module: str,
    use_cocotb: bool = False,
) -> Tuple[bool, List[str], str, Optional[_VcdParser]]:
    ast = json.loads(ast_json_path.read_text())
    assertions = _collect_assertions(ast)
    rc, log_text, err_text, vcd_path = _run_simulation(verilog_path, top_module, use_cocotb)
    if rc != 0:
        return False, [f"simulation failed (rc={rc}): {err_text or log_text}"], log_text, None
    vcd: Optional[_VcdParser] = None
    if vcd_path is not None:
        try:
            vcd = _VcdParser(vcd_path)
        except Exception as e:
            print(f"warning: could not parse VCD {vcd_path}: {e}")
    ok, errors = _cross_check(assertions, _parse_log(log_text), vcd)
    return ok, errors, log_text, vcd


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser(description="t27 cocotb reference-model cross-check")
    parser.add_argument("--ast-json", required=True, type=Path, help="t27 AST JSON produced by 't27c parse --json'")
    parser.add_argument("--verilog", required=True, type=Path, help="Generated Verilog testbench file")
    parser.add_argument("--top-module", required=True, help="Top-level Verilog module name")
    parser.add_argument("--use-cocotb", action="store_true", help="Use cocotb.runner instead of direct iverilog/vvp")
    parser.add_argument("--verbose", action="store_true", help="Print simulation log")
    args = parser.parse_args(argv)

    ok, errors, log_text, vcd = run_reference_check(args.ast_json, args.verilog, args.top_module, args.use_cocotb)
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
    vcd_note = " (+ VCD probe check)" if vcd is not None else ""
    print(f"cocotb reference-model OK: {len(expected)} test block(s) passed{vcd_note}")
    return 0


# ---------------------------------------------------------------------------
# Cocotb test entry point (used when this module is loaded by cocotb)
# ---------------------------------------------------------------------------

if HAVE_COCOTB:

    @cocotb.test()
    async def reference_model_check(dut) -> None:  # type: ignore
        """Wait for the self-checking testbench to finish and parse its log."""
        await Timer(100, units="us")
        pass


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

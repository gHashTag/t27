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

from __future__ import annotations

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


def _primitive_array_info(ty: str) -> Optional[Tuple[List[int], str, int, bool]]:
    """For '[2][3]i16' return (dims, elem, total_width, signed)."""
    parsed = _parse_array_type(ty)
    if not parsed:
        return None
    dims, elem = parsed
    elem_width = _TYPE_WIDTH.get(elem)
    if elem_width is None:
        return None
    total = elem_width
    for d in dims:
        total *= d
    return (dims, elem, total, elem in _TYPE_SIGNED)


def _is_primitive_scalar_type(ty: str) -> bool:
    """True for 'u32', 'i16', etc. and '[N]u32' one-dimensional scalar arrays."""
    parsed = _parse_array_type(ty)
    elem = parsed[1] if parsed else ty.strip()
    return elem in _TYPE_WIDTH and (parsed is None or len(parsed[0]) == 1)


def _packed_type_width_signed(
    ctx: EvalContext, ty: str
) -> Optional[Tuple[int, bool]]:
    """Return (width, signed) for a lowerable packed scalar struct or array."""
    parsed = _parse_array_type(ty)
    if parsed:
        dims, elem = parsed
        # W564: arrays of lowerable packed scalar structs fold the struct width
        # across all dimensions into a single unsigned packed vector.
        if _is_lowerable_scalar_struct_type(ctx, elem):
            base_ws = _packed_type_width_signed(ctx, elem)
            if base_ws is None:
                return None
            total = base_ws[0]
            for d in dims:
                total *= d
            return (total, False)
        # Fixed-size primitive scalar array (1-D or multi-D).
        info = _primitive_array_info(ty)
        if info is not None:
            return (info[2], info[3])
        return None
    # Lowerable packed scalar struct.
    if not _is_lowerable_scalar_struct_type(ctx, ty):
        return None
    decl = ctx.decls.get(f"struct:{ty.strip()}")
    if decl is None:
        return None
    total = 0
    for field in _children(decl):
        ftype = field.get("extra_type", "")
        finfo = _scalar_array_info(ftype)
        if finfo is not None:
            total += finfo[0] * finfo[1]
            continue
        fws = _type_width_signed(ftype)
        if fws is None:
            return None
        total += fws[0]
    return (total, False)


def _is_lowerable_scalar_struct_type(ctx: EvalContext, ty: str) -> bool:
    """Mirror the compiler's notion of a lowerable packed scalar struct."""
    decl = ctx.decls.get(f"struct:{ty.strip()}")
    if decl is None:
        return False
    for field in _children(decl):
        ftype = field.get("extra_type", "")
        if _scalar_array_info(ftype) is not None:
            continue
        if _type_width_signed(ftype) is not None:
            continue
        return False
    return True


def _contains_kind(node: Dict[str, Any], kind: str) -> bool:
    """Recursively check whether `node` or any descendant has the given kind."""
    if node.get("kind") == kind:
        return True
    return any(_contains_kind(child, kind) for child in _children(node))


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

    def __init__(self, root: Dict[str, Any], bind_module_initializers: bool = True) -> None:
        self.root = root
        self.decls = _collect_top_level_decls(root)
        self.vars: Dict[str, Bv] = {}
        # Cache function parameter/return types and local variable types.
        self.fn_param_types: Dict[str, List[Tuple[str, str]]] = {}
        self.fn_return_types: Dict[str, str] = {}
        self.fn_local_types: Dict[str, Dict[str, str]] = {}
        # Track which function we are evaluating so parameter/local declared
        # types can be resolved even when the identifier is bound in vars.
        self.current_fn: Optional[str] = None
        # W547: track test-block local variable types and the current test block
        # so that assertions referencing function-local packed arrays can infer
        # the correct element width/signedness for the VCD cross-check.
        self.test_local_types: Dict[str, Dict[str, str]] = {}
        self.current_block: Optional[str] = None
        for decl in _children(root):
            if decl.get("kind") != "FnDecl":
                continue
            name = decl.get("name", "")
            params = decl.get("params", []) or []
            if name:
                self.fn_param_types[name] = params
                self.fn_return_types[name] = decl.get("extra_return_type", "")
            locals_map: Dict[str, str] = {}
            # Function parameters are not emitted as StmtLocal nodes, but they
            # carry a type annotation in the FnDecl params list.  Record them so
            # field/index access on parameter identifiers resolves correctly.
            for pname, ptype in params:
                if pname and ptype:
                    locals_map[pname] = ptype
            for stmt in _children(decl):
                if stmt.get("kind") == "StmtLocal":
                    vname = stmt.get("name", "")
                    vtype = stmt.get("extra_type", "")
                    if vname and vtype:
                        locals_map[vname] = vtype
            self.fn_local_types[name] = locals_map
        # W541/W543: bind module-level const/var initializers of lowerable packed
        # scalar struct (or fixed-size scalar array) type so that assertions on
        # whole packed values can be independently evaluated.  Track which ones
        # are mutable so whole-struct assignments inside test blocks update the
        # reference model state.
        self.mutable_module_names: set = set()
        if bind_module_initializers:
            for decl in _children(root):
                kind = decl.get("kind")
                if kind not in ("ConstDecl",):
                    continue
                name = decl.get("name", "")
                vtype = decl.get("extra_type", "")
                if not name or not vtype:
                    continue
                if not _is_lowerable_scalar_struct_type(self, vtype) and _scalar_array_info(vtype) is None:
                    continue
                if decl.get("extra_mutable", False):
                    self.mutable_module_names.add(name)
                kids = _children(decl)
                if not kids:
                    continue
                init_node = kids[0]
                init = _eval_expr_bv(self, init_node)
                if init is None:
                    continue
                self.vars[name] = init

    def bind(self, name: str, value: Bv) -> None:
        self.vars[name] = value

    def resolve_var_type(self, name: str) -> Optional[str]:
        if name in self.vars:
            return None  # runtime binding; type is carried by the Bv
        # module-level const/var are both represented as ConstDecl in the AST.
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
            return _packed_type_width_signed(ctx, ty) or _type_width_signed(ty)
        return None
    if kind == "ExprCall":
        ret_ty = ctx.fn_return_types.get(node.get("name", ""), "")
        return _packed_type_width_signed(ctx, ret_ty) or _type_width_signed(ret_ty)
    if kind == "ExprArrayLiteral":
        elem_type = node.get("extra_type", "")
        size_str = node.get("extra_size", "")
        full_ty = f"[{size_str}]{elem_type}" if size_str and elem_type else ""
        # W564: use the packed-vector width for primitive scalar arrays and for
        # arrays of lowerable packed scalar structs.
        packed = _packed_type_width_signed(ctx, full_ty)
        if packed is not None:
            return packed
        info = _primitive_array_info(full_ty)
        if info is not None:
            return (info[2], info[3])
        ws = _type_width_signed(elem_type)
        if ws is None:
            return None
        try:
            count = int(size_str.split("][")[-1])
        except ValueError:
            return None
        return (count * ws[0], ws[1])
    if kind == "ExprStructLit":
        struct_name = node.get("extra_type", "") or node.get("name", "")
        return _packed_type_width_signed(ctx, struct_name)
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
        # W547: use the full declared type (including array dimensions) so that
        # primitive scalar arrays like [3]i8 resolve to the correct element
        # width/signedness.  _resolve_base_type strips dimensions for struct-name
        # lookups; that is the wrong granularity here.
        full_type = _resolve_full_type(ctx, base_name)
        if full_type is None:
            return None
        # Primitive scalar array element.
        parsed = _parse_array_type(full_type)
        if parsed:
            _, elem = parsed
            ws = _type_width_signed(elem)
            if ws:
                return ws
        # Scalar-struct array field element access: base is a field access
        # whose field is a fixed-size scalar array.  For this path the base type
        # is the struct name without array dimensions.
        base_type = _resolve_base_type(ctx, base_name)
        if base_type is not None and base.get("kind") == "ExprFieldAccess":
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


def _collect_index_chain(
    node: Dict[str, Any]
) -> Tuple[Optional[Dict[str, Any]], List[Dict[str, Any]]]:
    """Walk an ExprIndex chain and return (root, [indices in source order]).

    For ``m[i][j]`` the chain is ``ExprIndex(ExprIndex(m, i), j)``; this
    function descends to ``m`` and returns the indices ``[i, j]`` in the order
    they appear in the source.
    """
    indices: List[Dict[str, Any]] = []
    while node.get("kind") == "ExprIndex" and _children(node):
        kids = _children(node)
        if len(kids) >= 2:
            indices.append(kids[1])
        node = kids[0]
    return node, indices


def _resolve_full_type(ctx: EvalContext, name: str) -> Optional[str]:
    """Return the declared type of `name` including array dimensions."""
    if ctx.current_fn:
        local_ty = ctx.fn_local_types.get(ctx.current_fn, {}).get(name)
        if local_ty:
            return local_ty
    if ctx.current_block:
        local_ty = ctx.test_local_types.get(ctx.current_block, {}).get(name)
        if local_ty:
            return local_ty
    c = ctx.decls.get(f"const:{name}")
    if c is not None:
        ty = c.get("extra_type", "")
        if ty:
            return ty
    return ctx.resolve_var_type(name)


def _resolve_base_type(ctx: EvalContext, name: str) -> Optional[str]:
    # W542: function parameters are bound in vars but are not StmtLocal nodes,
    # so resolve their declared type from the current function's local map
    # first.  They shadow module-level names inside the function body.
    if ctx.current_fn:
        local_ty = ctx.fn_local_types.get(ctx.current_fn, {}).get(name)
        if local_ty:
            return _base_type_name(local_ty)
    # W547: test-block local variables (e.g. `let a : [3]i8 = seq();`) carry a
    # type annotation in the StmtLocal node.  Resolve them when evaluating
    # assertions inside the same test block.
    if ctx.current_block:
        local_ty = ctx.test_local_types.get(ctx.current_block, {}).get(name)
        if local_ty:
            return _base_type_name(local_ty)
    # W541: module-level const/var are now bound in ctx.vars, but we still need
    # their declared type for field/index type inference.  Top-level decls
    # always carry the type annotation.
    c = ctx.decls.get(f"const:{name}")
    if c is not None:
        ty = c.get("extra_type", "")
        if ty:
            return _base_type_name(ty)
    # Fallback for unbound identifiers.
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
    """Pack a scalar array literal into a bit-vector (element 0 at LSB).

    W548: handle multi-dimensional literals such as
    ``[2][3]u8{ row0, row1 }`` by concatenating inner packed arrays.  For a
    one-dimensional scalar array each child is masked to the element width
    before packing, matching the compiler's packed-vector layout.
    """
    children = _children(node)
    if not children:
        return None
    elem_type = node.get("extra_type", "")
    size_str = node.get("extra_size", "")
    full_ty = f"[{size_str}]{elem_type}" if size_str and elem_type else ""
    parsed = _parse_array_type(full_ty) if full_ty else None
    if parsed:
        dims, elem = parsed
        elem_ws = _type_width_signed(elem)
        if elem_ws and len(dims) >= 1:
            count = dims[0]
            total_width = elem_ws[0]
            for d in dims:
                total_width *= d
            inner_width = total_width // count
            raw = 0
            off = 0
            for child in children:
                val = _eval_expr_bv(ctx, child)
                if val is None:
                    return None
                mask = (1 << inner_width) - 1
                raw |= (val.value & mask) << off
                off += inner_width
            return Bv(raw, total_width, elem_ws[1])
    # Fallback: recursively concatenate children at their natural widths.
    raw = 0
    width = 0
    signed: Optional[bool] = None
    for child in children:
        val = _eval_expr_bv(ctx, child)
        if val is None:
            return None
        mask = (1 << val.width) - 1
        raw |= (val.value & mask) << width
        width += val.width
        if signed is None:
            signed = val.signed
    if signed is None:
        return None
    return Bv(raw, width, signed)


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
        info = _scalar_array_info(ftype)
        if info is not None:
            fw = info[0] * info[1]
            signed = info[2]
        else:
            ws = _type_width_signed(ftype)
            if ws is None:
                return None
            fw, signed = ws
        val = assigned.get(fname)
        if val is None:
            val = Bv(0, fw, signed)
        # Pack each field at its declared width, masking values that were
        # evaluated at a wider natural width (e.g. integer literals).
        mask = (1 << fw) - 1
        raw |= (val.value & mask) << offset
        offset += fw
        total_width += fw
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
    # W543: create a call-only context so that evaluating the callee body does
    # not re-enter the module-initializer binding loop.  The callee still sees
    # all module-level bindings already established in the outer context.
    call_ctx = EvalContext(ctx.root, bind_module_initializers=False)
    call_ctx.vars.update(ctx.vars)
    call_ctx.current_fn = name
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
    root, indices = _collect_index_chain(node)
    if root is None or not indices:
        return None
    idx_values = [_eval_expr_bv(ctx, idx) for idx in indices]
    if any(v is None for v in idx_values):
        return None
    idxs = [v.as_int() for v in idx_values]

    if root.get("kind") == "ExprIdentifier":
        base_name = root.get("name", "")
        if not base_name:
            return None
        # W547/W548: use the full declared type (including all array dimensions)
        # so that primitive scalar arrays like ``[2][3]i8`` resolve correctly.
        full_type = _resolve_full_type(ctx, base_name)
        if full_type is not None:
            parsed = _parse_array_type(full_type)
            if parsed:
                dims, elem = parsed
                elem_ws = _type_width_signed(elem)
                if elem_ws and len(dims) == len(idxs):
                    # W548: compute row-major flat element index from the full
                    # index chain.  Element ``[i][j]`` of ``[2][3]u8`` is at
                    # flat index ``i * 3 + j``.
                    flat = 0
                    for dim, idx in zip(dims, idxs):
                        if idx < 0 or idx >= dim:
                            return None
                        flat = flat * dim + idx
                    whole_bv = ctx.vars.get(base_name)
                    if whole_bv is None:
                        return None
                    raw = (whole_bv.value >> (flat * elem_ws[0])) & (
                        (1 << elem_ws[0]) - 1
                    )
                    return Bv(raw, *elem_ws)
        return None

    if root.get("kind") == "ExprFieldAccess":
        # Scalar-struct array field element access: ``aos[i].field[j]``.
        # Field arrays are currently one-dimensional.
        if len(idxs) != 1:
            return None
        idx = idxs[0]
        base_name = _base_name(root)
        if base_name is None:
            return None
        base_type = _resolve_base_type(ctx, base_name)
        if base_type is None:
            return None
        field_bv = _eval_field_bv(ctx, root)
        if field_bv is None:
            return None
        ftype = _struct_field_type(ctx.decls, base_type, root.get("name", ""))
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
    width, signed = ws
    raw = src.value
    if width > src.width:
        # Sign-extend signed sources, zero-extend unsigned sources.
        if src.signed and (raw & (1 << (src.width - 1))):
            raw |= ((1 << (width - src.width)) - 1) << src.width
    # Bv.__init__ masks/truncates to the target width.
    return Bv(raw, width, signed)


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


def _collect_assertions(root: Dict[str, Any]) -> List[Tuple[str, str, int, Optional[Any], str, str]]:
    """
    Return list of (block_name, block_kind, index, expected_value, note, probe_name) assertions.

    The expected value is a Python int when the expected expression can be
    statically evaluated; otherwise None with a descriptive note.
    """
    ctx = EvalContext(root)
    out: List[Tuple[str, str, int, Optional[Any], str, str]] = []
    for block in _children(root):
        bkind = block.get("kind")
        if bkind not in ("TestBlock", "InvariantBlock", "BenchBlock"):
            continue
        block_name = block.get("name", "")
        # W547: collect test-block local declarations before processing assertions
        # so that assertions on function-local packed arrays infer the correct
        # element width/signedness and can evaluate the packed value.
        block_locals: Dict[str, Bv] = {}
        block_local_types: Dict[str, str] = {}
        for stmt in _children(block):
            if stmt.get("kind") == "StmtLocal":
                vname = stmt.get("name", "")
                vtype = stmt.get("extra_type", "")
                if not vname or not vtype:
                    continue
                block_local_types[vname] = vtype
                kids = _children(stmt)
                if kids:
                    init_bv = _eval_expr_bv(ctx, kids[0])
                    if init_bv is not None:
                        block_locals[vname] = init_bv
        if block_local_types:
            ctx.test_local_types[block_name] = block_local_types
        # Temporarily bind block-local values while evaluating this block's
        # assertions; restore the outer bindings afterwards.
        saved_vars = {k: ctx.vars[k] for k in block_locals if k in ctx.vars}
        ctx.vars.update(block_locals)
        ctx.current_block = block_name
        idx = 0
        for stmt in _children(block):
            # W541: track whole-struct assignments to mutable module-level vars
            # so subsequent assertions see the updated value in the reference model.
            if stmt.get("kind") == "StmtAssign":
                kids = _children(stmt)
                if len(kids) >= 2 and kids[0].get("kind") == "ExprIdentifier":
                    lhs = kids[0].get("name", "")
                    if lhs in ctx.mutable_module_names:
                        lhs_ty = ctx.resolve_var_type(lhs)
                        if lhs_ty and (
                            _is_lowerable_scalar_struct_type(ctx, lhs_ty)
                            or _scalar_array_info(lhs_ty) is not None
                        ):
                            rhs_bv = _eval_expr_bv(ctx, kids[1])
                            if rhs_bv is not None:
                                ctx.vars[lhs] = rhs_bv
                continue
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
                out.append((block_name, bkind, idx, None, "skipped: assert_eq arity", probe))
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
                out.append((block_name, bkind, idx, expected_bv, "ok", probe))
            else:
                # Fall back to the simple literal evaluator for backwards
                # compatibility with constant-only specs.
                expected = _eval_simple_const(args[1])
                if expected is None:
                    out.append((block_name, bkind, idx, None, "skipped: non-literal expected", probe))
                else:
                    if actual_ws is not None and isinstance(expected, int):
                        expected = Bv(expected, actual_ws[0], actual_ws[1])
                    out.append((block_name, bkind, idx, expected, "ok", probe))
            idx += 1
        # W547: restore outer variable bindings after the test block.
        for k in block_locals:
            if k in saved_vars:
                ctx.vars[k] = saved_vars[k]
            else:
                ctx.vars.pop(k, None)
        ctx.current_block = None
    return out


def _block_has_evaluable_asserts(
    assertions: List[Tuple[str, str, int, Optional[Any], str, str]], block_name: str
) -> bool:
    return any(name == block_name and note == "ok" for name, _, _, _, note, _ in assertions)


def _expected_pass_blocks(
    assertions: List[Tuple[str, str, int, Optional[Any], str, str]]
) -> List[Tuple[str, str]]:
    """Block names and kinds that have at least one evaluable assert_eq."""
    seen: set = set()
    out: List[Tuple[str, str]] = []
    for name, kind, _, _, note, _ in assertions:
        if note == "ok" and name not in seen:
            seen.add(name)
            out.append((name, kind))
    return out


def _status_tag(kind: str) -> str:
    return "BENCH" if kind == "BenchBlock" else "TEST"


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

_STATUS_LINE_RE = re.compile(
    r"\[(TEST|BENCH)\]\s+(.+?)\s*:\s*(starting|PASSED|FAILED)", re.IGNORECASE
)
_PROBE_LINE_RE = re.compile(r"\[PROBE\]\s+(.+?)\s+(\d+)\s*=\s*(\d+)")


def _parse_log(log_text: str) -> Dict[str, Dict[str, Any]]:
    results: Dict[str, Dict[str, Any]] = {}
    for line in log_text.splitlines():
        m = _STATUS_LINE_RE.search(line)
        if not m:
            continue
        tag = m.group(1).upper()
        name, status = m.group(2).strip(), m.group(3).lower()
        key = f"{tag}:{name}"
        entry = results.setdefault(key, {"tag": tag, "started": False, "passed": False, "failed": False})
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
    assertions: List[Tuple[str, str, int, Optional[Any], str, str]],
    log_results: Dict[str, Dict[str, Any]],
    vcd: Optional[_VcdParser],
) -> Tuple[bool, List[str]]:
    errors: List[str] = []
    expected_blocks = _expected_pass_blocks(assertions)
    for block, kind in expected_blocks:
        tag = _status_tag(kind)
        key = f"{tag}:{block}"
        res = log_results.get(key)
        if res is None:
            errors.append(f"missing [{tag}] {block} in simulation log")
            continue
        if res["failed"]:
            errors.append(f"[{tag}] {block} : FAILED")
        elif not res["passed"]:
            errors.append(f"[{tag}] {block} never reported PASSED")
    expected_names = {block for block, _ in expected_blocks}
    for key, res in log_results.items():
        if res["failed"]:
            if res.get("tag", "TEST") == "TEST" and key.split(":", 1)[1] not in expected_names:
                errors.append(f"unexpected [{res.get('tag', 'TEST')}] {key.split(':', 1)[1]} : FAILED")

    if vcd is not None:
        for block_name, _kind, idx, expected, note, probe in assertions:
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
                # W553: the VCD signal width is authoritative for a single-signal
                # probe. The expected literal may be typed wider (e.g. an untyped
                # -1 defaults to 32 bits), so sign-extend from the physical width.
                full_width = width
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
    test_count = sum(1 for _, kind in expected if kind != "BenchBlock")
    bench_count = sum(1 for _, kind in expected if kind == "BenchBlock")
    label_parts = []
    if test_count:
        label_parts.append(f"{test_count} test block(s)")
    if bench_count:
        label_parts.append(f"{bench_count} bench block(s)")
    label = " / ".join(label_parts) if label_parts else "0 blocks"
    print(f"cocotb reference-model OK: {label} passed{vcd_note}")
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

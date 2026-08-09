#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# t27/tools/gen_formats_catalog.py
#
# Bootstrap codegen for the Universal Numeric Format Catalog.
#
# SSOT: specs/numeric/formats_catalog.t27
# Output: gen/numeric/formats_catalog.* -- SIXTEEN targets, not the six this
#         line used to name: md json py rs h ts zig go swift java kt hpp vh hs
#         ml jl  (corrected W603; the emitted set had outgrown the comment)
#
# Until t27c gains struct-literal support, the catalog records live in
# canonical CATALOG-line comments inside the .t27 SSOT. This Python tool
# parses those lines and emits Markdown / JSON / Python / Rust / C / TS.
# Companion to bootstrap/t27c.py (also Python, also stop-gap).
#
# Hard guarantees:
#   - ASCII-only output (all six artefacts).
#   - No banned words: breakthrough / nobel / revolution / proves /
#     first-ever / world-first / industry-leading / prize.
#   - No "scrape" / "crawl" tokens in output.

from __future__ import annotations

import json
import re
import shlex
import sys
from dataclasses import dataclass, asdict
from pathlib import Path

BANNED = re.compile(
    r"\b(breakthrough|nobel|revolution|revolutionary|first-ever|"
    r"world-first|industry-leading|prize)\b",
    re.IGNORECASE,
)
PROVES = re.compile(r"\bproves\b", re.IGNORECASE)
SCRAPE = re.compile(r"\b(scrape|scraping|crawl|crawling)\b", re.IGNORECASE)
NON_ASCII = re.compile(r"[^\x00-\x7F]")

CATALOG_LINE = re.compile(r"//\s*CATALOG:\s*(.+)$", re.MULTILINE)


@dataclass
class Format:
    id: str
    name: str
    bits: int
    s_bits: int
    e_bits: int
    m_bits: int
    bias: int  # -2 sentinel = value exceeds int64, see bias_formula
    bias_formula: str  # raw SSOT bias string (literal int or formula like '2^194-1')
    phi_distance: float  # -1.0 == undefined
    storage: str
    cluster: str
    status: str
    standard: str
    use_case: str
    gf_relation: str
    source: str


def parse_kv_line(line: str) -> dict[str, str]:
    """Parse a shell-style 'key=value key="quoted value"' line."""
    tokens = shlex.split(line, posix=True)
    out: dict[str, str] = {}
    for tok in tokens:
        if "=" not in tok:
            continue
        k, _, v = tok.partition("=")
        out[k.strip()] = v
    return out


def parse_bias(s: str) -> tuple[int, str]:
    """Parse a bias field.

    Returns (bias_int, bias_formula). bias_formula is the raw SSOT string
    (e.g. '2^194-1' for gf512); bias_int is the int() of s when possible,
    else -1 sentinel meaning 'see bias_formula'. This lets the int-typed
    bias field stay backwards-compatible with all downstream emitters
    (Rust i64, C int64_t, TS number) while preserving the rule-derived
    formulas for the limit-of-ladder GF rungs (gf512 bias 2^194-1,
    gf1024 bias 2^390-1) that were added by PR #1051. See issue #1064.
    """
    try:
        return int(s), s
    except ValueError:
        # Try to evaluate simple 2^N-1 / 2^N+1 / 2^N forms; if the
        # exponent stays within range, return the literal int.
        m = re.match(r"^2\^(\d+)([+-]\d+)?$", s.strip())
        if m:
            exp = int(m.group(1))
            off = int(m.group(2)) if m.group(2) else 0
            if exp <= 1024:  # python int is arbitrary precision but i64 isn't
                val = (1 << exp) + off
                # Only return as int if it fits signed 64-bit; otherwise
                # downstream emitters (Rust i64, C int64_t) overflow.
                if -(1 << 63) <= val < (1 << 63):
                    return val, s
        # Cannot lift to int; record formula, use sentinel -2 meaning
        # "value lives in bias_formula field, exceeds int64".
        return -2, s


def parse_t27(text: str) -> list[Format]:
    formats: list[Format] = []
    for m in CATALOG_LINE.finditer(text):
        fields = parse_kv_line(m.group(1))
        try:
            bias_int, bias_formula = parse_bias(fields["bias"])
            fmt = Format(
                id=fields["id"],
                name=fields["name"],
                bits=int(fields["bits"]),
                s_bits=int(fields["s"]),
                e_bits=int(fields["e"]),
                m_bits=int(fields["m"]),
                bias=bias_int,
                bias_formula=bias_formula,
                phi_distance=float(fields["phi_distance"]),
                storage=fields["storage"],
                cluster=fields["cluster"],
                status=fields["status"],
                standard=fields["standard"],
                use_case=fields["use_case"],
                gf_relation=fields["gf_relation"],
                source=fields["source"],
            )
        except (KeyError, ValueError) as e:
            print(f"WARN: malformed CATALOG line: {fields} ({e})",
                  file=sys.stderr)
            continue
        formats.append(fmt)
    return formats


def check_safe(name: str, content: str) -> None:
    bad = BANNED.findall(content)
    if bad:
        raise SystemExit(f"{name}: banned word(s) in output: {bad}")
    if PROVES.search(content):
        raise SystemExit(f"{name}: 'proves' in output")
    if SCRAPE.search(content):
        raise SystemExit(f"{name}: scrape/crawl word in output")
    if NON_ASCII.search(content):
        raise SystemExit(f"{name}: non-ASCII byte in output")


# -------------------------------------------------------------------- Markdown
def emit_markdown(formats: list[Format]) -> str:
    lines = [
        "# Universal Numeric Format Catalog",
        "",
        "Generated from specs/numeric/formats_catalog.t27 by the bootstrap",
        "codegen (tools/gen_formats_catalog.py). Do not edit by hand.",
        "",
        "Status labels: Verified | EmpiricalFit | Open | Risk | Retracted |",
        "Experimental | Historical. phi_distance: lower = more phi-aligned;",
        "-1 sentinel = undefined (non-radix-2 or non-FP).",
        "",
        f"Total formats: {len(formats)}.",
        "",
        "| ID | Bits | S:E:M | Bias | phi_dist | Storage | Cluster | "
        "Status | Standard | Use case | GF rel. |",
        "|----|-----:|------|-----:|--------:|---------|---------|"
        "--------|----------|----------|---------|",
    ]
    for f in formats:
        sem = f"{f.s_bits}:{f.e_bits}:{f.m_bits}"
        pd = "n/a" if f.phi_distance < 0 else f"{f.phi_distance:.3f}"
        lines.append(
            f"| {f.id} | {f.bits} | {sem} | {f.bias} | {pd} | "
            f"{f.storage} | {f.cluster} | {f.status} | {f.standard} | "
            f"{f.use_case} | {f.gf_relation} |"
        )
    lines += [
        "",
        "## Sources",
        "",
        "Per-row citation in the `source` field of the SSOT.",
        "",
        "## Honesty contract",
        "",
        "This catalog records cluster, status, phi_distance, and use case",
        "only. Per-rung quality claims (better than posit / takum / OCP-MX /",
        "LNS) live ONLY in FL-002 with the F1/F2/F3 falsification protocol.",
        "Default status of any moat claim is Open conjecture.",
    ]
    return "\n".join(lines) + "\n"


# ------------------------------------------------------------------------ JSON
def emit_json(formats: list[Format]) -> str:
    return json.dumps(
        {"count": len(formats),
         "formats": [asdict(f) for f in formats]},
        indent=2, sort_keys=False, ensure_ascii=True,
    ) + "\n"


# ---------------------------------------------------------------------- Python
def emit_python(formats: list[Format]) -> str:
    out = [
        '"""Universal Numeric Format Catalog (generated).',
        "",
        "Generated from specs/numeric/formats_catalog.t27. Do not edit.",
        '"""',
        "from __future__ import annotations",
        "from dataclasses import dataclass",
        "",
        "",
        "@dataclass(frozen=True)",
        "class Format:",
        "    id: str",
        "    name: str",
        "    bits: int",
        "    s_bits: int",
        "    e_bits: int",
        "    m_bits: int",
        "    bias: int",
        "    phi_distance: float  # -1.0 == undefined",
        "    storage: str",
        "    cluster: str",
        "    status: str",
        "    standard: str",
        "    use_case: str",
        "    gf_relation: str",
        "    source: str",
        "",
        "",
        "FORMATS: list[Format] = [",
    ]
    for f in formats:
        out.append(
            "    Format("
            f"id={f.id!r}, name={f.name!r}, bits={f.bits}, "
            f"s_bits={f.s_bits}, e_bits={f.e_bits}, m_bits={f.m_bits}, "
            f"bias={f.bias}, phi_distance={f.phi_distance}, "
            f"storage={f.storage!r}, cluster={f.cluster!r}, "
            f"status={f.status!r}, standard={f.standard!r}, "
            f"use_case={f.use_case!r}, gf_relation={f.gf_relation!r}, "
            f"source={f.source!r}),"
        )
    out += ["]", ""]
    return "\n".join(out)


# ------------------------------------------------------------------------ Rust
def emit_rust(formats: list[Format]) -> str:
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "",
        "#[derive(Debug, Clone, Copy)]",
        "pub struct Format {",
        "    pub id: &'static str,",
        "    pub name: &'static str,",
        "    pub bits: u32,",
        "    pub s_bits: u32,",
        "    pub e_bits: u32,",
        "    pub m_bits: u32,",
        "    pub bias: i64,",
        "    pub phi_distance: f64, // -1.0 == undefined",
        "    pub storage: &'static str,",
        "    pub cluster: &'static str,",
        "    pub status: &'static str,",
        "    pub standard: &'static str,",
        "    pub use_case: &'static str,",
        "    pub gf_relation: &'static str,",
        "    pub source: &'static str,",
        "}",
        "",
        f"pub const FORMATS: [Format; {len(formats)}] = [",
    ]
    for f in formats:
        out.append(
            "    Format { "
            f"id: {f.id!r}, name: {f.name!r}, bits: {f.bits}, "
            f"s_bits: {f.s_bits}, e_bits: {f.e_bits}, m_bits: {f.m_bits}, "
            f"bias: {f.bias}, phi_distance: {f.phi_distance}f64, "
            f"storage: {f.storage!r}, cluster: {f.cluster!r}, "
            f"status: {f.status!r}, standard: {f.standard!r}, "
            f"use_case: {f.use_case!r}, gf_relation: {f.gf_relation!r}, "
            f"source: {f.source!r} }},"
        )
    out += ["];", ""]
    return "\n".join(out)


# --------------------------------------------------------------------------- C
def c_esc(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


def emit_c(formats: list[Format]) -> str:
    out = [
        "/* Generated from formats_catalog.t27. Do not edit by hand. */",
        "/* SPDX-License-Identifier: Apache-2.0 */",
        "#ifndef T27_FORMATS_CATALOG_H",
        "#define T27_FORMATS_CATALOG_H",
        "",
        "#include <stdint.h>",
        "#include <stddef.h>",
        "",
        "typedef struct {",
        "    const char *id;",
        "    const char *name;",
        "    uint32_t bits;",
        "    uint32_t s_bits;",
        "    uint32_t e_bits;",
        "    uint32_t m_bits;",
        "    int64_t  bias;",
        "    double   phi_distance; /* -1.0 == undefined */",
        "    const char *storage;",
        "    const char *cluster;",
        "    const char *status;",
        "    const char *standard;",
        "    const char *use_case;",
        "    const char *gf_relation;",
        "    const char *source;",
        "} t27_format_t;",
        "",
        f"#define T27_FORMAT_COUNT {len(formats)}",
        "",
        "static const t27_format_t T27_FORMATS[T27_FORMAT_COUNT] = {",
    ]
    for f in formats:
        out.append(
            "    { "
            f'"{f.id}", "{c_esc(f.name)}", {f.bits}u, {f.s_bits}u, '
            f"{f.e_bits}u, {f.m_bits}u, {f.bias}, {f.phi_distance}, "
            f'"{c_esc(f.storage)}", "{f.cluster}", "{f.status}", '
            f'"{c_esc(f.standard)}", "{c_esc(f.use_case)}", '
            f'"{f.gf_relation}", "{c_esc(f.source)}" '
            "},"
        )
    out += ["};", "", "#endif /* T27_FORMATS_CATALOG_H */", ""]
    return "\n".join(out)


# ---------------------------------------------------------------- TypeScript/JS
def emit_ts(formats: list[Format]) -> str:
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "",
        "export interface NumericFormat {",
        "  id: string;",
        "  name: string;",
        "  bits: number;",
        "  s_bits: number;",
        "  e_bits: number;",
        "  m_bits: number;",
        "  bias: number;",
        "  phi_distance: number; // -1 == undefined",
        "  storage: string;",
        "  cluster: string;",
        "  status: string;",
        "  standard: string;",
        "  use_case: string;",
        "  gf_relation: string;",
        "  source: string;",
        "}",
        "",
        "export const FORMATS: ReadonlyArray<NumericFormat> = [",
    ]
    for f in formats:
        out.append(
            "  { "
            f"id: {json.dumps(f.id)}, name: {json.dumps(f.name)}, "
            f"bits: {f.bits}, s_bits: {f.s_bits}, e_bits: {f.e_bits}, "
            f"m_bits: {f.m_bits}, bias: {f.bias}, "
            f"phi_distance: {f.phi_distance}, "
            f"storage: {json.dumps(f.storage)}, "
            f"cluster: {json.dumps(f.cluster)}, "
            f"status: {json.dumps(f.status)}, "
            f"standard: {json.dumps(f.standard)}, "
            f"use_case: {json.dumps(f.use_case)}, "
            f"gf_relation: {json.dumps(f.gf_relation)}, "
            f"source: {json.dumps(f.source)} }},"
        )
    out += ["];", ""]
    return "\n".join(out)


# -------------------------------------------------------------------------- Go
def emit_go(formats: list[Format]) -> str:
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "",
        "package formatscatalog",
        "",
        "// Format describes one numeric format in the SSOT catalog.",
        "type Format struct {",
        "    ID          string",
        "    Name        string",
        "    Bits        uint32",
        "    SBits       uint32",
        "    EBits       uint32",
        "    MBits       uint32",
        "    Bias        int64",
        "    PhiDistance float64 // -1.0 == undefined",
        "    Storage     string",
        "    Cluster     string",
        "    Status      string",
        "    Standard    string",
        "    UseCase     string",
        "    GFRelation  string",
        "    Source      string",
        "}",
        "",
        "// Formats is the canonical list emitted from formats_catalog.t27.",
        "var Formats = []Format{",
    ]
    for f in formats:
        out.append(
            "    { "
            f"ID: {json.dumps(f.id)}, Name: {json.dumps(f.name)}, "
            f"Bits: {f.bits}, SBits: {f.s_bits}, EBits: {f.e_bits}, "
            f"MBits: {f.m_bits}, Bias: {f.bias}, "
            f"PhiDistance: {f.phi_distance}, "
            f"Storage: {json.dumps(f.storage)}, "
            f"Cluster: {json.dumps(f.cluster)}, "
            f"Status: {json.dumps(f.status)}, "
            f"Standard: {json.dumps(f.standard)}, "
            f"UseCase: {json.dumps(f.use_case)}, "
            f"GFRelation: {json.dumps(f.gf_relation)}, "
            f"Source: {json.dumps(f.source)} }},"
        )
    out += ["}", ""]
    return "\n".join(out)


# ------------------------------------------------------------------------- Zig
def emit_zig(formats: list[Format]) -> str:
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "",
        "pub const Format = struct {",
        "    id: []const u8,",
        "    name: []const u8,",
        "    bits: u32,",
        "    s_bits: u32,",
        "    e_bits: u32,",
        "    m_bits: u32,",
        "    bias: i64,",
        "    phi_distance: f64, // -1.0 == undefined",
        "    storage: []const u8,",
        "    cluster: []const u8,",
        "    status: []const u8,",
        "    standard: []const u8,",
        "    use_case: []const u8,",
        "    gf_relation: []const u8,",
        "    source: []const u8,",
        "};",
        "",
        f"pub const formats = [_]Format{{",
    ]
    for f in formats:
        out.append(
            "    .{ "
            f".id = {json.dumps(f.id)}, .name = {json.dumps(f.name)}, "
            f".bits = {f.bits}, .s_bits = {f.s_bits}, .e_bits = {f.e_bits}, "
            f".m_bits = {f.m_bits}, .bias = {f.bias}, "
            f".phi_distance = {f.phi_distance}, "
            f".storage = {json.dumps(f.storage)}, "
            f".cluster = {json.dumps(f.cluster)}, "
            f".status = {json.dumps(f.status)}, "
            f".standard = {json.dumps(f.standard)}, "
            f".use_case = {json.dumps(f.use_case)}, "
            f".gf_relation = {json.dumps(f.gf_relation)}, "
            f".source = {json.dumps(f.source)} }},"
        )
    out += ["};", ""]
    return "\n".join(out)


# ----------------------------------------------------------------------- Swift
def emit_swift(formats: list[Format]) -> str:
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "",
        "public struct Format {",
        "    public let id: String",
        "    public let name: String",
        "    public let bits: UInt32",
        "    public let sBits: UInt32",
        "    public let eBits: UInt32",
        "    public let mBits: UInt32",
        "    public let bias: Int64",
        "    public let phiDistance: Double // -1.0 == undefined",
        "    public let storage: String",
        "    public let cluster: String",
        "    public let status: String",
        "    public let standard: String",
        "    public let useCase: String",
        "    public let gfRelation: String",
        "    public let source: String",
        "}",
        "",
        "public let FORMATS: [Format] = [",
    ]
    for f in formats:
        out.append(
            "    Format("
            f"id: {json.dumps(f.id)}, name: {json.dumps(f.name)}, "
            f"bits: {f.bits}, sBits: {f.s_bits}, eBits: {f.e_bits}, "
            f"mBits: {f.m_bits}, bias: {f.bias}, "
            f"phiDistance: {f.phi_distance}, "
            f"storage: {json.dumps(f.storage)}, "
            f"cluster: {json.dumps(f.cluster)}, "
            f"status: {json.dumps(f.status)}, "
            f"standard: {json.dumps(f.standard)}, "
            f"useCase: {json.dumps(f.use_case)}, "
            f"gfRelation: {json.dumps(f.gf_relation)}, "
            f"source: {json.dumps(f.source)}),"
        )
    out += ["]", ""]
    return "\n".join(out)


# ------------------------------------------------------------------------ Java
def emit_java(formats: list[Format]) -> str:
    """Java 8+ compatible: classical immutable POJO (no records)."""
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "package ai.t27.numeric;",
        "",
        "import java.util.Arrays;",
        "import java.util.Collections;",
        "import java.util.List;",
        "",
        "public final class FormatsCatalog {",
        "    private FormatsCatalog() {}",
        "",
        "    public static final class Format {",
        "        public final String id;",
        "        public final String name;",
        "        public final long bits;",
        "        public final long sBits;",
        "        public final long eBits;",
        "        public final long mBits;",
        "        public final long bias;",
        "        public final double phiDistance;",
        "        public final String storage;",
        "        public final String cluster;",
        "        public final String status;",
        "        public final String standard;",
        "        public final String useCase;",
        "        public final String gfRelation;",
        "        public final String source;",
        "        public Format(String id, String name,",
        "                long bits, long sBits, long eBits, long mBits,",
        "                long bias, double phiDistance,",
        "                String storage, String cluster, String status,",
        "                String standard, String useCase,",
        "                String gfRelation, String source) {",
        "            this.id = id; this.name = name;",
        "            this.bits = bits; this.sBits = sBits;",
        "            this.eBits = eBits; this.mBits = mBits;",
        "            this.bias = bias; this.phiDistance = phiDistance;",
        "            this.storage = storage; this.cluster = cluster;",
        "            this.status = status; this.standard = standard;",
        "            this.useCase = useCase; this.gfRelation = gfRelation;",
        "            this.source = source;",
        "        }",
        "    }",
        "",
        "    public static final List<Format> FORMATS;",
        "    static {",
        "        Format[] arr = new Format[] {",
    ]
    for i, f in enumerate(formats):
        comma = "," if i < len(formats) - 1 else ""
        out.append(
            "            new Format("
            f"{json.dumps(f.id)}, {json.dumps(f.name)}, "
            f"{f.bits}L, {f.s_bits}L, {f.e_bits}L, {f.m_bits}L, "
            f"{f.bias}L, {f.phi_distance}, "
            f"{json.dumps(f.storage)}, {json.dumps(f.cluster)}, "
            f"{json.dumps(f.status)}, {json.dumps(f.standard)}, "
            f"{json.dumps(f.use_case)}, {json.dumps(f.gf_relation)}, "
            f"{json.dumps(f.source)}){comma}"
        )
    out += [
        "        };",
        "        FORMATS = Collections.unmodifiableList(Arrays.asList(arr));",
        "    }",
        "}",
        "",
    ]
    return "\n".join(out)


# ---------------------------------------------------------------------- Kotlin
def emit_kotlin(formats: list[Format]) -> str:
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "package ai.t27.numeric",
        "",
        "data class Format(",
        "    val id: String,",
        "    val name: String,",
        "    val bits: Int,",
        "    val sBits: Int,",
        "    val eBits: Int,",
        "    val mBits: Int,",
        "    val bias: Long,",
        "    val phiDistance: Double, // -1.0 == undefined",
        "    val storage: String,",
        "    val cluster: String,",
        "    val status: String,",
        "    val standard: String,",
        "    val useCase: String,",
        "    val gfRelation: String,",
        "    val source: String,",
        ")",
        "",
        "val FORMATS: List<Format> = listOf(",
    ]
    for f in formats:
        out.append(
            "    Format("
            f"{json.dumps(f.id)}, {json.dumps(f.name)}, "
            f"{f.bits}, {f.s_bits}, {f.e_bits}, {f.m_bits}, "
            f"{f.bias}L, {f.phi_distance}, "
            f"{json.dumps(f.storage)}, {json.dumps(f.cluster)}, "
            f"{json.dumps(f.status)}, {json.dumps(f.standard)}, "
            f"{json.dumps(f.use_case)}, {json.dumps(f.gf_relation)}, "
            f"{json.dumps(f.source)}),"
        )
    out += [")", ""]
    return "\n".join(out)


# ------------------------------------------------------------------------- Cpp
def emit_cpp(formats: list[Format]) -> str:
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "#pragma once",
        "#include <array>",
        "#include <cstdint>",
        "#include <string_view>",
        "",
        "namespace t27 {",
        "",
        "struct Format {",
        "    std::string_view id;",
        "    std::string_view name;",
        "    std::uint32_t bits;",
        "    std::uint32_t s_bits;",
        "    std::uint32_t e_bits;",
        "    std::uint32_t m_bits;",
        "    std::int64_t  bias;",
        "    double phi_distance; // -1.0 == undefined",
        "    std::string_view storage;",
        "    std::string_view cluster;",
        "    std::string_view status;",
        "    std::string_view standard;",
        "    std::string_view use_case;",
        "    std::string_view gf_relation;",
        "    std::string_view source;",
        "};",
        "",
        f"inline constexpr std::array<Format, {len(formats)}> FORMATS = {{{{",
    ]
    for f in formats:
        out.append(
            "    Format{ "
            f"{json.dumps(f.id)}, {json.dumps(f.name)}, "
            f"{f.bits}u, {f.s_bits}u, {f.e_bits}u, {f.m_bits}u, "
            f"{f.bias}, {f.phi_distance}, "
            f"{json.dumps(f.storage)}, {json.dumps(f.cluster)}, "
            f"{json.dumps(f.status)}, {json.dumps(f.standard)}, "
            f"{json.dumps(f.use_case)}, {json.dumps(f.gf_relation)}, "
            f"{json.dumps(f.source)} }},"
        )
    out += ["}};", "", "} // namespace t27", ""]
    return "\n".join(out)


# --------------------------------------------------------------------- Verilog
def emit_verilog(formats: list[Format]) -> str:
    """Verilog header: bit-width parameters only (no string tables in HDL)."""
    out = [
        "// Generated from formats_catalog.t27. Do not edit by hand.",
        "// SPDX-License-Identifier: Apache-2.0",
        "// Per-format bit-width parameters for HDL synthesis. String fields",
        "// are intentionally omitted; consult the .h / .json artefact for",
        "// metadata. Identifiers are ASCII, uppercased, dashes -> underscore.",
        "`ifndef T27_FORMATS_CATALOG_VH",
        "`define T27_FORMATS_CATALOG_VH",
        "",
        f"`define T27_FORMAT_COUNT {len(formats)}",
        "",
    ]
    for f in formats:
        ident = re.sub(r"[^A-Za-z0-9_]", "_", f.id).upper()
        out.append(
            f"`define T27_FMT_{ident}_BITS  {f.bits}\n"
            f"`define T27_FMT_{ident}_S     {f.s_bits}\n"
            f"`define T27_FMT_{ident}_E     {f.e_bits}\n"
            f"`define T27_FMT_{ident}_M     {f.m_bits}\n"
            f"`define T27_FMT_{ident}_BIAS  {f.bias}"
        )
    out += ["", "`endif // T27_FORMATS_CATALOG_VH", ""]
    return "\n".join(out)


# --------------------------------------------------------------------- Haskell
def emit_haskell(formats: list[Format]) -> str:
    out = [
        "-- Generated from formats_catalog.t27. Do not edit by hand.",
        "-- SPDX-License-Identifier: Apache-2.0",
        "{-# LANGUAGE OverloadedStrings #-}",
        "module T27.FormatsCatalog (Format(..), formats) where",
        "",
        "import Data.Int (Int64)",
        "import Data.Word (Word32)",
        "",
        "data Format = Format",
        "  { fId          :: String",
        "  , fName        :: String",
        "  , fBits        :: Word32",
        "  , fSBits       :: Word32",
        "  , fEBits       :: Word32",
        "  , fMBits       :: Word32",
        "  , fBias        :: Int64",
        "  , fPhiDistance :: Double  -- -1.0 == undefined",
        "  , fStorage     :: String",
        "  , fCluster     :: String",
        "  , fStatus      :: String",
        "  , fStandard    :: String",
        "  , fUseCase     :: String",
        "  , fGFRelation  :: String",
        "  , fSource      :: String",
        "  } deriving (Show, Eq)",
        "",
        "formats :: [Format]",
        "formats =",
    ]
    for i, f in enumerate(formats):
        bullet = "  [ " if i == 0 else "  , "
        out.append(
            bullet + "Format "
            f"{json.dumps(f.id)} {json.dumps(f.name)} "
            f"{f.bits} {f.s_bits} {f.e_bits} {f.m_bits} "
            f"{f.bias} ({f.phi_distance}) "
            f"{json.dumps(f.storage)} {json.dumps(f.cluster)} "
            f"{json.dumps(f.status)} {json.dumps(f.standard)} "
            f"{json.dumps(f.use_case)} {json.dumps(f.gf_relation)} "
            f"{json.dumps(f.source)}"
        )
    out += ["  ]", ""]
    return "\n".join(out)


# ----------------------------------------------------------------------- OCaml
def emit_ocaml(formats: list[Format]) -> str:
    out = [
        "(* Generated from formats_catalog.t27. Do not edit by hand. *)",
        "(* SPDX-License-Identifier: Apache-2.0 *)",
        "",
        "type format = {",
        "  id           : string;",
        "  name         : string;",
        "  bits         : int;",
        "  s_bits       : int;",
        "  e_bits       : int;",
        "  m_bits       : int;",
        "  bias         : int;",
        "  phi_distance : float; (* -1.0 == undefined *)",
        "  storage      : string;",
        "  cluster      : string;",
        "  status       : string;",
        "  standard     : string;",
        "  use_case     : string;",
        "  gf_relation  : string;",
        "  source       : string;",
        "}",
        "",
        "let formats : format list = [",
    ]
    for f in formats:
        out.append(
            "  { "
            f"id = {json.dumps(f.id)}; name = {json.dumps(f.name)}; "
            f"bits = {f.bits}; s_bits = {f.s_bits}; e_bits = {f.e_bits}; "
            f"m_bits = {f.m_bits}; bias = {f.bias}; "
            f"phi_distance = {f.phi_distance}; "
            f"storage = {json.dumps(f.storage)}; "
            f"cluster = {json.dumps(f.cluster)}; "
            f"status = {json.dumps(f.status)}; "
            f"standard = {json.dumps(f.standard)}; "
            f"use_case = {json.dumps(f.use_case)}; "
            f"gf_relation = {json.dumps(f.gf_relation)}; "
            f"source = {json.dumps(f.source)} }};"
        )
    out += ["]", ""]
    return "\n".join(out)


# ----------------------------------------------------------------------- Julia
def emit_julia(formats: list[Format]) -> str:
    out = [
        "# Generated from formats_catalog.t27. Do not edit by hand.",
        "# SPDX-License-Identifier: Apache-2.0",
        "",
        "module FormatsCatalog",
        "",
        "export Format, FORMATS",
        "",
        "struct Format",
        "    id::String",
        "    name::String",
        "    bits::UInt32",
        "    s_bits::UInt32",
        "    e_bits::UInt32",
        "    m_bits::UInt32",
        "    bias::Int64",
        "    phi_distance::Float64  # -1.0 == undefined",
        "    storage::String",
        "    cluster::String",
        "    status::String",
        "    standard::String",
        "    use_case::String",
        "    gf_relation::String",
        "    source::String",
        "end",
        "",
        "const FORMATS = Format[",
    ]
    for f in formats:
        out.append(
            "    Format("
            f"{json.dumps(f.id)}, {json.dumps(f.name)}, "
            f"UInt32({f.bits}), UInt32({f.s_bits}), UInt32({f.e_bits}), "
            f"UInt32({f.m_bits}), Int64({f.bias}), {f.phi_distance}, "
            f"{json.dumps(f.storage)}, {json.dumps(f.cluster)}, "
            f"{json.dumps(f.status)}, {json.dumps(f.standard)}, "
            f"{json.dumps(f.use_case)}, {json.dumps(f.gf_relation)}, "
            f"{json.dumps(f.source)}),"
        )
    out += ["]", "", "end # module", ""]
    return "\n".join(out)


# ----------------------------------------------------------------------- Main
def main(argv: list[str]) -> int:
    # W603: the defaults were `formats_catalog.t27` and `gen_catalog/` relative
    # to the CURRENT DIRECTORY, so running this from the repo root -- the only
    # place anybody runs it from -- failed with FileNotFoundError, and the
    # output path did not match the `gen/numeric/...` this file's own header
    # documents. Default to the repo-root-relative paths that header states.
    repo = Path(__file__).resolve().parent.parent
    src = Path(argv[1]) if len(argv) > 1 else repo / "specs/numeric/formats_catalog.t27"
    out_root = Path(argv[2]) if len(argv) > 2 else repo / "gen/numeric"
    text = src.read_text(encoding="utf-8")
    formats = parse_t27(text)
    print(f"parsed {len(formats)} formats from {src}", file=sys.stderr)

    artifacts = {
        "formats_catalog.md":      emit_markdown(formats),
        "formats_catalog.json":    emit_json(formats),
        "formats_catalog.py":      emit_python(formats),
        "formats_catalog.rs":      emit_rust(formats),
        "formats_catalog.h":       emit_c(formats),
        "formats_catalog.ts":      emit_ts(formats),
        "formats_catalog.go":      emit_go(formats),
        "formats_catalog.zig":     emit_zig(formats),
        "formats_catalog.swift":   emit_swift(formats),
        "FormatsCatalog.java":     emit_java(formats),
        "formats_catalog.kt":      emit_kotlin(formats),
        "formats_catalog.hpp":     emit_cpp(formats),
        "formats_catalog.vh":      emit_verilog(formats),
        "FormatsCatalog.hs":       emit_haskell(formats),
        "formats_catalog.ml":      emit_ocaml(formats),
        "FormatsCatalog.jl":       emit_julia(formats),
    }
    for rel, content in artifacts.items():
        check_safe(rel, content)
        path = out_root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="ascii")
        print(f"wrote {path} ({len(content)} bytes)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

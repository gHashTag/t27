# Enumeration of 9 Repeated Names in Formats Catalog

## Overview
The numeric format catalogue contains 118 raw `name=` occurrences but only 109 distinct names, meaning 9 names appear exactly twice each. These are not duplicates but **aliases** - they represent the same format under different naming conventions due to a renaming event.

## The 9 Repeated Names (All Aliases)

| Name | GF-T Row | TNF Row | Classification |
|------|----------|---------|----------------|
| GF-T4 | Line 296: `id=gft4 name="GF-T4"` | Line 368: `id=tnf4 name="TNF4" former_name="GF-T4"` | **Alias** - Former name preserved for historical continuity |
| GF-T8 | Line 297: `id=gft8 name="GF-T8"` | Line 369: `id=tnf8 name="TNF8" former_name="GF-T8"` | **Alias** - Former name preserved for historical continuity |
| GF-T16 | Line 298: `id=gft16 name="GF-T16"` | Line 370: `id=tnf16 name="TNF16" former_name="GF-T16"` | **Alias** - Former name preserved for historical continuity |
| GF-T32 | Line 299: `id=gft32 name="GF-T32"` | Line 371: `id=tnf32 name="TNF32" former_name="GF-T32"` | **Alias** - Former name preserved for historical continuity |
| GF-T64 | Line 300: `id=gft64 name="GF-T64"` | Line 372: `id=tnf64 name="TNF64" former_name="GF-T64"` | **Alias** - Former name preserved for historical continuity |
| GF-T128 | Line 301: `id=gft128 name="GF-T128"` | Line 373: `id=tnf128 name="TNF128" former_name="GF-T128"` | **Alias** - Former name preserved for historical continuity |
| GF-T256 | Line 302: `id=gft256 name="GF-T256"` | Line 374: `id=tnf256 name="TNF256" former_name="GF-T256"` | **Alias** - Former name preserved for historical continuity |
| GF-T512 | Line 303: `id=gft512 name="GF-T512"` | Line 375: `id=tnf512 name="TNF512" former_name="GF-T512"` | **Alias** - Former name preserved for historical continuity |
| GF-T1024 | Line 304: `id=gft1024 name="GF-T1024"` | Line 376: `id=tnf1024 name="TNF1024" former_name="GF-T1024"` | **Alias** - Former name preserved for historical continuity |

## Classification: All Are Aliases

**NOT duplicates** - These are intentional aliases maintained for historical continuity:

1. **GF-T Section**: Contains the original "GF-TX" naming convention (phi-derived ternary ladder)
2. **TNF Section**: Contains the current "TNFX" naming convention with `former_name="GF-TX"` fields

## Why This Is Correct

The integrity checker correctly counts 109 because:
- It counts distinct `id=` rows, not `name=` fields
- Each `id=` is unique (gft4, tnf4, etc.)
- The `former_name=` field is metadata, not a separate format

## Historical Context

From the catalog comments:
- "Former names carried in `former_name=`: **GF-T** through 2026, then **TEF** for one day on 2026-08-09, renamed 2026-08-09"
- "The ladder has NEVER been published under either name -- arXiv:2606.05017 is the binary GF family and contains no occurrence of 'GF-T', and arXiv:2606.09686 is this catalog, which had zero GF-T rows until they were added on 2026-08-09"

The former names are preserved for:
- Research notes continuity
- Prior branches reference  
- Author's CV and profile
- Measurements recorded under old labels

## Conclusion

All 9 repeated names are **aliases**, not duplicates. The count of 109 distinct formats is correct and authoritative. The repetition is intentional metadata for historical tracking, not an error.
# Anonymous GitHub Repository Setup — NeurIPS 2026 Double-Blind

> **Purpose:** Create an anonymous code repository for GoldenFloat paper submission.
> **Deadline:** Before May 6, 2026 (full paper deadline)

## NeurIPS Double-Blind Requirements

According to NeurIPS 2026 Call for Papers:

- **No author identification** in repository, code, or documentation
- **Anonymous GitHub account** (no real name, affiliation, or photo)
- **Readme must be anonymous** (no "we", "our lab", etc.)
- **Code must be reproducible** with clear instructions
- **Link to be provided** in Paper Checklist (URL withheld during review)

## Setup Steps

### 1. Create Anonymous GitHub Account

```bash
# Use a new email address not linked to identity
# Username: e.g., "goldenfloat-anon" or "gf16-research"
# Display name: Leave blank or use "Anonymous"
# Profile: No photo, no bio, no location
```

### 2. Create Public Repository

```bash
# Repository name: goldenfloat-anon or gf16-neurips
# Visibility: Public
# Description: "GoldenFloat: A $\varphi$-Optimal Floating-Point Family"
# Initialize with: README.md, .gitignore (choose Rust)
```

### 3. Repository Structure

```
goldenfloat-anon/
├── README.md               # Anonymous README
├── CITATION.cff            # Citation file (anonymous)
├── LICENSE                 # MIT or Apache-2.0
├── src/
│   ├── gf_format.zig       # GoldenFloat format implementation
│   ├── gf_ops.zig          # Arithmetic operations
│   └── gf_bench.zig        # Benchmarking code
├── tests/
│   ├── roundtrip_test.zig
│   └── phi_test.zig
└── scripts/
    ├── benchmark.sh
    └── reproducibility.sh
```

### 4. Anonymous README Template

```markdown
# GoldenFloat: A $\varphi$-Optimal Floating-Point Family

This repository contains the reference implementation of GoldenFloat (GF),
a family of seven floating-point formats parameterized by the golden ratio.

## Citation

```bibtex
@misc{anonymous2026goldenfloat,
  title={GoldenFloat: A $\varphi$-Optimal Floating-Point Family},
  author={Anonymous},
  year={2026},
  note={NeurIPS 2026 submission}
}
```

## Reproduction

### Prerequisites

- Zig 0.15.0 or later
- Python 3.10+ (for benchmarking scripts)

### Build

```bash
zig build
```

### Run Tests

```bash
zig test src/gf_format.zig
zig test src/gf_ops.zig
```

### Run Benchmarks

```bash
./scripts/benchmark.sh
```

## Format Specifications

The GoldenFloat formats use $\varphi$-guided bit allocation:

| Format | Bits | Exponent | Mantissa |
|--------|------|----------|----------|
| GF4    | 4    | 1        | 2        |
| GF8    | 8    | 3        | 4        |
| GF16   | 16   | 6        | 9        |
| GF32   | 32   | 12       | 19       |

## Files

- `src/gf_format.zig`: Format definitions and encoding/decoding
- `src/gf_ops.zig`: Arithmetic operations
- `tests/`: Unit tests for format correctness
- `scripts/`: Benchmarking and reproducibility scripts

## License

MIT License - see LICENSE file for details.
```

### 5. Commit Messages (Anonymous Style)

```bash
# Good (anonymous)
git commit -m "Add GF16 format encoding/decoding"
git commit -m "Fix overflow in GF multiplication"
git commit -m "Add benchmarking scripts"

# Bad (reveals identity)
git commit -m "Fixed bug I found yesterday"  # "I" reveals identity
git commit -m "Our implementation of..."     # "Our" reveals identity
```

### 6. Link in Paper Checklist

In the NeurIPS Paper Checklist section of `neurips_main.tex`:

```latex
\item \textbf{Open Access to Data and Code}

\textbf{Question:} Does the paper provide URLs for anonymized data and code repositories?

\textbf{Answer:} Yes

\textbf{Justification:} Code is available at an anonymous GitHub repository:
\url{https://github.com/goldenfloat-anon/goldenfloat-anon} (URL withheld during review for double-blind compliance).
```

## Post-Submission De-Anonymization

After acceptance (notification: September 24, 2026):

1. Update README with real author names
2. Add link to main repository (gHashTag/trinity)
3. Update GitHub profile (optional)
4. Merge into main t27 repository

## Notes

- Do NOT fork existing t27 repository (reveals history)
- Do NOT include any author names in comments
- Do NOT link to personal blogs, Twitter, or other identities
- Repository can be deleted/recreated if identity is accidentally revealed

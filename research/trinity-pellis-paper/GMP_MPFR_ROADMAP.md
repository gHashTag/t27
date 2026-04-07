# GMP / MPFR / arbitrary precision — roadmap for t27

**Status:** planning. **Not SSOT** for release gates (`*.t27` + `tri` / `t27c` remain canonical).  
**Links:** [GNU GMP](https://gmplib.org), [MPFR](https://www.mpfr.org/), [Zig `std.math.big` discussion](https://github.com/ziglang/zig/issues/364).

## What GMP is

[GMP](https://gmplib.org) is a C library for **arbitrary-precision** integers (`mpz`), rationals (`mpq`), and floats (`mpf`). Precision is limited mainly by RAM. Typical users: cryptography, computer algebra (Sage, etc.), record \(\pi\) computations. [HKU HPC note](https://hpc.hku.hk/hpc/software/gmp/).

| | IEEE `f64` | GMP / MPFR |
|---|------------|------------|
| Precision | ~53 bits (~16 decimal digits) | User-chosen |
| \(\varphi\) | rounded | as many digits as requested |
| Speed | hardware | often \(10\times\)–\(1000\times\) slower |
| Error | \(\sim 10^{-16}\) rounding | exact for `mpq`; bounded for `mpf` / MPFR |

## Why t27 might adopt it

1. **Formal / reviewer narrative:** “\(\varphi^2+\varphi^{-2}=3\) EXACT” in ℝ is already algebra; **numeric** stress tests at **1000+ bits** still help separate implementation bugs from tolerance folklore.
2. **Pre-registered checkpoint:** Pellis \(\alpha^{-1}\) closed form matches CODATA to **sub-ppb**; publishing **50+ decimal digits** (unchanging, formula-fixed) tightens the “not f64 accident” story ahead of **CODATA 2026+**.
3. **Credibility:** answer to “rounding error?” — **mpmath** (today) or **GMP/MPFR** (later) reproducible scripts.

## Implementation options

### A. Zig + GMP (C interop)

Zig `@cImport` + `mpf_init2`, compute \(\varphi = (1+\sqrt{5})/2\), then `360/φ² - 2/φ³ + (3φ)⁻⁵`. `build.zig`: `exe.linkSystemLibrary("gmp")`.

### B. MPFR on top of GMP

Correct rounding for transcendentals (`sin`, `cos`, `log`, …). **Pellis** needs only algebraic ops; **hybrid** \(\arccos\) paths may want MPFR later.

### C. Python mpmath (done today)

`scripts/verify_precision.py` — optional, not on the verification critical path (`TZ-T27-001`).

### D. Stdlib-only seal (no deps)

`scripts/print_pellis_seal_decimal.py` — `Decimal.sqrt(5)`; good for a **one-line** refresh in docs.

## Suggested rollout

| Step | Deliverable | Location | Status |
|------|-------------|----------|--------|
| 1 | mpmath dump | `scripts/verify_precision.py` | **Done** |
| 2 | stdlib Decimal seal | `scripts/print_pellis_seal_decimal.py` | **Done** |
| 2b | Paste refreshed digits into `FORMULA_TABLE.md` when you lock precision | `FORMULA_TABLE.md` | Maintainer runs script, copies output |
| 3 | `build.zig` + system GMP | infra | Future |
| 4 | `tri math compare --precision N` (host-side high precision) | `bootstrap/` | Future PR |
| 5 | CI goldens at extended precision | workflows | After step 4 |

## Note on all 31 `FORMULA_TABLE` rows

Only rows **1, 2, 3, 5, 22–25, 27–31** are **pure \(\varphi\) / integer** closed forms. The rest need **PDG inputs**, **hybrid map** code, or **Koide masses** — extend scripts with explicit constants when you want a single report artifact.

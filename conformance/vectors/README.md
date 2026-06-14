# t27 numeric-format conformance vector packs (v0)

Bit-precise (and, where bit-precision is undefined, structural) conformance
vectors for the **complete t27 numeric-format catalog**, in a single shared row
schema so one differ runs across all packs.

- SSOT: https://github.com/gHashTag/t27/blob/master/specs/numeric/formats_catalog.t27
- Format spec: https://github.com/gHashTag/t27/blob/master/conformance/FORMAT-SPEC-001.json
- Anchor identity (ASCII): `phi^2 + 1/phi^2 = 3`
- Context preprint: https://arxiv.org/abs/2606.05017
- Schema tag: `t27-conformance/v0.1`

## Coverage at a glance

The catalog defines **83 numeric formats** across 13 families. This directory
ships **83 conformance packs — one per format**, with no gaps:

| Class | Packs | Meaning |
|---|---|---|
| **Bit-precise** | **55** | Native bits decode to f64 exactly; `abs_error = 0` by construction for every representable value. Values not exactly representable in a format report a nonzero `abs_error` **honestly** (e.g. 0.1 in bf16) — nothing is hidden. |
| **Structural** | **28** | The format has no single fixed radix-2 S:E:M round-trip (parametric / lookup / open-R&D / multi-double composite). These packs carry full catalog metadata plus an explicit `structural_reason` and are marked `bitexact: false`. They are honest placeholders, **not** bit-exact claims. |
| **Total** | **83** | One pack per catalog format. |

Coverage policy (deterministic, reproducible):

- formats ≤ 8 bits → **exhaustive** enumeration of every code;
- formats > 8 bits → a **curated** named vector set (zero, one, two, three,
  half, four, neg_one, neg_three, plus format specials) via explicit encoders —
  no brute force, no multi-megabyte files;
- non-S:E:M formats → **structural** pack with a documented reason.

Six formats were promoted from structural to **bit-precise** with dedicated
reference codecs (see the changelog at the end): the three IBM HFP base-16
floats (`ibm_hfp32/64/128`), Intel `x87_fp80` (explicit integer bit), the NF4
16-entry quantile table (`nf4`), and the 2-bit `gfternary` set. Each carries an
explicit decoder + reference encoder and `abs_error = 0` for every recorded
vector.

Of the 55 bit-precise packs, the IBM HFP and x87 packs hit the 3.0 anchor
**exactly** (3.0 = 0.1875 x 16^1 in HFP; 3.0 = 1.5 x 2^1 in x87). The packs
that do **not** place 3.0 on a grid point — `lns8/16/32/64`, `gf4`, `mxgf4`,
and now `gfternary` and `nf4` — are honest about it: log2(3) is not exactly
representable in a logarithmic number system, the 4-bit GoldenFloat grids are
too coarse, the `gfternary` levels are only {-phi, 0, +phi} (3.0 arises as
phi^2 + phi^-2 = 3, not as a single code), and the NF4 table spans [-1, 1].
Each such pack records the nearest representable value (or a null anchor with a
note) and its true `abs_error`.

## Index — bit-precise packs (55)

| Pack | Format | Vectors | Round-trip |
|---|---|---|---|
| `bf16_golden_conformance_v0.json` | BFLOAT16 | 8+golden | ✔ (предсуществующий) |
| `gfternary_conformance_v0.json` | GFTERNARY | 4 | ✔ (2-bit {-phi,0,+phi}, exhaustive) |
| `ibm_hfp128_conformance_v0.json` | IBM_HFP128 | 8 | ✔ (base-16, named small values) |
| `ibm_hfp32_conformance_v0.json` | IBM_HFP32 | 8 | ✔ (base-16 exponent) |
| `ibm_hfp64_conformance_v0.json` | IBM_HFP64 | 8 | ✔ (base-16 exponent) |
| `nf4_conformance_v0.json` | NF4 | 16 | ✔ (16-entry quantile table) |
| `x87_fp80_conformance_v0.json` | X87_FP80 | 8 | ✔ (explicit integer bit) |
| `binary128_conformance_v0.json` | BINARY128 | 8 | ✔ |
| `binary16_conformance_v0.json` | BINARY16 | 8 | ✔ |
| `binary256_conformance_v0.json` | BINARY256 | 8 | ✔ |
| `binary32_conformance_v0.json` | BINARY32 | 8 | ✔ |
| `binary64_conformance_v0.json` | BINARY64 | 8 | ✔ |
| `cray_float_conformance_v0.json` | CRAY_FLOAT | 8 | ✔ |
| `fp4_e2m1_conformance_v0.json` | FP4_E2M1 | 16 | ✔ |
| `fp6_e2m3_conformance_v0.json` | FP6_E2M3 | 64 | ✔ |
| `fp6_e3m2_conformance_v0.json` | FP6_E3M2 | 64 | ✔ |
| `fp8_e4m3fn_conformance_v0.json` | FP8_E4M3FN | 14 | ✔ (предсуществующий) |
| `fp8_e5m2_conformance_v0.json` | FP8_E5M2 | 16 | ✔ (предсуществующий) |
| `gf12_conformance_v0.json` | GF12 | 8 | ✔ |
| `gf16_conformance_v0.json` | GF16 | 21 | ✔ (предсуществующий) |
| `gf20_conformance_v0.json` | GF20 | 8 | ✔ |
| `gf24_conformance_v0.json` | GF24 | 8 | ✔ |
| `gf32_conformance_v0.json` | GF32 | 8 | ✔ |
| `gf4_conformance_v0.json` | GF4 | 16 | ✔ |
| `gf64_conformance_v0.json` | GF64 | 8 | ✔ |
| `gf6_conformance_v0.json` | GF6 | 64 | ✔ |
| `gf8_bfp_conformance_v0.json` | GF8_BFP | 256 | ✔ |
| `gf8_conformance_v0.json` | GF8 | 256 | ✔ |
| `gf_lns_hybrid_conformance_v0.json` | GF_LNS_HYBRID | 8 | ✔ |
| `int128_conformance_v0.json` | INT128 | 7 | ✔ |
| `int16_conformance_v0.json` | INT16 | 7 | ✔ |
| `int32_conformance_v0.json` | INT32 | 7 | ✔ |
| `int4_conformance_v0.json` | INT4 | 16 | ✔ |
| `int64_conformance_v0.json` | INT64 | 7 | ✔ |
| `int8_conformance_v0.json` | INT8 | 256 | ✔ |
| `lns16_conformance_v0.json` | LNS16 | 5 | ✔ |
| `lns32_conformance_v0.json` | LNS32 | 5 | ✔ |
| `lns64_conformance_v0.json` | LNS64 | 5 | ✔ |
| `lns8_conformance_v0.json` | LNS8 | 256 | ✔ |
| `ms_mbf32_conformance_v0.json` | MS_MBF32 | 8 | ✔ |
| `ms_mbf64_conformance_v0.json` | MS_MBF64 | 8 | ✔ |
| `mxfp4_e2m1_conformance_v0.json` | MXFP4_E2M1 | 16 | ✔ (предсуществующий) |
| `mxfp6_conformance_v0.json` | MXFP6 | 64 | ✔ |
| `mxfp8_conformance_v0.json` | MXFP8 | 256 | ✔ |
| `mxgf4_conformance_v0.json` | MXGF4 | 16 | ✔ |
| `mxgf6_conformance_v0.json` | MXGF6 | 64 | ✔ |
| `posit16_conformance_v0.json` | POSIT16 | 8 | ✔ |
| `posit32_conformance_v0.json` | POSIT32 | 8 | ✔ |
| `posit64_conformance_v0.json` | POSIT64 | 8 | ✔ |
| `posit8_conformance_v0.json` | POSIT8 | 256 | ✔ |
| `tf32_conformance_v0.json` | TF32 | 8 | ✔ |
| `vax_d_conformance_v0.json` | VAX_D | 8 | ✔ |
| `vax_f_conformance_v0.json` | VAX_F | 8 | ✔ |
| `vax_g_conformance_v0.json` | VAX_G | 8 | ✔ |
| `vax_h_conformance_v0.json` | VAX_H | 8 | ✔ |

`+golden` denotes an attached `golden_accumulation` section (bf16 reduction
reference for tt-mlir #6252).

## Index — structural packs (28)

| Pack | Format | Why structural (not bit-exact) |
|---|---|---|
| `afp_conformance_v0.json` | AFP | No fixed bit-precise round-trip is defined for this entry; recorded structurally with catalog metadata. |
| `bcd_conformance_v0.json` | BCD | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `block_fp_conformance_v0.json` | BLOCK_FP | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `decimal128_conformance_v0.json` | DECIMAL128 | IEEE 754 decimal (DPD/BID) encodes coefficients in a packed decimal field; round-trip is exact for decimal values but the bit layout is not a plain radix-2 S:E:M. |
| `decimal32_conformance_v0.json` | DECIMAL32 | IEEE 754 decimal (DPD/BID) encodes coefficients in a packed decimal field; round-trip is exact for decimal values but the bit layout is not a plain radix-2 S:E:M. |
| `decimal64_conformance_v0.json` | DECIMAL64 | IEEE 754 decimal (DPD/BID) encodes coefficients in a packed decimal field; round-trip is exact for decimal values but the bit layout is not a plain radix-2 S:E:M. |
| `double_double_conformance_v0.json` | DOUBLE_DOUBLE | Extended-precision layout (explicit integer bit / multi-double components) is not a single S:E:M field; recorded structurally. |
| `gf1024_conformance_v0.json` | GF1024 | No fixed bit-precise round-trip is defined for this entry; recorded structurally with catalog metadata. |
| `gf10_conformance_v0.json` | GF10 | No fixed bit-precise round-trip is defined for this entry; recorded structurally with catalog metadata. |
| `gf128_conformance_v0.json` | GF128 | Bias is an OPEN R&D parameter for this width (see catalog status Experimental); a bit-precise pack is deferred until the bias is fixed. |
| `gf14_conformance_v0.json` | GF14 | No fixed bit-precise round-trip is defined for this entry; recorded structurally with catalog metadata. |
| `gf256_conformance_v0.json` | GF256 | Bias is an OPEN R&D parameter for this width (see catalog status Experimental); a bit-precise pack is deferred until the bias is fixed. |
| `gf48_conformance_v0.json` | GF48 | No fixed bit-precise round-trip is defined for this entry; recorded structurally with catalog metadata. |
| `gf512_conformance_v0.json` | GF512 | No fixed bit-precise round-trip is defined for this entry; recorded structurally with catalog metadata. |
| `gf96_conformance_v0.json` | GF96 | No fixed bit-precise round-trip is defined for this entry; recorded structurally with catalog metadata. |
| `minifloat_conformance_v0.json` | MINIFLOAT | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `per_channel_scale_conformance_v0.json` | PER_CHANNEL_SCALE | INT8 payload with an external per-channel fp32 scale; the decoded value depends on the scale tensor, so a standalone round-trip table is not defined. |
| `q_format_conformance_v0.json` | Q_FORMAT | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `quad_double_conformance_v0.json` | QUAD_DOUBLE | Extended-precision layout (explicit integer bit / multi-double components) is not a single S:E:M field; recorded structurally. |
| `shared_exp_conformance_v0.json` | SHARED_EXP | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `stochastic_rounding_conformance_v0.json` | STOCHASTIC_ROUNDING | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `takum16_conformance_v0.json` | TAKUM16 | Takum (Hunhold 2024) is a tapered LOGARITHMIC format; its decode is not a plain S:E:M field. |
| `takum32_conformance_v0.json` | TAKUM32 | Takum (Hunhold 2024) is a tapered LOGARITHMIC format; its decode is not a plain S:E:M field. |
| `takum64_conformance_v0.json` | TAKUM64 | Takum (Hunhold 2024) is a tapered LOGARITHMIC format; its decode is not a plain S:E:M field. |
| `takum8_conformance_v0.json` | TAKUM8 | Takum (Hunhold 2024) is a tapered LOGARITHMIC format; its decode is not a plain S:E:M field. |
| `tapered_fp_conformance_v0.json` | TAPERED_FP | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `unum_i_conformance_v0.json` | UNUM_I | This format has no single fixed bit layout (parametric / technique / variable-width). |
| `unum_ii_conformance_v0.json` | UNUM_II | This format has no single fixed bit layout (parametric / technique / variable-width). |

The four `takum*` packs double as the **live FL-002 counterexample**: a tapered
logarithmic format whose decode is not a plain S:E:M field, recorded as an open
falsification entry rather than forced into a round-trip it does not satisfy.

## Shared row schema

Each bit-precise vector row carries:

```
name              short label
input_f64         the f64 value (or "NaN" / "Inf" / "-Inf" for specials)
input_f64_hex     big-endian IEEE-754 double hex of input
<fmt>_bits_hex    the native format bit pattern, hex
<fmt>_bits_int    the native format bit pattern, integer
decoded_f64       value after decode back to f64
decoded_f64_hex   big-endian double hex of decoded
abs_error         |input_f64 - decoded_f64| (0 for representable values)
category          one of: zero, subnormal, normal, inf, nan, phi_anchor
```

The bits field is named per format (`gf16_bits_*`, `fp8_bits_*`, …). All other
keys are identical across packs, so one differ tool runs across the whole set.

Structural packs share the top-level metadata block (`schema`, `format`,
`format_name`, `catalog`, `ssot`, `preprint`, `anchor_identity`) but carry
`bitexact: false` and a `structural_reason` instead of a `vectors` array of
round-trips.

## Machine-readable index

`INDEX_all_formats.json` lists all 83 packs with totals
(`total_formats: 83`, `total_packs: 83`, `bitexact_packs: 55`,
`structural_packs: 28`), the anchor identity, the SSOT path, and the preprint.

## SHA-256

```
6b5b83ff6949acc2f0ecf6cf2488109388bc05dc298791cb3187b5d8ea860d01  afp_conformance_v0.json
42de461e6878d7a8b891f72b07c9158ac4cc1c92c5dbfbdbbe0c46c60429dd29  bcd_conformance_v0.json
98bbddcbb8a520dc45a6dfed7209c50a0acc0fabc4d3b430359969467eee4e13  bf16_golden_conformance_v0.json
9d08a8a6a10f94e875f73e5f44643f088435a570bfafaef9991464d5377b26e0  binary128_conformance_v0.json
84fd7629430b06d761ac3b92fc85208c472a4582040b1ac2001cc87a6612f7b4  binary16_conformance_v0.json
43d39ae4c4808276ba1ed5b2d8c221c17983a53570da2fc161996b0f7ea3aee3  binary256_conformance_v0.json
9dc16a1c3b65b7f7a5d59c546886ed12e99fb37cbfc9f3d5d45813921e6a70ff  binary32_conformance_v0.json
1d3e3d6daee576ae3b2b4dca6f26560390535fb7441a54b389f98a4238e58bec  binary64_conformance_v0.json
3129fa92145096e55527c2fc22d9e6bed23db1a6d88148e8b711a3b6641a43c1  block_fp_conformance_v0.json
b1a8f6652112be3f49949bafe9f6cd7f46f0271e8f4e19cadb55c2a0e972f503  cray_float_conformance_v0.json
0e3ab9f3bd6bc3525457c9cbbab1d99ecc801a5019dd92cc87f2c5468b0d471a  decimal128_conformance_v0.json
cd43863397f911142eed1e8e5f2a8ef41ba97d344aa6b64f8f550b5b390bbcad  decimal32_conformance_v0.json
c42aabb5cf847a521698d9451f490ceb17c29ff1169e9c1147609ae737635a5b  decimal64_conformance_v0.json
6bc6d15ba3258a125c591dedc63b4125b7ecc0d502bc1c3a2ecf65a76714c526  double_double_conformance_v0.json
8ded6625c4644139320dd89b2b7815d6ba27177c35b7d645b2d93b8cfdc63fd9  fp4_e2m1_conformance_v0.json
de70d6aacf0ac2d47decae0866d14f126058176428315d4c767e460c0a9ae5e5  fp6_e2m3_conformance_v0.json
17a80f0a3b5b2495dbcd6de6062d8c1f8ce19b9746d1e370e6d16897ef5f9c02  fp6_e3m2_conformance_v0.json
7193ccd0d330d3e05154432abcec5da4a4c170e11004d4ffa44ff5cbbff9cba9  fp8_e4m3fn_conformance_v0.json
9c31fbd03923bd6555304848a092504dfbc02f72d2be82d2b80f49243e925a18  fp8_e5m2_conformance_v0.json
652fe39cee0023880091c79bb4b6def96950947da920fdf6f059a33ea1b7c405  gf1024_conformance_v0.json
e7043494280dd9716f7e47ed2ef1b149872823514a9267cca5190599fc6b2102  gf10_conformance_v0.json
322bfc28e4182878b476ae4e869b5a697ea2224895550ada1dd953fad0b110cc  gf128_conformance_v0.json
ea00efde4825931a421ec9feb5910f3ad9ab7ab5d38a77d2c364ea9fa49a7f96  gf12_conformance_v0.json
0ed0eb17a72fb959746fd86892a257a53a7191edb7889e26d59fd630922f54a9  gf14_conformance_v0.json
7aea5b9e86ea71a54ae0c1601cea13e2d90d95fecaf2ae969eac1349cf7a2b42  gf16_conformance_v0.json
76c7814558901d5633cb16ffead7468583de5577c4ccf0378c296c73ae08acc5  gf20_conformance_v0.json
983642c7aea54b7e6c5b6e41edcf20828bfc3a1f2707307eaa713ca5a45e612c  gf24_conformance_v0.json
49875dcac61b316151064ce2f462fe732f37ce07c18e9024240cba4d707e2f9d  gf256_conformance_v0.json
f7222e2442f2c106e7f3590e5dbe8ed177603fc2324560987af138ae9abeceb4  gf32_conformance_v0.json
85550904fc58172e11862a11be4cdad7a769064b39f7f2b457224208e4018d16  gf48_conformance_v0.json
25471b7a0e3dc3633118191e722ced2f450a3ed8a6228ad2492f92084f556f96  gf4_conformance_v0.json
f03fa45ee9e640c8d1fba1b176c7951de1409696bf0f4395380ba6f029944087  gf512_conformance_v0.json
887223d0bc8b00d76b70238ddbc8933e3a773ed6a9fbc10264d9fdbebca76cd3  gf64_conformance_v0.json
9c9fc955db5f6c9b185bdd5d88bd92f3f21a71ad4d784b944330d5cba85fb724  gf6_conformance_v0.json
fe600234cab0e589b69d84e673d74729cff153f9e4e63e871e285fa82ad2cc70  gf8_bfp_conformance_v0.json
6dccbc6628cbc051e06a006a0731499970c1d99e65fc0d42d9007d8f0ed1402d  gf8_conformance_v0.json
786f9d144243db2e6c4dba2ddbef4ae2975d045f1ad370cd462eb61cc70dd3d5  gf96_conformance_v0.json
eb7c946281fb6ed6fadd9c63c7e7fa186412480910c9fedcb25fbc056c1bd34a  gf_lns_hybrid_conformance_v0.json
9f246d24511fbff6fb9e83e60e1bedfce401052537f7c8929fe205d0f6e57b81  gfternary_conformance_v0.json
2f02899d621a8a7aebfdf2a69a2484d7616c61ac7cebdc483f643e8109c4e31f  ibm_hfp128_conformance_v0.json
8e35040e30d3a0091ecca5fdb08d1dd1ce98031e5d655239c7196bc667fc3876  ibm_hfp32_conformance_v0.json
fbe42a167c13f226fe8eaf876c9e17109fb32dbddd76d3a677cc0b9aef2626a1  ibm_hfp64_conformance_v0.json
df77519366e4c59888dcfafa66c20db4389e162f7dae96767684f46d8427d9fc  int128_conformance_v0.json
a14f51cd6b29bef2215573bc7f1d299559d5d34af36fd0ffceb513f0659765b9  int16_conformance_v0.json
e7f8ddbc4f8606a83febb5c8836f38a143c28f650d73330af5650ea698d91570  int32_conformance_v0.json
ec35d81224f9635c21b53165e123d1b6b0ed13ad3c1d7ec830e018df22c46ad1  int4_conformance_v0.json
15e7ae6bd373de5b4f755c71877d6bb662c241526f3a5f4ead22307764f754f5  int64_conformance_v0.json
3e7358f28f5d242e24a46fcf0359e24e21bb4f54834e88b38235fc6332e86978  int8_conformance_v0.json
0b4b9d4ce2162e079239aa05e742c7f699453b96f6a8abd90c17bca95d88eff6  lns16_conformance_v0.json
7877795b137b599bafb64e8c4b114e128a2ef5af433db222b2875e0df6919872  lns32_conformance_v0.json
9969a97f62be45bc120cc2310b5be5c0e6c83d3151a51d6fbb5173db54581ec0  lns64_conformance_v0.json
1cb3ca966b7564b15d6b64b37efb4548fa4b6ff26686576cf5da9ac7657379b8  lns8_conformance_v0.json
d42d1d167c5aa9f504de5ca9ebc04cbd863ff5ebb300fbcb09306413d7b334c3  minifloat_conformance_v0.json
260382da8c40576d046d6d044753522e1c3730e55a4712f21e91b21ad918c365  ms_mbf32_conformance_v0.json
8d750f03d47a4113548a5e57be2201d4b7f5689143b3605a9a51a60b49988ec9  ms_mbf64_conformance_v0.json
b5795fed0c0f2b580174b443d2c54519c4953916525237bb7ea7d6831f14fde7  mxfp4_e2m1_conformance_v0.json
3db420779597b673a691d409b1fc11aee8168549c55be1c8ebfb70dc8330e0c7  mxfp6_conformance_v0.json
16eedca7e82c4e6753f8248dce0000ba9a50ba09bab9be747b0dcac3efc21b6b  mxfp8_conformance_v0.json
5e8d03fe80c59b458dc4bbd3fba3213dbc00626d8bc73b2cfcc09836539e89fb  mxgf4_conformance_v0.json
9d77d8be5522942e9276b723915b3223123b7741a076a1bfd819cc73ab29f1ec  mxgf6_conformance_v0.json
723ddd4237153c7c0cc6a9c3436ba071f8affcef8ad0c384070a3e1a3bf13f45  nf4_conformance_v0.json
300f176150f74952183199befd9c9972473a1dfbd0206f2726e93bcf7b2d4957  per_channel_scale_conformance_v0.json
7cc2edfeb0f52769b1a536dcbe04945a301cdcc3799a23267380b0c4fb0b82d5  posit16_conformance_v0.json
aee6cc72691a0ae211e39bf6315ac68a5fe74e87190e0c27088871c6ccc87f52  posit32_conformance_v0.json
66b14056938549c1aaa522097ee8744246581cf6d3bb6002cef3b2f3f6ea0ff6  posit64_conformance_v0.json
0c638ef95b6537e4dc0e256dc1ca2d9363152b3d5a800501472230ce98a84b76  posit8_conformance_v0.json
2bc0c114aecd1d0dbfa7925efe298cea80efd510022ee0734cf9450af8027b63  q_format_conformance_v0.json
d0d7b15d7d8d6c590437a1e9f2d215d63115448f4eb26fe479cc5999f7b21c03  quad_double_conformance_v0.json
ca139ebd7bc5c139357c533bb0a6509e4edd05044d7869fcea57cf34d052c3c7  shared_exp_conformance_v0.json
fc2a0a6dcce7bbb0eccc1e23ebdacb9abbd81cd54111796be6dc7e6a87a2071a  stochastic_rounding_conformance_v0.json
631c574d46f1b2288ffddeab0b157c12a4f904b5bf7371128267f96db0b19c5c  takum16_conformance_v0.json
40672b9193cf6af56079f60fedd17a44c662293b52de3f4fb32fe92385c28445  takum32_conformance_v0.json
3966b0c46655e572dc83867dab2d1b09a80d3e01b45f1de88fc10f1ff095654c  takum64_conformance_v0.json
bb244148c18796e924a2c44a6fff8cbf59300d72951549808945b6accc9abde5  takum8_conformance_v0.json
e20d828fda7d3d6b86d047b390fae2964622b83fb8200220f5a4283bc5ae2b4b  tapered_fp_conformance_v0.json
b35c334092b49dad9a944d37a91697f08f442b633142e81c69296c12bac0055d  tf32_conformance_v0.json
2b28ce1eca1f39623122fbdd853945d05c73adc47e132c250e6ccb56217d689b  unum_i_conformance_v0.json
a79c4cdca84fc702a4ce25fd36040912f841dc9d75f35476e78dbbdfc6fb12f8  unum_ii_conformance_v0.json
b87494ddee38fe68f77dd8082e8a9811530cc346526b80e59c81b17665677792  vax_d_conformance_v0.json
a7f45aec8da42931da5ad9f24c3ee369419ec58a783abe657a275210ae9b1e4d  vax_f_conformance_v0.json
9a6372bbf85a50457e0b66db8849845333582e3fef28934047963e89dd95e65a  vax_g_conformance_v0.json
eaaa44e4ce2e5454da2cc83571bf3261a84a1b6fabb0dc8064c04cd71fa581f0  vax_h_conformance_v0.json
e9be37c939c7108081bd2190e949f4d01be7ad12511d82bb8849f337c94e7e0c  x87_fp80_conformance_v0.json
```

## Provenance

The five hand-curated reference packs (byte-stable) come from:

```
gen_fp8_e4m3.py    -> fp8_e4m3fn_conformance_v0.json
gen_fp8_e5m2.py    -> fp8_e5m2_conformance_v0.json
gen_mxfp4_e2m1.py  -> mxfp4_e2m1_conformance_v0.json
gen_bf16_golden.py -> bf16_golden_conformance_v0.json
(gf16_conformance_v0.json is the original SSOT anchor pack)
```

The remaining 78 packs (and `INDEX_all_formats.json`) are produced by the
catalog-wide master generator:

```
gen_all_formats.py   # reads the live SSOT, emits one pack per catalog format
```

Re-run from this directory to reproduce byte-identical packs:

```
python3 gen_all_formats.py
```

All packs are ASCII-only. Apache-2.0, consistent with the t27 repository.

## Changelog

- **2026-06-14** — promoted 6 packs from structural to **bit-precise** by adding
  dedicated reference codecs to `gen_all_formats.py`
  (bit-precise 49 -> 55, structural 34 -> 28):
  - `ibm_hfp32/64/128` — IBM Hexadecimal Floating Point, base-16 exponent
    (S1 : E7 excess-64 : M), `value = 0.M(2) * 16^(E-64)`. 3.0 = 0.1875 x 16^1.
  - `x87_fp80` — Intel 80-bit extended with the explicit integer bit as the MSB
    of the 64-bit significand field (S1 : E15 : SIG64), bias 16383.
  - `nf4` — QLoRA/bitsandbytes NF4 16-entry quantile table over [-1, 1];
    exhaustive over all 16 codes.
  - `gfternary` — 2-bit {-phi, 0, +phi}; exhaustive over all 4 codes (3.0 arises
    as phi^2 + phi^-2 = 3, not as a single code, recorded in the anchor note).
  Remaining `ExtendedFloat` entries `double_double` / `quad_double` stay
  structural (composite multi-double, no single S:E:M field).

# FPGA Loop Evidence — Wave Loop 412

> Date: 2026-07-04  
> Target board: QMTech Wukong V1 / XC7A100T-FGG676 (IDCODE `0x13631093`)  
> Cable: Digilent DLC10 / Xilinx Platform Cable USB II (`VID=0x03FD`)  
> Issue: [#1332](https://github.com/gHashTag/t27/issues/1332)

---

## 1. Cable detection

```text
$ dlc10 idcode
DLC10 cable not found (VID=0x03FD)
```

Hardware not available; W412 was done under **Variant C** (formal tooling
without physical measurement).

---

## 2. Branching/strategy commands

```text
$ git checkout master
$ git pull
$ gh pr list --state open
#1331  W411 close-out — strategy/architecture ...  wave-loop-411  about 5 hours ago

$ gh pr merge 1331 --squash --admin
✓ Squash-merged pull request #1331

$ gh issue close 1329
✓ Closed issue #1329

$ gh api repos/{owner}/{repo}/compare/master...trinity-rust-rings --jq '.ahead_by, .behind_by'
1
5
```

`trinity-rust-rings` remains 5 commits behind and 1 ahead; it is **archived**,
not deleted.

---

## 3. tri CLI verification

### 3.1 Standalone measured-to-lean

```text
$ tri fpga measured-to-lean --in docs/data/cclk_example.json --out MeasuredColdPOR.lean --standalone
Wrote standalone proof stub to MeasuredColdPOR.lean
```

Generated file header:

```lean
import Trinity.BitstreamConfig
import Trinity.TernaryFPGABoot

namespace Trinity.BitstreamConfig
```

### 3.2 Raw-ns measured-to-lean

```text
$ tri fpga measured-to-lean --raw-ns --in docs/data/cclk_raw_ns_example.json --out RawNsStub.lean
Wrote raw-ns proof stub to RawNsStub.lean
```

### 3.3 Main verification

```text
$ ./scripts/tri test
parse/typecheck/gen/seal-verify ... ok
yosys smoke ... 40 pass, 16 fail (pre-existing)
```

---

## 4. Lean 4 verification

```text
$ lake build Trinity.TernaryFPGABoot
[0/1] Building Trinity.TernaryFPGABoot
...
Build completed successfully.
```

New theorems checked:

- `measured_cclk_from_raw_ns_implies_transaction_ok`
- `measured_raw_ns_40_20_20_satisfies_flash_spec`
- `measured_cclk_with_pvt_implies_transaction_ok`
- `measured_25mhz_50duty_pvt_satisfies_flash_spec`

---

## 5. Rust unit-test verification

```text
$ cargo test -p tri fpga::tests
running 16 tests
test fpga::tests::test_measured_to_lean_output ... ok
test fpga::tests::test_measured_to_lean_output_standalone ... ok
test fpga::tests::test_measured_to_lean_output_raw_ns ... ok
...
test result: ok. 16 passed; 0 failed; 0 ignored
```

---

## 6. Conclusion

No physical evidence was captured in W412 because the DLC10 cable and logic
analyzer channel were unavailable. Instead, the wave delivered formal,
tool-level infrastructure that makes physical evidence useful once it is
captured, and updated the branch model to reflect the new `master`-as-release
reality.

---

*phi^2 + phi^-2 = 3 | TRINITY*

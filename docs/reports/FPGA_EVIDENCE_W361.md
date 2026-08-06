:# FPGA Evidence — Wave Loop 361

**Date:** 2026-07-02  
**Target board:** QMTech Wukong V1 / XC7A100T-FGG676 (`xc7a100tfgg676-1`)  
**Design:** `ternary_mac_demo_top` → wraps `ternary_mac_top`  
**Toolchain:** OpenXC7 = yosys 0.63 + nextpnr-xilinx (stable-backports) + f4pga/prjxray

---

## Status

| Step | Result |
|------|--------|
| Hand-written ternary MAC RTL | ✅ `fpga/verilog/ternary_mac_synth.v` |
| Self-checking testbench | ✅ 6/6 vectors pass |
| yosys synthesis | ✅ JSON netlist generated |
| `nextpnr-xilinx` place & route | ✅ 4 warnings, 0 errors, Fmax 643.92 MHz |
| FASM → frames | ✅ 9.6 MB frames generated |
| frames → `.bit` | ✅ **3.6 MB bitstream generated** |
| Board `idcode` / flash | ❌ DLC10 cable not connected to this session |

---

## Bitstream generation log

### 1. Synthesis

```sh
yosys -p 'read_verilog ternary_mac_synth.v ternary_mac_demo_top.v; \
          synth_xilinx -family xc7 -top ternary_mac_demo_top -flatten; \
          write_json ternary_mac_demo_top.json'
```

### 2. Place & route

```sh
nextpnr-xilinx --chipdb xc7a100tfgg676.bin \
    --xdc ternary_mac_demo_top.xdc \
    --json ternary_mac_demo_top.json \
    --fasm ternary_mac_demo_top.fasm \
    --ignore-loops
```

Output:

```
Info: Max frequency for clock 'chain[19]': 643.92 MHz (PASS at 12.00 MHz)
Info: 4 warnings, 0 errors
```

### 3. FASM → frames

```sh
PYTHONPATH=/path/to/prjxray \
    python /path/to/prjxray/utils/fasm2frames.py \
    --db-root prjxray-db/artix7 \
    --part xc7a100tfgg676-1 \
    ternary_mac_demo_top.fasm \
    ternary_mac_demo_top.frames
```

### 4. Frames → bitstream

```sh
xc7frames2bit \
    --frm_file ternary_mac_demo_top.frames \
    --output_file ternary_mac_demo_top.bit \
    --part_file prjxray-db/artix7/xc7a100tfgg676-1/part.yaml \
    --part_name xc7a100tfgg676-1
```

File identification:

```
$ file ternary_mac_demo_top.bit
Xilinx BIT data - from ternary_mac_demo_top.frames;
Generator=xc7frames2bit - for xc7a100tfgg676-1
```

---

## OpenXC7 toolchain build (macOS arm64)

### nextpnr-xilinx

```sh
git clone --branch stable-backports https://github.com/openXC7/nextpnr-xilinx.git
cd nextpnr-xilinx
git submodule update --init --recursive
cmake -B build -DARCH=xilinx -DUSE_OPENMP=OFF \
    -DCMAKE_CXX_FLAGS="-I$(brew --prefix eigen)/include/eigen3"
cmake --build build --target nextpnr-xilinx bbasm -j4
```

Notes:
- `boost-python3` must be installed (`brew install boost-python3`); the directory from `brew --prefix` alone is insufficient.
- `-DUSE_OPENMP=OFF` is required for Apple clang.
- Eigen 5.0 needs explicit include path because it no longer sets `EIGEN3_INCLUDE_DIRS`.

### chipdb

```sh
cd nextpnr-xilinx
PYTHONPATH=xilinx/python python3 xilinx/python/bbaexport.py \
    --device xc7a100tfgg676-1 \
    --xray xilinx/external/prjxray-db/artix7 \
    --bba build/xc7a100tfgg676.bba
build/bbasm --le build/xc7a100tfgg676.bba build/xc7a100tfgg676.bin
```

Result: `xc7a100tfgg676.bin` = 152 MB.

### prjxray

```sh
git clone https://github.com/f4pga/prjxray.git
git clone https://github.com/f4pga/prjxray-db.git
cd prjxray
git submodule update --init --recursive
cmake -B build -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DPRJXRAY_BUILD_TESTING=OFF
cmake --build build --target xc7frames2bit -j4
```

### Python venv for fasm2frames

```sh
python3 -m venv prjxray-venv
prjxray-venv/bin/pip install fasm pyyaml simplejson intervaltree numpy
```

Run `fasm2frames.py` with `PYTHONPATH=<prjxray repo>`.

---

## Next steps

1. Connect QMTech Wukong V1 + Xilinx Platform Cable USB II to the host.
2. `cargo build --release -p dlc10`
3. `/Users/playra/t27/target/release/dlc10 idcode` → expect `0x13631093`
4. `/Users/playra/t27/target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`
5. Verify LEDs toggle and `DONE=HIGH`.

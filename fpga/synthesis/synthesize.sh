# Trinity/t27 FPGA Synthesis Setup

**XC7A100T Target**
- Device: XC7A100T-FGG484
- Package: LQFP100
- Max LUTs: 63,400
- Max DSPs: 0 (Ternary MAC uses LUTs only)
- Max Clock: Target ~92 MHz

## Synthesis Flow

### 1. Yosys Synthesis
```bash
yosys -p "read_verilog gen/verilog/fpga/*.v; synth_xilinx -top Trinity_FPGA_Top -flatten; write_json build/synth.json"

# 2. Nextpnr Place & Route
```bash
nextpnr-xilinx --chipdb prjxray-db/artix7/db/artix7-100t.bin \
  --xdc specs/fpga/constraints/qmtech_a100t.xdc \
  --netlist build/synth.json \
  --write build/routed.fasm \
  --fasm build/fasm2frames

# 3. Bitstream Generation
```bash
fasm2frames --db-root prjxray-db/artix7 \
  --part_file prjxray-db/artix7/100t.yaml \
  --frm_file build/trinity.bit
```

### 4. Build Scripts

Create `fpga/synthesis/synthesize.sh` - Main synthesis script
Create `fpga/synthesis/README.md` - Documentation and requirements

### Verification Targets

- [ ] MAC module: LUT usage (should be 0 DSPs)
- [ ] UART module: Max clock frequency
- [ ] Top-level: Successfully syntheses to bitstream
- [ ] Timing: All paths meet timing constraints

### Prerequisites

- Yosys: `apt install yosys` (Debian/Ubuntu)
- nextpnr-xilinx: `pip install nextpnr-xilinx`
- prjxray database: `prjxray-db` project
- Constraint file: `specs/fpga/constraints/qmtech_a100t.xdc`

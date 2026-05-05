# Appendix I: Bitstream Generation and Toolchain

## I.1 Toolchain Overview

The FPGA configuration bitstream for the Trinity VSA accelerator is generated using the Xilinx Vivado Design Suite. The toolchain follows the standard FPGA compilation flow: synthesis, implementation, and bitstream generation.

```
Verilog Source (.v) ──> Vivado Synthesis ──> Netlist (.edf)
                                                    │
XDC Constraints (.xdc) ──────────────────────────> │
                                                    ▼
                                            Vivado Implementation
                                            (Place + Route)
                                                    │
                                                    ▼
                                            design.bit (3.8 MB)
                                                    │
                                                    ▼
                                         SHA-256 Checksum Verification
                                                    │
                                                    ▼
                                    openFPGALoader ──> XVC ──> FPGA
```

## I.2 Synthesis Flow

### I.2.1 Source Files

The synthesis input consists of:

| File                      | Purpose                                    |
|---------------------------|--------------------------------------------|
| `fpga/vsa/vsa_bind.v`     | Ternary bind module (parameterized DIM)    |
| `fpga/vsa/vsa_unbind.v`   | Unbind wrapper (alias to bind)             |
| `fpga/vsa/vsa_bundle.v`   | Majority-vote bundle module                |
| `fpga/vsa/vsa_top.v`      | Top-level integration with opcode dispatch |
| `xdc/qmtech_xc7a.xdc`    | Pin constraints for QMTech board           |

The top module `vsa_top` is the synthesis entry point. The `DIM` parameter is set via Vivado's `set_property generic` mechanism:

```tcl
set_property generic {DIM=10000} [current_fileset]
```

### I.2.2 Constraint File

The XDC constraint file (`xdc/qmtech_xc7a.xdc`) defines:

1. **Clock constraint**: 50 MHz input clock on the designated clock pin
2. **I/O standards**: LVCMOS33 for all user I/O
3. **Pin assignments**: LED pins (R3, R23), JTAG pins, and any user I/O

The corrected XDC file (commit `a63d3fb8`) fixed 23 invalid pin assignments from the CSG324 package to the correct FGG676 package pins. The original constraints targeted a different package variant, causing Vivado to report "invalid site" errors during implementation.

### I.2.3 Vivado Commands

The complete synthesis flow can be executed from the Vivado TCL console:

```tcl
# Create project
create_project vsa_accel ./build -part xc7a100tfgg676-1

# Add source files
add_files {fpga/vsa/vsa_bind.v fpga/vsa/vsa_unbind.v \
           fpga/vsa/vsa_bundle.v fpga/vsa/vsa_top.v}
add_files -fileset constrs_1 xdc/qmtech_xc7a.xdc

# Set top module with parameter
set_property top vsa_top [current_fileset]
set_property generic {DIM=10000} [current_fileset]

# Run synthesis
launch_runs synth_1 -jobs 4
wait_on_run synth_1

# Run implementation
launch_runs impl_1 -jobs 4
wait_on_run impl_1

# Generate bitstream
launch_runs impl_1 -to_step write_bitstream
wait_on_run impl_1

# Export
write_bitstream ./build/design.bit
```

## I.3 Bitstream Structure

The output bitstream `design.bit` is a 3.8 MB file in Xilinx BIT format. The file structure consists of:

### I.3.1 Header

| Offset | Field             | Value                     |
|--------|-------------------|---------------------------|
| 0x00   | Header length     | Variable (header size)    |
| 0x0B   | Sync word         | `AA 99 55 66`             |
| After sync | Design name  | `vsa_top;UserID=0XFFFFFFFF;Version=2024.1` |
|        | Part number       | `xc7a100tfgg676`         |
|        | Date              | `2026/05/05`             |
|        | Time              | `15:00:00`               |
|        | Data length       | `0x003B_F0C0` (~3.8 MB)  |

### I.3.2 Configuration Data

After the header, the bitstream contains configuration frames organized by block type:

- **Type 0 (CLB/IO)**: Logic and I/O configuration
- **Type 1 (BRAM)**: Block RAM initialization and configuration
- **Type 2 (CFG/CLK)**: Clock and global signal configuration

Each frame consists of a frame address (row, column, minor address) followed by 101 words of configuration data (for 7-series devices).

### I.3.3 Configuration Packets

The configuration data is encoded as a sequence of Type 1 and Type 2 packets:

```
Type 1: [Opcode(2)][Type(1)][Register Address(5)][Word Count(24)]
Type 2: [Opcode(2)][Type(1)][Word Count(29)]
```

Key registers written during configuration:

| Register      | Address  | Purpose                         |
|---------------|----------|---------------------------------|
| CRC           | 0x00     | CRC-32 integrity check          |
| IDCODE        | 0x0C     | Target device verification      |
| MASK          | 0x16     | Write mask for CTL0             |
| CTL0          | 0x12     | Configuration control           |
| FAR           | 0x17     | Frame address register          |
| FDRI          | 0x14     | Frame data register (input)     |
| CMD           | 0x04     | Command register                |
| STAT          | 0x27     | Status register (readback)      |

The configuration sequence ends with a `JSTART` command (CMD register = 0x0D) followed by a `DESYNC` command (CMD register = 0x0E), which transitions the FPGA into the operational state.

## I.4 Integrity Verification

### I.4.1 SHA-256 Checksum

The bitstream integrity is verified using SHA-256:

```
File: bitstream/design.bit
SHA-256: 8536e265b6b2119c528cc521133b484c2d27a1c0a97346095920ca9a2d77352b
```

This checksum is stored in `bitstream/design.bit.sha256` and can be verified with:

```bash
sha256sum -c design.bit.sha256
# design.bit: OK
```

### I.4.2 Built-in CRC

The FPGA's configuration engine also verifies a CRC-32 checksum embedded in the bitstream. After loading all configuration frames, the hardware computes a running CRC and compares it with the expected value in the CRC register. A mismatch sets the CRC_ERROR bit in the STAT register. Our deployment confirmed STAT = `0x401079FC` with CRC_ERROR = 0, proving bitstream integrity through both software (SHA-256) and hardware (CRC-32) mechanisms.

## I.5 Deployment

### I.5.1 Command

The bitstream is deployed to the FPGA using `openFPGALoader` with the XVC client cable:

```bash
openFPGALoader --cable xvc-client \
               --ip 192.168.1.30 \
               --port 2542 \
               -b arty_a7_100t \
               design.bit
```

The `-b arty_a7_100t` flag specifies the target board family, which determines the IDCODE expected by the programmer. Despite targeting the "arty_a7_100t" profile, the actual device (XC7A200T with IDCODE `0x3631093`) is accepted due to backward compatibility.

### I.5.2 Deploy Flow

The deployment proceeds through these phases:

1. **JTAG reset**: 5 cycles of TMS=1 to enter Test-Logic-Reset state
2. **IDCODE check**: Shift IR = 0x01 (IDCODE instruction), read 32-bit DR
3. **IR loading**: Shift IR = 0x0B (JPROGRAM) to initiate programming
4. **Configuration**: Stream bitstream data through DR using shift operations
5. **Startup**: JSTART command, wait for DONE pin assertion
6. **Verification**: Read STAT register to confirm clean configuration

The entire process takes approximately 60 seconds over WiFi-XVC, compared to ~5 seconds over a direct USB-JTAG connection.

## I.6 Resource Summary

The deployed bitstream (initial blink test, not VSA modules) uses:

```
Resource utilization:
  LUT:      83 / 101440  (0.08%)
  FF:       27 / 126800  (0.02%)
  IO:       25 / 210     (11.90%)
  BRAM:     0 / 135      (0.00%)
  DSP:      0 / 240      (0.00%)
  Max freq: 309.28 MHz
```

The minimal resource utilization confirms that the FPGA has ample capacity for the VSA modules (estimated ~25% LUT utilization for DIM=10000 bind+bundle) and future expansion.

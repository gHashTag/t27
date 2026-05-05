# Appendix F: FPGA Hardware Platform — QMTech XC7A100T

## F.1 Overview

The hardware platform for Trinity VSA acceleration is the QMTech XC7A100T-FGG676 development board, based on the Xilinx Artix-7 family of FPGAs. The board provides a cost-effective entry point for 7-series FPGA development with sufficient logic resources to implement the complete VSA bind/bundle/unbind pipeline at dimension D = 10,000.

## F.2 Device Specifications

The Artix-7 XC7A100T provides the following resources:

| Resource              | Quantity   | Notes                              |
|-----------------------|------------|-------------------------------------|
| Look-Up Tables (LUT)  | 101,440    | 6-input, dual-output               |
| Flip-Flops (FF)       | 126,800    | D-type, clock enable               |
| Block RAM (BRAM, 36Kb)| 135        | True dual-port, 36 Kb each         |
| DSP48E1 slices        | 240        | 25x18 multiplier + 48-bit accumulator |
| I/O pins (user)       | 210        | 3.3V LVCMOS                        |
| Clock management      | 6 CMTs     | Each: 1 MMCM + 1 PLL               |
| Package               | FGG676     | 676-pin fine-pitch BGA             |
| Process node          | 28 nm      | TSMC HPL                           |

The total on-chip memory is 135 x 36 Kb = 4,860 Kb = 607.5 KB, sufficient for storing multiple hypervector codebooks. Each BRAM can be configured as two independent 18 Kb memories, effectively doubling the storage granularity.

## F.3 Board-Level Details

The QMTech board provides:

- **Clock**: 50 MHz crystal oscillator (primary), accessible via BUFG
- **LEDs**: 2 user LEDs (D5: active-low, D6: active-low) on pins R3 and R23
- **JTAG header**: 6-pin 2.54mm header (VCC, GND, TCK, TDO, TDI, TMS)
- **Power**: USB-C 5V input, onboard 3.3V and 1.0V regulators
- **Configuration**: JTAG (primary), SPI flash (optional)

The board's JTAG header pinout was verified from the silkscreen:

| Header Pin | Signal | ESP32 GPIO |
|------------|--------|------------|
| 1          | VCC    | 3.3V       |
| 2          | GND    | GND        |
| 3          | TCK    | GPIO19     |
| 4          | TDO    | GPIO35     |
| 5          | TDI    | GPIO23     |
| 6          | TMS    | GPIO18     |

Pin 1 (VCC) must be connected for the FPGA's JTAG transceiver to operate correctly. Failure to connect VCC results in TDO returning all-zeros regardless of the shift operation.

## F.4 WiFi-JTAG Transport

The JTAG transport layer is implemented using an ESP32-D0WD-V3 microcontroller running a custom Xilinx Virtual Cable (XVC) server. The ESP32 connects to the local WiFi network (2.4 GHz band only — ESP32 does not support 5 GHz) and listens for TCP connections on port 2542.

### F.4.1 Architecture

```
Host Computer                    ESP32                          FPGA
┌─────────────┐               ┌──────────┐               ┌──────────┐
│ openFPGA    │  TCP:2542     │ XVC      │  GPIO         │ JTAG     │
│ Loader      │──────────────>│ Server   │──────────────>│ TAP      │
│             │  (WiFi)       │          │  (Dupont)     │          │
│ xvc-client  │               │ port 2542│               │ XC7A100T │
└─────────────┘               └──────────┘               └──────────┘
  192.168.1.x                  192.168.1.30               JTAG header
```

### F.4.2 Latency Characterization

| Path                         | Typical Latency |
|------------------------------|-----------------|
| TCP round-trip (LAN)         | ~1 ms           |
| XVC command processing       | ~0.5 ms         |
| JTAG shift (per bit)         | ~166 ns         |
| Full IDCODE read (32 bits)   | ~5.3 us + overhead |
| Full bitstream program (3.8 MB) | ~60 seconds  |

### F.4.3 WiFi-XVC vs USB-JTAG Comparison

| Property            | WiFi-XVC (ESP32)    | USB-JTAG (FTDI)     |
|---------------------|----------------------|----------------------|
| Latency per shift   | ~2 ms               | ~0.1 ms             |
| Bandwidth           | ~1 Mbps (TCP)       | ~30 Mbps (USB 2.0)  |
| Programming time    | ~60 s (3.8 MB)      | ~5 s (3.8 MB)       |
| Cable length        | Wireless (10m+)     | USB (5m max)        |
| Multi-client        | Yes (TCP)           | No (USB exclusive)  |
| Cost                | ~$5 (ESP32)         | ~$40 (FTDI cable)   |
| Portability         | Phone/tablet access | Requires USB host   |

The WiFi transport sacrifices ~12x speed for wireless convenience and dramatically lower cost. For development and testing (where bitstreams are programmed infrequently), this trade-off is favorable. For production deployment, a USB-JTAG interface would be recommended.

## F.5 IDCODE Verification

The FPGA's JTAG TAP controller returns IDCODE `0x13631093` when interrogated via the `shift` command. This 32-bit value decodes as:

```
Bit [31:28] = 0001 (version 1)
Bit [27:12] = 0011_0110_0011_0001 (device ID = 0x3631, XC7A200T)
Bit [11:1]  = 00100100100 (manufacturer = Xilinx, 0x049)
Bit [0]     = 1 (IDCODE marker)
```

The device ID `0x3631` corresponds to an XC7A200T, despite the board being labeled "XC7A100T". This is consistent with the XC7A200T being a superset of the XC7A100T — a design targeting the 100T will work on the 200T without modification.

## F.6 Critical Bug: Little-Endian Shift Length

During initial deployment, a critical endianness bug was discovered in the interaction between `openFPGALoader` and the ESP32 XVC server. The XVC `shift` command format specifies a 4-byte length field. The original ESP32 firmware interpreted this field as **big-endian** (network byte order), consistent with the XVC specification's convention.

However, `openFPGALoader` transmits the shift length in **little-endian** byte order. The raw bytes `0x0a 0x00 0x00 0x00` were interpreted as:

- **Big-endian interpretation**: `0x0a000000` = 167,772,160 bits (167M bits)
- **Little-endian interpretation**: `0x0000000a` = 10 bits (correct)

The big-endian interpretation caused the ESP32 to attempt allocating `167772160 / 8 = 20,971,520 bytes` of heap memory for the TMS and TDI buffers, which immediately failed on the ESP32's 520 KB SRAM. The server then hung without sending a response, causing `openFPGALoader` to timeout.

The fix was a single-line change in the ESP32 firmware's shift handler:

```cpp
// Before (incorrect):
uint32_t length = (buf[0] << 24) | (buf[1] << 16) | (buf[2] << 8) | buf[3];

// After (correct):
uint32_t length = buf[0] | (buf[1] << 8) | (buf[2] << 16) | (buf[3] << 24);
```

This bug illustrates a common pitfall in protocol implementation: the XVC specification does not mandate a specific endianness for the shift length field, and different clients may use different conventions. The fix is documented in the ESP32 firmware source (`firmware/xvc-esp32/xvc-esp32.ino`) with a comment noting the `openFPGALoader`-specific little-endian requirement.

## F.7 Configuration Status

After deploying `design.bit` via the WiFi-JTAG transport, the FPGA's STATUS register reads `0x401079FC`, confirming successful configuration:

```
CRC Error       = No error
DONE            = 1 (configuration complete)
INIT_B          = 1 (initialization complete)
EOS             = 1 (end of startup)
GWE             = 1 (global write enable)
GHIGH_B         = 1 (I/O active)
MMCM_LOCK       = 1 (clock stable)
DCI_MATCH       = 1 (impedance matched)
STARTUP_STATE   = 0x4 (post-startup)
```

The clean STATUS register — no CRC error, no ID error, no DEC error — confirms that the bitstream was transmitted correctly over the WiFi-JTAG link and that the FPGA successfully loaded the configuration.

## F.8 Resource Utilization

The initial deployment (D=10,000 bind/bundle/unbundle) targets the following utilization:

| Module          | LUTs  | FFs   | BRAM | DSP | % LUT |
|-----------------|-------|-------|------|-----|-------|
| vsa_bind        | 10000 | 20000 | 0    | 0   | 9.86% |
| vsa_bundle      | 15000 | 20000 | 0    | 0   | 14.8% |
| vsa_top (mux)   | 200   | 2     | 0    | 0   | 0.2%  |
| **Total**       | **~25200** | **~40002** | **0** | **0** | **~24.8%** |

The D=10,000 configuration uses approximately 25% of the XC7A100T LUT resources, leaving 75% available for future expansion (inference pipeline, similarity search, codebook memory).

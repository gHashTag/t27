# Appendix J: JTAG Debug Protocol via ESP32-XVC Bridge

## J.1 Introduction

This appendix documents the Xilinx Virtual Cable (XVC) protocol as implemented in the ESP32-based JTAG bridge used to program and debug the QMTech XC7A100T FPGA. The XVC protocol provides a TCP-based tunnel for JTAG operations, enabling wireless FPGA configuration from any network-connected host.

The protocol analysis is based on packet captures obtained during the 6-hour debugging session on 2026-05-05, during which the ESP32 XVC server was developed and verified against `openFPGALoader`.

## J.2 XVC Protocol Specification (v1.0)

### J.2.1 Connection

The XVC server listens on TCP port 2542. Upon connection, the client queries server capabilities with the `getinfo:` command and receives a version string.

### J.2.2 Command Format

All XVC commands are ASCII-encoded with binary payload. The general format is:

```
Command: <ASCII command name>:<binary payload>
```

The colon (`:`) is the delimiter between the command name and the payload. Commands are not null-terminated.

### J.2.3 Commands

#### `getinfo:`

Queries server version and maximum shift length.

Request:
```
getinfo:                    (8 bytes ASCII, no null terminator)
```

Hex: `67 65 74 69 6E 66 6F 3A`

Response:
```
xvcServer_v1.0:2048\n       (22 bytes ASCII + newline)
```

Hex: `78 76 63 53 65 72 76 65 72 5F 76 31 2E 30 3A 32 30 34 38 0A`

The `2048` value indicates the maximum shift length in bits supported by the server. This is an advisory limit — the ESP32 implementation accepts shifts up to 16384 bytes (131,072 bits) with dynamic allocation.

#### `settck:<period>`

Sets the TCK period in nanoseconds.

Request:
```
settck:<4-byte period>
```

The period is a 4-byte **little-endian** unsigned integer.

Example: `settck:` followed by `A6 00 00 00` (LE) = 166 ns period = ~6 MHz TCK frequency.

Hex capture: `73 65 74 74 63 6B 3A A6 00 00 00`

Response: The server echoes the period as a 4-byte little-endian integer:
```
A6 00 00 00                (166 ns acknowledged)
```

**Implementation note**: The ESP32 firmware must read exactly 6 bytes after detecting the 's' character: the 5 bytes "ettck" + the colon ':'. Reading only 5 bytes (missing the colon) causes the subsequent period bytes to be shifted by one position, producing incorrect TCK timing.

#### `shift:<length><tms_bytes><tdi_bytes>`

Performs a JTAG shift operation, clocking TMS and TDI while capturing TDO.

Request format:
```
shift:<4-byte length><tms_bytes><tdi_bytes>
```

Where:
- `length`: 4-byte **little-endian** unsigned integer — number of bits to shift
- `tms_bytes`: `ceil(length/8)` bytes — TMS bit pattern
- `tdi_bytes`: `ceil(length/8)` bytes — TDI bit pattern

Example: JTAG reset (shift 10 bits with TMS=all-1):
```
shift:0A 00 00 00          (length = 10 bits, little-endian)
      BF 00 00 00          (TMS: 0xBF = 10111111, padded to 4 bytes)
      00 00 00 00          (TDI: all zeros)
```

Hex capture: `73 68 69 66 74 3A 0A 00 00 00 BF 00 00 00 00 00 00 00`

Response: `ceil(length/8)` bytes of TDO data.

Response for 10-bit shift: 2 bytes of TDO data.

## J.3 Critical Bug: Shift Length Endianness

### J.3.1 Symptom

During initial testing, the ESP32 XVC server would accept the `getinfo:` and `settck:` commands correctly but hang when receiving `shift:` commands. The server would stop responding to TCP keepalives, and `openFPGALoader` would timeout after 30 seconds.

### J.3.2 Root Cause

The `shift:` command includes a 4-byte length field. The original firmware parsed this as **big-endian** (network byte order):

```cpp
// Original (incorrect for openFPGALoader):
uint32_t length = (buf[0] << 24) | (buf[1] << 16) | (buf[2] << 8) | buf[3];
```

When `openFPGALoader` sent a 10-bit shift, the raw bytes were:
```
0x0A 0x00 0x00 0x00
```

- Big-endian interpretation: `0x0A000000` = **167,772,160 bits** (167 million)
- Little-endian interpretation: `0x0000000A` = **10 bits** (correct)

The server attempted to allocate `167772160 / 8 = 20,971,520 bytes` for both TMS and TDI buffers, totaling ~40 MB — far exceeding the ESP32's 520 KB SRAM. The `malloc()` call returned NULL, and the subsequent `read_all()` with a NULL buffer caused undefined behavior (watchdog reset).

### J.3.3 Proxy Intercept Evidence

The bug was diagnosed by intercepting the TCP stream between `openFPGALoader` and the ESP32. The proxy captured the raw byte sequences:

```
[Client -> Server] getinfo:
  67 65 74 69 6E 66 6F 3A

[Server -> Client] xvcServer_v1.0:2048\n
  78 76 63 53 65 72 76 65 72 5F 76 31 2E 30 3A 32 30 34 38 0A

[Client -> Server] settck: (166 ns)
  73 65 74 74 63 6B 3A A6 00 00 00

[Client -> Server] shift: 10 bits
  73 68 69 66 74 3A 0A 00 00 00 BF 00 00 00 00 00 00 00
                 ^^^^^^^^^^^^
                 0x0A 0x00 0x00 0x00 = 10 (LE) NOT 167772160 (BE)
```

### J.3.4 Fix

```cpp
// Corrected (little-endian for openFPGALoader compatibility):
uint32_t length = buf[0] | (buf[1] << 8) | (buf[2] << 16) | (buf[3] << 24);
```

This single-line change resolved the hang, and the server successfully processed all subsequent shift operations.

## J.4 IDCODE Detection

After fixing the endianness bug, the JTAG chain was successfully detected:

```
$ openFPGALoader --cable xvc-client --ip 192.168.1.30 --port 2542 --detect

found 1 devices
index 0:
  idcode 0x3631093
  manufacturer xilinx
  family artix a7 100t
  model  xc7a100
  irlength 6
```

### J.4.1 IDCODE Decoding

The 32-bit IDCODE `0x13631093` decodes as:

```
Bit  [31:28] = 0001          Version = 1
Bit  [27:12] = 0011011000110001  Part Number = 0x3631 (XC7A200T)
Bit  [11:1]  = 00100100100   Manufacturer = Xilinx (JTAG ID 0x049)
Bit  [0]     = 1             Required by IEEE 1149.1
```

Binary: `0001_00110110_00110001_00100100100_1`
Hex:    `0x13631093`

The part number `0x3631` corresponds to an XC7A200T, which is the actual silicon on the QMTech board despite the "XC7A100T" labeling. The `irlength 6` confirms the Xilinx 7-series instruction register width.

## J.5 TAP State Machine

The JTAG Test Access Port (TAP) controller is a 16-state finite state machine. All transitions are controlled by TMS, sampled on the rising edge of TCK:

```
                    TMS=1
    ┌──────────────────────────────────────┐
    │                                      │
    ▼              TMS=0                   │
 ┌──────────┐   ┌──────────┐              │
 │Test-Logic│──>│  Run-    │              │
 │  Reset   │   │  Idle    │              │
 └──────────┘   └──────────┘              │
    │ TMS=1         │ TMS=1               │
    │               ▼                      │
    │         ┌──────────┐                │
    │         │Select-   │                │
    │         │  DR      │                │
    │         └──────────┘                │
    │          │ TMS=0       │ TMS=1      │
    │          ▼             ▼            │
    │    ┌──────────┐  ┌──────────┐       │
    │    │Capture-  │  │Select-   │       │
    │    │  DR      │  │  IR      │       │
    │    └──────────┘  └──────────┘       │
    │          │ TMS=0       │ TMS=0      │
    │          ▼             ▼            │
    │    ┌──────────┐  ┌──────────┐       │
    │    │ Shift-   │  │Capture-  │       │
    │    │  DR  <───│  │  IR      │       │
    │    └──────────┘  └──────────┘       │
    │          │ TMS=1       │ TMS=0      │
    │          ▼             ▼            │
    │    ┌──────────┐  ┌──────────┐       │
    │    │Exit1-    │  │ Shift-   │       │
    │    │  DR      │  │  IR  <───│       │
    │    └──────────┘  └──────────┘       │
    │                                  │
    └──────────────────────────────────┘
```

Key state transitions for FPGA programming:

| Operation            | States (TMS sequence)                          |
|----------------------|------------------------------------------------|
| Reset TAP            | 5x TMS=1 → Test-Logic-Reset                    |
| Read IDCODE          | Reset → Idle → SelectDR → CaptureDR → ShiftDR  |
| Load IR              | Reset → Idle → SelectDR → SelectIR → CaptureIR → ShiftIR |
| Load DR (bitstream)  | Reset → Idle → SelectDR → CaptureDR → ShiftDR  |

The 10-bit shift with TMS=`0xBF` (`10111111`) seen in the XVC capture performs a TAP reset: it keeps TMS=1 for 5 clocks (entering Test-Logic-Reset), then shifts 5 more bits with mixed TMS values to transition to the desired operating state.

## J.6 ESP32 XVC Server Implementation

### J.6.1 Architecture

The ESP32 XVC server runs as a FreeRTOS task on core 1 (core 0 handles WiFi). The server loop:

1. Accepts TCP connection on port 2542
2. Reads command byte
3. Dispatches to handler based on first character ('g'=getinfo, 's'=settck/shift)
4. For `shift`: reads length (LE), allocates TMS/TDI buffers, performs bit-level JTAG
5. Sends TDO response

### J.6.2 JTAG Bit-Level Processing

The server processes JTAG shifts at the bit level, not the byte level. For each bit position `i` in the shift:

```cpp
for (int bit = 0; bit < length; bit++) {
    int byte_idx = bit / 8;
    int bit_idx = bit % 8;

    // Set TMS and TDI
    digitalWrite(TMS_PIN, (tms_buf[byte_idx] >> bit_idx) & 1);
    digitalWrite(TDI_PIN, (tdi_buf[byte_idx] >> bit_idx) & 1);

    // Clock pulse
    digitalWrite(TCK_PIN, HIGH);
    delayMicroseconds(1);

    // Sample TDO
    int tdo = digitalRead(TDO_PIN);
    digitalWrite(TCK_PIN, LOW);

    // Store TDO
    if (tdo) tdo_buf[byte_idx] |= (1 << bit_idx);
}
```

The bit-level approach is slower than byte-level shifting but is compatible with any shift length (not just multiples of 8), which is required for JTAG operations like IDCODE read (32 bits) and IR loading (6 bits for Xilinx 7-series).

### J.6.3 Memory Management

The server uses heap allocation (`malloc`/`free`) for shift buffers, with a maximum shift length of 16,384 bytes (131,072 bits). Buffer allocation includes a NULL check with graceful error response to prevent crashes on oversized requests:

```cpp
uint8_t *tms_buf = (uint8_t *)malloc(num_bytes);
uint8_t *tdi_buf = (uint8_t *)malloc(num_bytes);
uint8_t *tdo_buf = (uint8_t *)calloc(num_bytes, 1);

if (!tms_buf || !tdi_buf || !tdo_buf) {
    client.println("ERROR: allocation failed");
    free(tms_buf); free(tdi_buf); free(tdo_buf);
    continue;
}
```

This graceful handling was added specifically to prevent the crash caused by the endianness bug described in Section J.3.

## J.7 Debugging Methodology

The XVC server was debugged using a three-layer approach:

1. **Serial monitor**: ESP32 `Serial.printf()` output for command parsing verification
2. **TCP proxy**: Custom proxy between `openFPGALoader` and ESP32, logging all bytes in hex
3. **Logic analysis**: DSLogic U2Basic on JTAG pins (attempted but not completed due to USB driver issues)

The TCP proxy was the most effective tool, as it revealed the exact byte sequences being exchanged and allowed comparison with the XVC specification. The hex dump format used in the proxy output became the basis for the protocol analysis in this appendix.

## J.8 Lessons Learned

1. **Endianness is not standardized in XVC**: Different XVC clients may send the shift length in different byte orders. The ESP32 server should detect or negotiate the endianness, or default to little-endian for `openFPGALoader` compatibility.

2. **Colon delimiters matter**: The `settck:` handler must read 6 bytes after 's' (including the ':'), not 5. Missing the colon shifts all subsequent byte parsing by one position.

3. **Heap allocation must be bounded**: The ESP32 has only 520 KB of SRAM. Any shift length exceeding ~200,000 bytes will fail allocation. The server must enforce a maximum shift length and reject oversized requests gracefully.

4. **WiFi latency is acceptable for development**: The ~2 ms round-trip latency of WiFi-XVC is acceptable for bitstream programming (~60 seconds) and JTAG debugging. For high-speed JTAG operations (repeated IDCODE reads, boundary scan), a wired connection would be necessary.

## J.9 Complete Session Timeline

| Time    | Event                                                    |
|---------|----------------------------------------------------------|
| 22:00   | Started ESP32 XVC server development                     |
| 22:10   | `getinfo:` working, `settck:` failing (5-byte read bug)  |
| 22:15   | Fixed settck: handler (6-byte read including colon)       |
| 22:20   | shift: command hanging — server stops responding          |
| 22:30   | Deployed TCP proxy to intercept byte stream               |
| 22:35   | Discovered big-endian shift length interpretation bug      |
| 22:38   | Fixed: LE byte order for openFPGALoader compatibility     |
| 22:40   | `--detect` succeeds: IDCODE `0x3631093` read              |
| 22:42   | Bitstream programming begins                              |
| 22:44   | DONE=1, STAT=`0x401079FC` — deployment successful         |

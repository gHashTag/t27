# JTAG Wiring Reference — QMTECH Wukong V1 (XC7A100T)

## JTAG Header Pinout

| JTAG Pin | Signal | FPGA Ball | DSLogic CH |
|----------|--------|-----------|------------|
| 1        | VREF   | 3V3       | —          |
| 2        | GND    | —         | GND        |
| 3        | TDO    | U15       | CH3        |
| 4        | TDI    | U14       | CH2        |
| 5        | TCK    | T14       | CH0        |
| 6        | TMS    | T15       | CH1        |
| 7        | SRST   | —         | —          |
| 8        | TRST   | —         | —          |
| 9        | DET    | —         | —          |
| 10       | GND    | —         | GND        |

## Cable Options

### Digilent DLC-10 (Xilinx Platform Cable USB II)
- Driver: `tools/dlc10_jtag.py` (commit `f5ad8be0`)
- VID: `0x03FD`, PID: `0x0008` (after firmware load)
- Known working: IDCODE=`0x03631093`, STATUS=`0x401079FC`

### ESP32 XVC (broken)
- Firmware: `firmware/esp32-xvc-firmware.bin`
- Known broken: IDCODE=`0x00001388`
- Issue: `shift:` commands produce garbage TDO
- Root cause: TBD (DSLogic diagnostics needed)

## DSView Capture

Config: `fpga/diagnostics/dsview_jtag_config.json`

```bash
# 1. Open DSView, load config
# 2. Start capture
# 3. Run JTAG operation:
python3 tools/dlc10_jtag.py --detect
# or
openFPGALoader --cable xvc-client --ip 192.168.1.30 --detect
# 4. Stop capture, save to fpga/diagnostics/captures/
```

## Expected JTAG Sequence (IDCODE read)

1. TMS: 5x TCK high (Test-Logic-Reset)
2. TMS: 0 (Run-Test/Idle)
3. TMS: 1,1,0 (Select-DR -> Capture-DR -> Shift-DR)
4. TDI: shift 32 bits of IDCODE instruction (0x09 for XC7A)
5. TDO: should return `0x03631093` for XC7A100T

## XC7A100T IDCODE Breakdown

```
0x03631093 = 0000 0011 0110 0011 0001 0000 1001 0011
              |----|----| |-version-| |---part---| |mfg|
              mfg=0x049 (Xilinx)  part=0x631  ver=0x3
```

## Reference

- `docs/fpga/clocking.md` — full pin mapping
- Issue #590 — DSLogic diagnostics tracking
- Issue #14 — FPGA flash verification

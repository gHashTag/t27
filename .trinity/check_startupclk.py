import sys
paths = [
    "fpga/tools/bscan_spi_xc7a100t_fgg676.bit",
    "fpga/tools/spiOverJtag_xc7a100tfgg676.bit.v2",
    "fpga/tools/bscan_spi_xc7a100t.bit",
    "fpga/tools/spiOverJtag_xc7a100tfgg676.bit",
]
# Xilinx 7-series Type-1 write packet:
#   header = 001 (type=1) | 010 (op=write) | 14-bit reg_addr | 11-bit word_count
# COR0 is reg 0x09, 1 word write → header = 0x30012001
# Bits in COR0: STARTUPCLK occupies bits [27:26] in 7-series (UG470, table 5-30):
#   00=Cclk, 01=UserClk, 10=JtagClk
# We'll check both [27:26] and [28:27] just to be safe.
COR0_HDR = bytes.fromhex("30012001")
SYNC = b"\xaa\x99\x55\x66"
for path in paths:
    try:
        data = open(path, "rb").read()
    except FileNotFoundError:
        print(f"{path}: NOT FOUND")
        continue
    i = data.find(SYNC)
    j = data.find(COR0_HDR, max(i,0))
    if j < 0:
        print(f"{path}: COR0 packet not found (sync@{i})")
        continue
    val = int.from_bytes(data[j+4:j+8], "big")
    names = {0:"Cclk(00)", 1:"UserClk(01)", 2:"JtagClk(10)", 3:"reserved(11)"}
    s_26 = (val >> 26) & 0x3
    s_27 = (val >> 27) & 0x3
    print(f"{path}")
    print(f"  COR0 = 0x{val:08x}")
    print(f"  STARTUPCLK[27:26] = {names[s_26]}")
    print(f"  STARTUPCLK[28:27] = {names[s_27]}")

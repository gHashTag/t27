#!/usr/bin/env python3
"""Decode the matmul_serial_hw 256-bit serial readback captured by DSLogic.

Wiring for capture (rewire DSLogic probes to the FPGA output pins):
    DSLogic CH0 -> E3   (12 MHz clock)      [sample reference]
    DSLogic CH1 -> R23  (led_data, serial)
    DSLogic CH2 -> T23  (led_sync, frame start)

Capture (DSLogic via sigrok-cli), e.g. 10 MS at 50 MHz to oversample the 12 MHz clock:
    sigrok-cli -d dreamsourcelab-dslogic --config samplerate=50m \
        --channels 0=clk,1=data,2=sync --samples 5000000 -O csv > cap.csv
    python3 decode_matmul_serial.py cap.csv

The decoder samples `data` on each rising edge of `clk`, starting at the edge where
`sync`==1, collects 256 bits (MSB first), unpacks 16 GF16 words c00..c33, decodes to
float, and compares to the expected A*I result.
"""
import sys

def gf16_decode(w):
    s=(w>>15)&1; e=(w>>9)&0x3F; m=w&0x1FF
    if e==0 and m==0: return -0.0 if s else 0.0
    if e==0x3F: return float('nan') if m else (float('-inf') if s else float('inf'))
    v=(1.0+m/512.0)*(2.0**(e-31)); return -v if s else v

EXPECTED = [0x3E00,0x4000,0x4100,0x4200, 0x4300,0x4380,0x4400,0x4440,
            0x4480,0x44C0,0x4500,0x4520, 0x4540,0x4560,0x4580,0x45A0]

def decode_stream(clk, data, sync):
    """clk/data/sync: equal-length 0/1 sample lists. Returns list of 256-bit ints (frames)."""
    frames=[]; prev=0; bits=[]; collecting=False
    for i in range(len(clk)):
        rise = (clk[i]==1 and prev==0); prev=clk[i]
        if not rise: continue
        if sync[i]==1:                      # frame start edge
            if collecting and len(bits)>=256:
                frames.append(int("".join(map(str,bits[:256])),2))
            bits=[]; collecting=True
        if collecting:
            bits.append(data[i])
            if len(bits)==256:
                frames.append(int("".join(map(str,bits)),2)); collecting=False
    return frames

def check_frame(val256):
    words=[(val256>>(16*(15-i)))&0xFFFF for i in range(16)]  # MSB-first: words[0]=c00
    ok = (words==EXPECTED)
    print("  words:", " ".join(f"{w:04x}" for w in words))
    print("  floats:", " ".join(f"{gf16_decode(w):.4g}" for w in words))
    print("  ==expected A:", ok)
    return ok

def self_test():
    # build the 256-bit expected stream and a synthetic clk/data/sync capture (4x oversample)
    val=0
    for w in EXPECTED: val=(val<<16)|w
    bitstr=f"{val:0256b}"
    clk=[];data=[];sync=[]
    OS=4
    # one leading partial frame of zeros then the real frame (mimics config-init frame 1)
    seq = "0"*256 + bitstr
    for idx,b in enumerate(seq):
        is_frame0 = (idx%256==0)
        for p in range(OS):
            clk.append(1 if p>=OS//2 else 0)         # rising edge mid-bit
            data.append(int(b)); sync.append(1 if is_frame0 else 0)
    frames=decode_stream(clk,data,sync)
    print(f"self-test: {len(frames)} frame(s) decoded")
    real=[f for f in frames if f!=0]
    return bool(real) and check_frame(real[-1])

if __name__=="__main__":
    if len(sys.argv)>1:
        import csv
        clk=[];data=[];sync=[]
        with open(sys.argv[1]) as f:
            r=csv.reader(x for x in f if not x.startswith(';'))
            hdr=next(r)
            ci=hdr.index('clk'); di=hdr.index('data'); si=hdr.index('sync')
            for row in r:
                if len(row)<=max(ci,di,si): continue
                clk.append(int(row[ci])); data.append(int(row[di])); sync.append(int(row[si]))
        frames=[f for f in decode_stream(clk,data,sync) if f!=0]
        if not frames: print("no non-zero frame found"); sys.exit(1)
        print(f"decoded {len(frames)} frame(s); checking last:")
        sys.exit(0 if check_frame(frames[-1]) else 1)
    else:
        print("SELF-TEST", "PASS" if self_test() else "FAIL")

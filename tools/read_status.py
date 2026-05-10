#!/usr/bin/env python3
"""Read Xilinx 7-series STATUS register via XVC — exact protocol match."""

import socket, struct, sys

HOST = "192.168.1.30"
PORT = 2542

def xvc_shift(sock, num_bits, tms_bytes, tdi_bytes):
    """Send shift command matching openFPGALoader exactly."""
    nb = (num_bits + 7) // 8
    pkt = struct.pack('<I', num_bits) + bytes(tms_bytes[:nb]) + bytes(tdi_bytes[:nb])
    sock.sendall(b"shift:" + pkt)
    
    tdo = bytearray(nb)
    got = 0
    while got < nb:
        chunk = sock.recv(nb - got)
        if not chunk:
            raise RuntimeError("Connection closed")
        tdo[got:got+len(chunk)] = chunk
        got += len(chunk)
    return bytes(tdo)

def jtag_go_rti(sock):
    """TLR -> RTI: TMS=01111 (5 bits), then RTI: TMS=0"""
    xvc_shift(sock, 5, bytes([0x1F]), bytes(1))   # TLR
    xvc_shift(sock, 1, bytes([0x00]), bytes(1))    # RTI

def jtag_shift_ir(sock, instruction, irlen=6):
    """Shift IR: RTI->SelDR->SelIR->CapIR->ShiftIR, shift instruction, Exit1IR->UpdIR->RTI"""
    # RTI -> Select-IR-Scan: TMS=11 (2 bits)  
    # Capture-IR: TMS=0 (enters automatically)
    # Shift-IR: shift irlen bits with TMS=0 for all but last (TMS=1)
    # Exit1-IR -> Update-IR: TMS=1
    # Update-IR -> RTI: TMS=0
    
    total_bits = 2 + irlen + 2
    tms = bytearray((total_bits + 7) // 8)
    tdi = bytearray((total_bits + 7) // 8)
    
    # Bits 0-1: TMS=11 (SelDR->SelIR)
    tms[0] |= 0x03
    # Bits 2 to 2+irlen-2: TMS=0 (stay in Shift-IR)
    # Bit 2+irlen-1: TMS=1 (exit Shift-IR to Exit1-IR)
    last_shift_bit = 2 + irlen - 1
    tms[last_shift_bit // 8] |= (1 << (last_shift_bit % 8))
    # Bit 2+irlen: TMS=1 (Exit1-IR -> Update-IR)
    upd_bit = 2 + irlen
    tms[upd_bit // 8] |= (1 << (upd_bit % 8))
    # Bit 2+irlen+1: TMS=0 (Update-IR -> RTI)
    
    # TDI: instruction bits starting at bit 2
    for i in range(irlen):
        bit_pos = 2 + i
        if (instruction >> i) & 1:
            tdi[bit_pos // 8] |= (1 << (bit_pos % 8))
    
    return xvc_shift(sock, total_bits, tms, tdi)

def jtag_shift_dr(sock, tdi_data):
    """Shift DR: RTI->SelDR->CapDR->ShiftDR, shift data, Exit1DR->UpdDR->RTI"""
    n = len(tdi_data)
    total_bits = 2 + n + 2
    tms = bytearray((total_bits + 7) // 8)
    tdi = bytearray((total_bits + 7) // 8)
    
    # Bits 0-1: TMS=10 (SelDR->CapDR)
    tms[0] |= 0x02
    # Bits 2 to 2+n-2: TMS=0 (stay in Shift-DR)
    # Bit 2+n-1: TMS=1 (exit to Exit1-DR)
    last_shift_bit = 2 + n - 1
    tms[last_shift_bit // 8] |= (1 << (last_shift_bit % 8))
    # Bit 2+n: TMS=1 (Exit1-DR -> Update-DR)
    upd_bit = 2 + n
    tms[upd_bit // 8] |= (1 << (upd_bit % 8))
    # Bit 2+n+1: TMS=0 (Update-DR -> RTI)
    
    # TDI data starting at bit 2
    for i in range(n):
        bit_pos = 2 + i
        if tdi_data[i]:
            tdi[bit_pos // 8] |= (1 << (bit_pos % 8))
    
    tdo_raw = xvc_shift(sock, total_bits, tms, tdi)
    
    # Extract TDO bits starting at bit 3 (after SelDR and CapDR)
    result = []
    for i in range(n):
        bit_pos = 3 + i  # TDO is one bit delayed
        if bit_pos // 8 < len(tdo_raw):
            result.append((tdo_raw[bit_pos // 8] >> (bit_pos % 8)) & 1)
        else:
            result.append(0)
    return result

def main():
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(10)
    sock.connect((HOST, PORT))
    
    # getinfo
    sock.sendall(b"getinfo:")
    info = sock.recv(1024)
    print(f"XVC: {info.strip().decode()}")
    
    # settck
    sock.sendall(b"settck:" + struct.pack('<I', 5000))
    sock.recv(1024)
    
    print("\n--- Going to RTI ---")
    jtag_go_rti(sock)
    
    print("--- Reading IDCODE ---")
    jtag_shift_ir(sock, 0x09)  # IDCODE = 0x09
    tdo = jtag_shift_dr(sock, [0]*32)
    idcode = 0
    for i in range(32):
        idcode |= (tdo[i] << i)
    print(f"IDCODE = 0x{idcode:08x}")
    
    print("\n--- Reading STATUS (IR=0x3C) ---")
    jtag_shift_ir(sock, 0x3C)  # STATUS register read
    tdo = jtag_shift_dr(sock, [0]*32)
    status = 0
    for i in range(32):
        status |= (tdo[i] << i)
    print(f"STATUS = 0x{status:08x}")
    
    # Decode status
    print(f"  CRC Error    = {(status >> 0) & 1}")
    print(f"  Part Secured = {(status >> 1) & 1}")
    print(f"  MMCM Lock    = {(status >> 2) & 1}")
    print(f"  DCI Match    = {(status >> 3) & 1}")
    print(f"  EOS          = {(status >> 4) & 1}")
    print(f"  GTS CFG B    = {(status >> 5) & 1}")
    print(f"  GWE          = {(status >> 6) & 1}")
    print(f"  GHIGH B      = {(status >> 7) & 1}")
    print(f"  MODE         = {(status >> 8) & 7}")
    print(f"  INIT Complete= {(status >> 11) & 1}")
    print(f"  INIT B       = {(status >> 12) & 1}")
    print(f"  Release Done = {(status >> 13) & 1}")
    print(f"  Done         = {(status >> 14) & 1}")
    print(f"  ID Error     = {(status >> 15) & 1}")
    
    sock.close()

if __name__ == "__main__":
    main()

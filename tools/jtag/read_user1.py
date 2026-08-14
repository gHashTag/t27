import sys; sys.path.insert(0,'tools/jtag')
from mpsse_jtag import Mpsse, CLK_BITS_OUT_NEG, CLK_TMS_OUT_NEG
def shift_ir(m, val):
    m.tms([1,1,0,0])
    m.w(bytes([CLK_BITS_OUT_NEG, 4, val & 0x1F]))
    m.w(bytes([CLK_TMS_OUT_NEG, 0, (((val>>5)&1) << 7) | 0x01]))
    m.tms([1,0])
def user1(idx=0, nbits=4):
    m = Mpsse(idx)
    try:
        m.tms([1,1,1,1,1]); m.tms([0])
        shift_ir(m, 0x02)
        m.tms([1,0,0])
        r = m.shift_dr_read(nbits); m.tms([1,1,0])
        return r[0] & ((1<<nbits)-1) if r else None
    finally:
        m.close()
if __name__ == "__main__":
    v = user1()
    print(f"  USER1 = 0b{v:04b}" if v is not None else "  нет данных")

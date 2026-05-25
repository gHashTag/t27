pub struct Crc32;

impl Crc32 {
    const POLY: u32 = 0xEDB88320;

    fn make_table() -> [u32; 256] {
        let mut table = [0u32; 256];
        for i in 0..256u32 {
            let mut crc = i;
            for _ in 0..8 {
                if crc & 1 != 0 { crc = (crc >> 1) ^ Self::POLY; }
                else { crc >>= 1; }
            }
            table[i as usize] = crc;
        }
        table
    }

    pub fn compute(data: &[u8]) -> u32 {
        let table = Self::make_table();
        let mut crc = 0xFFFFFFFFu32;
        for &b in data {
            let idx = ((crc ^ b as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ table[idx];
        }
        !crc
    }

    pub fn compute_continued(crc: u32, data: &[u8]) -> u32 {
        let table = Self::make_table();
        let mut crc = crc ^ 0xFFFFFFFF;
        for &b in data {
            let idx = ((crc ^ b as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ table[idx];
        }
        !crc
    }

    pub fn hex(data: &[u8]) -> String { format!("{:08x}", Self::compute(data)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() { assert_eq!(Crc32::hex(b""), "00000000"); }

    #[test]
    fn hello() { assert_eq!(Crc32::hex(b"hello"), "3610a686"); }

    #[test]
    fn deterministic() {
        assert_eq!(Crc32::compute(b"test"), Crc32::compute(b"test"));
    }

    #[test]
    fn different() { assert_ne!(Crc32::compute(b"foo"), Crc32::compute(b"bar")); }

    #[test]
    fn continued() {
        let full = Crc32::compute(b"hello world");
        let c1 = Crc32::compute(b"hello ");
        let c2 = Crc32::compute_continued(c1, b"world");
        assert_eq!(full, c2);
    }

    #[test]
    fn long_input() {
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let c = Crc32::compute(&data);
        assert_ne!(c, 0);
    }
}

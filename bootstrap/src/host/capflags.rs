pub const CAP_INFERENCE: u32 = 1 << 0;
pub const CAP_DMA: u32 = 1 << 1;
pub const CAP_IRQ: u32 = 1 << 2;
pub const CAP_TERNARY: u32 = 1 << 3;
pub const CAP_CRC32: u32 = 1 << 4;
pub const CAP_SCATTER_GATHER: u32 = 1 << 5;
pub const CAP_WEIGHT_VALIDATION: u32 = 1 << 6;
pub const CAP_MULTI_LAYER: u32 = 1 << 7;
pub const CAP_DEBUG_CSR: u32 = 1 << 8;
pub const CAP_WATCHDOG: u32 = 1 << 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapFlags(u32);

impl CapFlags {
    pub const fn none() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self(0x03FF)
    }

    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn has(self, cap: u32) -> bool {
        (self.0 & cap) != 0
    }

    pub const fn set(mut self, cap: u32) -> Self {
        self.0 |= cap;
        self
    }

    pub const fn clear(mut self, cap: u32) -> Self {
        self.0 &= !cap;
        self
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub fn missing(self, required: CapFlags) -> CapFlags {
        CapFlags(required.0 & !self.0)
    }

    pub fn satisfies(self, required: CapFlags) -> bool {
        (self.0 & required.0) == required.0
    }

    pub fn names(self) -> Vec<&'static str> {
        let mut result = Vec::new();
        if self.has(CAP_INFERENCE) { result.push("inference"); }
        if self.has(CAP_DMA) { result.push("dma"); }
        if self.has(CAP_IRQ) { result.push("irq"); }
        if self.has(CAP_TERNARY) { result.push("ternary"); }
        if self.has(CAP_CRC32) { result.push("crc32"); }
        if self.has(CAP_SCATTER_GATHER) { result.push("scatter_gather"); }
        if self.has(CAP_WEIGHT_VALIDATION) { result.push("weight_validation"); }
        if self.has(CAP_MULTI_LAYER) { result.push("multi_layer"); }
        if self.has(CAP_DEBUG_CSR) { result.push("debug_csr"); }
        if self.has(CAP_WATCHDOG) { result.push("watchdog"); }
        result
    }
}

impl std::fmt::Display for CapFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = self.names();
        if names.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", names.join("|"))
        }
    }
}

impl std::fmt::Binary for CapFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:010b}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityError {
    pub missing: CapFlags,
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "missing capabilities: {}", self.missing)
    }
}

impl std::error::Error for CapabilityError {}

pub fn check_capabilities(have: CapFlags, need: CapFlags) -> Result<(), CapabilityError> {
    let missing = have.missing(need);
    if missing.is_empty() {
        Ok(())
    } else {
        Err(CapabilityError { missing })
    }
}

pub const BASE_CAPABILITIES: CapFlags = CapFlags::none()
    .set(CAP_INFERENCE)
    .set(CAP_TERNARY)
    .set(CAP_CRC32);

pub const FULL_CAPABILITIES: CapFlags = CapFlags::all();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_empty() {
        assert!(CapFlags::none().is_empty());
        assert_eq!(CapFlags::none().count(), 0);
    }

    #[test]
    fn all_has_ten() {
        assert_eq!(CapFlags::all().count(), 10);
    }

    #[test]
    fn set_and_has() {
        let caps = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA);
        assert!(caps.has(CAP_INFERENCE));
        assert!(caps.has(CAP_DMA));
        assert!(!caps.has(CAP_IRQ));
    }

    #[test]
    fn clear() {
        let caps = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA).clear(CAP_INFERENCE);
        assert!(!caps.has(CAP_INFERENCE));
        assert!(caps.has(CAP_DMA));
    }

    #[test]
    fn satisfies() {
        let need = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA);
        let have = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA).set(CAP_IRQ);
        assert!(have.satisfies(need));
        assert!(!need.satisfies(have));
    }

    #[test]
    fn missing() {
        let have = CapFlags::none().set(CAP_INFERENCE);
        let need = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA);
        let miss = have.missing(need);
        assert!(miss.has(CAP_DMA));
        assert!(!miss.has(CAP_INFERENCE));
    }

    #[test]
    fn names() {
        let caps = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA);
        let names = caps.names();
        assert_eq!(names, vec!["inference", "dma"]);
    }

    #[test]
    fn display() {
        let caps = CapFlags::none().set(CAP_INFERENCE);
        assert_eq!(caps.to_string(), "inference");
        assert_eq!(CapFlags::none().to_string(), "none");
    }

    #[test]
    fn check_ok() {
        let have = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA);
        let need = CapFlags::none().set(CAP_INFERENCE);
        check_capabilities(have, need).unwrap();
    }

    #[test]
    fn check_missing() {
        let have = CapFlags::none().set(CAP_INFERENCE);
        let need = CapFlags::none().set(CAP_INFERENCE).set(CAP_DMA);
        let err = check_capabilities(have, need).unwrap_err();
        assert!(err.missing.has(CAP_DMA));
        assert!(err.to_string().contains("dma"));
    }

    #[test]
    fn from_bits_roundtrip() {
        let caps = CapFlags::from_bits(CAP_INFERENCE | CAP_TERNARY);
        assert!(caps.has(CAP_INFERENCE));
        assert!(caps.has(CAP_TERNARY));
        assert_eq!(caps.bits(), CAP_INFERENCE | CAP_TERNARY);
    }

    #[test]
    fn base_capabilities() {
        assert!(BASE_CAPABILITIES.has(CAP_INFERENCE));
        assert!(BASE_CAPABILITIES.has(CAP_TERNARY));
        assert!(BASE_CAPABILITIES.has(CAP_CRC32));
        assert!(!BASE_CAPABILITIES.has(CAP_DMA));
    }

    #[test]
    fn binary_format() {
        let caps = CapFlags::from_bits(0x03);
        assert_eq!(format!("{:b}", caps), "0000000011");
    }
}

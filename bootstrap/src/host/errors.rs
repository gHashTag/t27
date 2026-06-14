// Variant P (#969 dead-code audit, host/errors.rs layer). This module is a
// self-contained, intentional public API: a structured catalog of host-driver
// error codes (Severity, ErrorDomain, ErrorCode, the ERR_* constants, and the
// CATALOG lookup helpers). It is fully exercised by this module's own test
// suite (domain decoding, raw round-trip, catalog lookup/filter, severity
// ordering, display) but is not yet wired into production host code, so every
// symbol emits a `dead_code` warning (28 in total) in the non-test build.
// These are deliberate public surface, not dead code -- a single module-scoped
// allow documents that without removing or weakening any symbol. Scoped to
// `not(test)` so the test build still flags genuinely unused items, exactly as
// in the #1105 / #1111 slices of this audit.
#![cfg_attr(not(test), allow(dead_code))]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Fatal = 3,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Fatal => write!(f, "FATAL"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorDomain {
    Transport,
    Protocol,
    Session,
    Pipeline,
    Watchdog,
    Memory,
    Firmware,
    Config,
    Checksum,
    Generic,
}

impl std::fmt::Display for ErrorDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorDomain::Transport => write!(f, "transport"),
            ErrorDomain::Protocol => write!(f, "protocol"),
            ErrorDomain::Session => write!(f, "session"),
            ErrorDomain::Pipeline => write!(f, "pipeline"),
            ErrorDomain::Watchdog => write!(f, "watchdog"),
            ErrorDomain::Memory => write!(f, "memory"),
            ErrorDomain::Firmware => write!(f, "firmware"),
            ErrorDomain::Config => write!(f, "config"),
            ErrorDomain::Checksum => write!(f, "checksum"),
            ErrorDomain::Generic => write!(f, "generic"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode(u16);

impl ErrorCode {
    pub const fn new(domain: ErrorDomain, code: u8) -> Self {
        let domain_bits = match domain {
            ErrorDomain::Transport => 0x01,
            ErrorDomain::Protocol => 0x02,
            ErrorDomain::Session => 0x03,
            ErrorDomain::Pipeline => 0x04,
            ErrorDomain::Watchdog => 0x05,
            ErrorDomain::Memory => 0x06,
            ErrorDomain::Firmware => 0x07,
            ErrorDomain::Config => 0x08,
            ErrorDomain::Checksum => 0x09,
            ErrorDomain::Generic => 0x00,
        };
        Self(((domain_bits as u16) << 8) | code as u16)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }

    pub const fn domain(self) -> ErrorDomain {
        match (self.0 >> 8) as u8 {
            0x01 => ErrorDomain::Transport,
            0x02 => ErrorDomain::Protocol,
            0x03 => ErrorDomain::Session,
            0x04 => ErrorDomain::Pipeline,
            0x05 => ErrorDomain::Watchdog,
            0x06 => ErrorDomain::Memory,
            0x07 => ErrorDomain::Firmware,
            0x08 => ErrorDomain::Config,
            0x09 => ErrorDomain::Checksum,
            _ => ErrorDomain::Generic,
        }
    }

    pub const fn code(self) -> u8 {
        (self.0 & 0xFF) as u8
    }
}

pub const ERR_CRC_MISMATCH: ErrorCode = ErrorCode::new(ErrorDomain::Transport, 0x01);
pub const ERR_FRAME_SHORT: ErrorCode = ErrorCode::new(ErrorDomain::Transport, 0x02);
pub const ERR_PAYLOAD_OVERFLOW: ErrorCode = ErrorCode::new(ErrorDomain::Transport, 0x03);
pub const ERR_BAD_COMMAND: ErrorCode = ErrorCode::new(ErrorDomain::Protocol, 0x01);
pub const ERR_BAD_VERSION: ErrorCode = ErrorCode::new(ErrorDomain::Protocol, 0x02);
pub const ERR_BAD_RESPONSE: ErrorCode = ErrorCode::new(ErrorDomain::Protocol, 0x03);
pub const ERR_SEQ_MISMATCH: ErrorCode = ErrorCode::new(ErrorDomain::Session, 0x01);
pub const ERR_MAX_RETRIES: ErrorCode = ErrorCode::new(ErrorDomain::Session, 0x02);
pub const ERR_WRONG_STATE: ErrorCode = ErrorCode::new(ErrorDomain::Pipeline, 0x01);
pub const ERR_TIMEOUT: ErrorCode = ErrorCode::new(ErrorDomain::Pipeline, 0x02);
pub const ERR_HARDWARE: ErrorCode = ErrorCode::new(ErrorDomain::Pipeline, 0x03);
pub const ERR_WATCHDOG_EXPIRE: ErrorCode = ErrorCode::new(ErrorDomain::Watchdog, 0x01);
pub const ERR_POOL_EMPTY: ErrorCode = ErrorCode::new(ErrorDomain::Memory, 0x01);
pub const ERR_DOUBLE_FREE: ErrorCode = ErrorCode::new(ErrorDomain::Memory, 0x02);
pub const ERR_FW_BAD_MAGIC: ErrorCode = ErrorCode::new(ErrorDomain::Firmware, 0x01);
pub const ERR_FW_TRUNCATED: ErrorCode = ErrorCode::new(ErrorDomain::Firmware, 0x02);
pub const ERR_CONFIG_INVALID: ErrorCode = ErrorCode::new(ErrorDomain::Config, 0x01);
pub const ERR_CHECKSUM_FAIL: ErrorCode = ErrorCode::new(ErrorDomain::Checksum, 0x01);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    pub code: ErrorCode,
    pub severity: Severity,
    pub message: &'static str,
    pub recovery: &'static str,
}

impl CatalogEntry {
    pub const fn new(code: ErrorCode, severity: Severity, message: &'static str, recovery: &'static str) -> Self {
        Self { code, severity, message, recovery }
    }
}

pub const CATALOG: &[CatalogEntry] = &[
    CatalogEntry::new(ERR_CRC_MISMATCH, Severity::Error, "CRC mismatch", "Re-transmit frame"),
    CatalogEntry::new(ERR_FRAME_SHORT, Severity::Error, "Frame too short", "Check cable connection"),
    CatalogEntry::new(ERR_PAYLOAD_OVERFLOW, Severity::Error, "Payload too large", "Reduce payload size"),
    CatalogEntry::new(ERR_BAD_COMMAND, Severity::Warning, "Unknown command", "Update firmware"),
    CatalogEntry::new(ERR_BAD_VERSION, Severity::Warning, "Protocol version mismatch", "Update host driver"),
    CatalogEntry::new(ERR_BAD_RESPONSE, Severity::Warning, "Unknown response code", "Update firmware"),
    CatalogEntry::new(ERR_SEQ_MISMATCH, Severity::Error, "Sequence mismatch", "Reset session"),
    CatalogEntry::new(ERR_MAX_RETRIES, Severity::Error, "Max retries exceeded", "Check hardware, retry later"),
    CatalogEntry::new(ERR_WRONG_STATE, Severity::Warning, "Wrong pipeline state", "Reset pipeline"),
    CatalogEntry::new(ERR_TIMEOUT, Severity::Error, "Operation timed out", "Check FPGA status"),
    CatalogEntry::new(ERR_HARDWARE, Severity::Fatal, "Hardware error", "Power-cycle board"),
    CatalogEntry::new(ERR_WATCHDOG_EXPIRE, Severity::Fatal, "Watchdog expired", "Check heartbeat, power-cycle"),
    CatalogEntry::new(ERR_POOL_EMPTY, Severity::Error, "Memory pool exhausted", "Reduce concurrency"),
    CatalogEntry::new(ERR_DOUBLE_FREE, Severity::Error, "Double free detected", "Fix caller logic"),
    CatalogEntry::new(ERR_FW_BAD_MAGIC, Severity::Fatal, "Bad firmware magic", "Reflash firmware"),
    CatalogEntry::new(ERR_FW_TRUNCATED, Severity::Fatal, "Truncated firmware image", "Re-download firmware"),
    CatalogEntry::new(ERR_CONFIG_INVALID, Severity::Warning, "Invalid configuration", "Check parameter ranges"),
    CatalogEntry::new(ERR_CHECKSUM_FAIL, Severity::Error, "Checksum verification failed", "Re-transfer data"),
];

pub fn lookup(code: ErrorCode) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.code == code)
}

pub fn by_domain(domain: ErrorDomain) -> Vec<&'static CatalogEntry> {
    CATALOG.iter().filter(|e| e.code.domain() == domain).collect()
}

pub fn by_severity(severity: Severity) -> Vec<&'static CatalogEntry> {
    CATALOG.iter().filter(|e| e.severity == severity).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_domain() {
        assert_eq!(ERR_CRC_MISMATCH.domain(), ErrorDomain::Transport);
        assert_eq!(ERR_BAD_COMMAND.domain(), ErrorDomain::Protocol);
        assert_eq!(ERR_WRONG_STATE.domain(), ErrorDomain::Pipeline);
        assert_eq!(ERR_WATCHDOG_EXPIRE.domain(), ErrorDomain::Watchdog);
    }

    #[test]
    fn error_code_raw_roundtrip() {
        let c = ErrorCode::new(ErrorDomain::Session, 0x42);
        assert_eq!(c.code(), 0x42);
        assert_eq!(c.domain(), ErrorDomain::Session);
    }

    #[test]
    fn catalog_lookup_found() {
        let e = lookup(ERR_TIMEOUT).unwrap();
        assert_eq!(e.severity, Severity::Error);
        assert!(!e.message.is_empty());
        assert!(!e.recovery.is_empty());
    }

    #[test]
    fn catalog_lookup_not_found() {
        let unknown = ErrorCode::new(ErrorDomain::Generic, 0xFF);
        assert!(lookup(unknown).is_none());
    }

    #[test]
    fn by_domain_filter() {
        let transport = by_domain(ErrorDomain::Transport);
        assert!(!transport.is_empty());
        for e in &transport {
            assert_eq!(e.code.domain(), ErrorDomain::Transport);
        }
    }

    #[test]
    fn by_severity_fatal() {
        let fatal = by_severity(Severity::Fatal);
        assert!(!fatal.is_empty());
        for e in &fatal {
            assert_eq!(e.severity, Severity::Fatal);
        }
    }

    #[test]
    fn catalog_has_all_defined_codes() {
        assert!(lookup(ERR_CRC_MISMATCH).is_some());
        assert!(lookup(ERR_FRAME_SHORT).is_some());
        assert!(lookup(ERR_BAD_COMMAND).is_some());
        assert!(lookup(ERR_SEQ_MISMATCH).is_some());
        assert!(lookup(ERR_WRONG_STATE).is_some());
        assert!(lookup(ERR_TIMEOUT).is_some());
        assert!(lookup(ERR_HARDWARE).is_some());
        assert!(lookup(ERR_WATCHDOG_EXPIRE).is_some());
        assert!(lookup(ERR_POOL_EMPTY).is_some());
        assert!(lookup(ERR_DOUBLE_FREE).is_some());
        assert!(lookup(ERR_FW_BAD_MAGIC).is_some());
        assert!(lookup(ERR_CONFIG_INVALID).is_some());
        assert!(lookup(ERR_CHECKSUM_FAIL).is_some());
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Info.to_string(), "INFO");
        assert_eq!(Severity::Fatal.to_string(), "FATAL");
    }

    #[test]
    fn domain_display() {
        assert_eq!(ErrorDomain::Transport.to_string(), "transport");
        assert_eq!(ErrorDomain::Memory.to_string(), "memory");
    }

    #[test]
    fn catalog_entry_fields() {
        let e = CatalogEntry::new(ERR_TIMEOUT, Severity::Error, "msg", "fix");
        assert_eq!(e.message, "msg");
        assert_eq!(e.recovery, "fix");
    }

    #[test]
    fn catalog_size() {
        assert_eq!(CATALOG.len(), 18);
    }
}

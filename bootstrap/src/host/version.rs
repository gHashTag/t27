#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl SemVer {
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch }
    }

    pub const fn as_u32(self) -> u32 {
        ((self.major as u32) << 16) | ((self.minor as u32) << 8) | self.patch as u32
    }

    pub fn from_u32(v: u32) -> Self {
        Self {
            major: ((v >> 16) & 0xFF) as u8,
            minor: ((v >> 8) & 0xFF) as u8,
            patch: (v & 0xFF) as u8,
        }
    }

    pub fn is_compatible(&self, other: &SemVer) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub const HOST_VERSION: SemVer = SemVer::new(0, 5, 0);
pub const PROTOCOL_VERSION: SemVer = SemVer::new(1, 0, 0);
pub const MIN_FIRMWARE_VERSION: SemVer = SemVer::new(1, 0, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionError {
    Incompatible { host: SemVer, target: SemVer },
    FirmwareTooOld { have: SemVer, need: SemVer },
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionError::Incompatible { host, target } => {
                write!(f, "incompatible: host {host}, target {target}")
            }
            VersionError::FirmwareTooOld { have, need } => {
                write!(f, "firmware too old: {have}, need {need}")
            }
        }
    }
}

impl std::error::Error for VersionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionInfo {
    pub host: SemVer,
    pub protocol: SemVer,
    pub min_firmware: SemVer,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            host: HOST_VERSION,
            protocol: PROTOCOL_VERSION,
            min_firmware: MIN_FIRMWARE_VERSION,
        }
    }
}

impl VersionInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_firmware(&self, firmware: SemVer) -> Result<(), VersionError> {
        if firmware.major != self.min_firmware.major || firmware.minor < self.min_firmware.minor {
            return Err(VersionError::FirmwareTooOld {
                have: firmware,
                need: self.min_firmware,
            });
        }
        Ok(())
    }

    pub fn check_protocol(&self, remote: SemVer) -> Result<(), VersionError> {
        if !self.protocol.is_compatible(&remote) {
            return Err(VersionError::Incompatible {
                host: self.protocol,
                target: remote,
            });
        }
        Ok(())
    }

    pub fn version_string(&self) -> String {
        format!(
            "host={} protocol={} min_firmware={}",
            self.host, self.protocol, self.min_firmware
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildInfo {
    pub version: SemVer,
    pub git_hash: u32,
    pub build_timestamp: u64,
}

impl BuildInfo {
    pub const fn new(version: SemVer, git_hash: u32, build_timestamp: u64) -> Self {
        Self { version, git_hash, build_timestamp }
    }

    pub fn encode(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&self.version.as_u32().to_le_bytes());
        buf[4..8].copy_from_slice(&self.git_hash.to_le_bytes());
        buf[8..16].copy_from_slice(&self.build_timestamp.to_le_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }
        let version = SemVer::from_u32(u32::from_le_bytes([data[0], data[1], data[2], data[3]]));
        let git_hash = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let build_timestamp = u64::from_le_bytes([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15],
        ]);
        Some(Self { version, git_hash, build_timestamp })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_display() {
        assert_eq!(SemVer::new(1, 2, 3).to_string(), "1.2.3");
    }

    #[test]
    fn semver_ordering() {
        assert!(SemVer::new(1, 2, 3) < SemVer::new(1, 3, 0));
        assert!(SemVer::new(1, 2, 3) < SemVer::new(2, 0, 0));
        assert!(SemVer::new(0, 5, 0) < SemVer::new(0, 5, 1));
    }

    #[test]
    fn semver_as_u32_roundtrip() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(SemVer::from_u32(v.as_u32()), v);
    }

    #[test]
    fn semver_compatible_same_major() {
        let a = SemVer::new(1, 2, 0);
        let b = SemVer::new(1, 3, 0);
        assert!(b.is_compatible(&a));
        assert!(!a.is_compatible(&b));
    }

    #[test]
    fn semver_incompatible_major() {
        let a = SemVer::new(1, 0, 0);
        let b = SemVer::new(2, 0, 0);
        assert!(!a.is_compatible(&b));
        assert!(!b.is_compatible(&a));
    }

    #[test]
    fn version_info_default() {
        let v = VersionInfo::default();
        assert_eq!(v.host, HOST_VERSION);
        assert_eq!(v.protocol, PROTOCOL_VERSION);
    }

    #[test]
    fn check_firmware_ok() {
        let v = VersionInfo::new();
        v.check_firmware(SemVer::new(1, 0, 0)).unwrap();
        v.check_firmware(SemVer::new(1, 1, 0)).unwrap();
    }

    #[test]
    fn check_firmware_too_old() {
        let v = VersionInfo::new();
        let err = v.check_firmware(SemVer::new(0, 9, 0)).unwrap_err();
        assert!(matches!(err, VersionError::FirmwareTooOld { .. }));
    }

    #[test]
    fn check_firmware_wrong_major() {
        let v = VersionInfo::new();
        let err = v.check_firmware(SemVer::new(2, 0, 0)).unwrap_err();
        assert!(matches!(err, VersionError::FirmwareTooOld { .. }));
    }

    #[test]
    fn check_protocol_ok() {
        let v = VersionInfo::new();
        v.check_protocol(SemVer::new(1, 0, 0)).unwrap();
    }

    #[test]
    fn check_protocol_incompatible() {
        let v = VersionInfo::new();
        let err = v.check_protocol(SemVer::new(2, 0, 0)).unwrap_err();
        assert!(matches!(err, VersionError::Incompatible { .. }));
    }

    #[test]
    fn version_string() {
        let v = VersionInfo::new();
        let s = v.version_string();
        assert!(s.contains("host="));
        assert!(s.contains("protocol="));
    }

    #[test]
    fn build_info_roundtrip() {
        let b = BuildInfo::new(SemVer::new(1, 2, 3), 0xDEADBEEF, 123456789);
        let encoded = b.encode();
        let decoded = BuildInfo::decode(&encoded).unwrap();
        assert_eq!(decoded, b);
    }

    #[test]
    fn build_info_decode_too_short() {
        assert!(BuildInfo::decode(&[0u8; 8]).is_none());
    }

    #[test]
    fn error_display() {
        let e = VersionError::FirmwareTooOld {
            have: SemVer::new(0, 1, 0),
            need: SemVer::new(1, 0, 0),
        };
        assert!(e.to_string().contains("too old"));
        let e = VersionError::Incompatible {
            host: SemVer::new(1, 0, 0),
            target: SemVer::new(2, 0, 0),
        };
        assert!(e.to_string().contains("incompatible"));
    }
}

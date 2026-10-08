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

// ============================================================================
// Render Delivery Module
// ============================================================================

/// Locale enumeration for story reel variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Ru = 0,
    En = 1,
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locale::Ru => write!(f, "ru"),
            Locale::En => write!(f, "en"),
        }
    }
}

/// Delivery status for render jobs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Pending = 0,
    Processing = 1,
    Delivered = 2,
    Failed = 3,
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryStatus::Pending => write!(f, "pending"),
            DeliveryStatus::Processing => write!(f, "processing"),
            DeliveryStatus::Delivered => write!(f, "delivered"),
            DeliveryStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Locale-specific version metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleVersion {
    pub locale: Locale,
    pub version: SemVer,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl LocaleVersion {
    pub fn new(locale: Locale, version: SemVer) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            locale,
            version,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_version(&mut self, new_version: SemVer) {
        self.version = new_version;
        self.updated_at = std::time::SystemTime::now();
    }
}

/// Render delivery tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderDelivery {
    pub render_id: String,
    pub locale_versions: Vec<LocaleVersion>,
    pub delivery_status: DeliveryStatus,
    pub bot_delivery_enabled: bool,
    pub owner_id: String,
}

impl RenderDelivery {
    pub fn new(render_id: String, owner_id: String, bot_delivery_enabled: bool) -> Self {
        Self {
            render_id,
            locale_versions: Vec::new(),
            delivery_status: DeliveryStatus::Pending,
            bot_delivery_enabled,
            owner_id,
        }
    }

    pub fn add_locale_version(&mut self, locale: Locale, version: SemVer) {
        // Check if locale already exists
        if let Some(existing) = self.locale_versions.iter_mut().find(|lv| lv.locale == locale) {
            existing.update_version(version);
        } else {
            self.locale_versions.push(LocaleVersion::new(locale, version));
        }
    }

    pub fn get_locale_version(&self, locale: Locale) -> Option<&LocaleVersion> {
        self.locale_versions.iter().find(|lv| lv.locale == locale)
    }

    pub fn has_locale(&self, locale: Locale) -> bool {
        self.locale_versions.iter().any(|lv| lv.locale == locale)
    }

    pub fn get_locale_count(&self) -> usize {
        self.locale_versions.len()
    }

    pub fn set_delivery_status(&mut self, status: DeliveryStatus) {
        self.delivery_status = status;
    }

    pub fn is_delivery_complete(&self) -> bool {
        self.delivery_status == DeliveryStatus::Delivered
    }

    pub fn is_bot_delivery_enabled(&self) -> bool {
        self.bot_delivery_enabled
    }

    pub fn trigger_bot_delivery(&mut self) -> Result<(), String> {
        if !self.bot_delivery_enabled {
            return Err("Bot delivery is not enabled".to_string());
        }

        if self.locale_versions.is_empty() {
            return Err("No locale versions to deliver".to_string());
        }

        // Simulate bot delivery process
        self.delivery_status = DeliveryStatus::Processing;
        
        // In a real implementation, this would make an HTTP call to @t27ai_bot
        // For now, we'll just simulate successful delivery
        self.delivery_status = DeliveryStatus::Delivered;
        
        Ok(())
    }

    pub fn validate_render_readiness(&self) -> Result<(), String> {
        // Check if we have both RU and EN locales
        if !self.has_locale(Locale::Ru) || !self.has_locale(Locale::En) {
            return Err("Missing required locale variants".to_string());
        }

        // Check if delivery is already completed
        if self.is_delivery_complete() {
            return Err("Render already delivered".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod render_delivery_tests {
    use super::*;

    #[test]
    fn locale_display() {
        assert_eq!(Locale::Ru.to_string(), "ru");
        assert_eq!(Locale::En.to_string(), "en");
    }

    #[test]
    fn delivery_status_display() {
        assert_eq!(DeliveryStatus::Pending.to_string(), "pending");
        assert_eq!(DeliveryStatus::Delivered.to_string(), "delivered");
    }

    #[test]
    fn locale_version_creation() {
        let version = SemVer::new(1, 2, 3);
        let lv = LocaleVersion::new(Locale::Ru, version);
        assert_eq!(lv.locale, Locale::Ru);
        assert_eq!(lv.version, version);
    }

    #[test]
    fn locale_version_update() {
        let version1 = SemVer::new(1, 2, 3);
        let version2 = SemVer::new(1, 2, 4);
        let mut lv = LocaleVersion::new(Locale::Ru, version1);
        lv.update_version(version2);
        assert_eq!(lv.version, version2);
    }

    #[test]
    fn render_delivery_creation() {
        let delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        assert_eq!(delivery.render_id, "render123");
        assert_eq!(delivery.owner_id, "owner456");
        assert_eq!(delivery.bot_delivery_enabled, true);
        assert_eq!(delivery.delivery_status, DeliveryStatus::Pending);
        assert_eq!(delivery.get_locale_count(), 0);
    }

    #[test]
    fn add_locale_version() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        let version = SemVer::new(1, 0, 0);
        
        delivery.add_locale_version(Locale::Ru, version);
        assert_eq!(delivery.get_locale_count(), 1);
        assert!(delivery.has_locale(Locale::Ru));
        
        // Add same locale again (should update)
        let new_version = SemVer::new(1, 1, 0);
        delivery.add_locale_version(Locale::Ru, new_version);
        assert_eq!(delivery.get_locale_count(), 1);
        assert_eq!(delivery.get_locale_version(Locale::Ru).unwrap().version, new_version);
    }

    #[test]
    fn get_locale_version() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        let version = SemVer::new(1, 0, 0);
        
        delivery.add_locale_version(Locale::Ru, version);
        
        let result = delivery.get_locale_version(Locale::Ru);
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, version);
        
        let result = delivery.get_locale_version(Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn set_delivery_status() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        assert_eq!(delivery.delivery_status, DeliveryStatus::Pending);
        
        delivery.set_delivery_status(DeliveryStatus::Delivered);
        assert_eq!(delivery.delivery_status, DeliveryStatus::Delivered);
    }

    #[test]
    fn is_delivery_complete() {
        let mut delivery1 = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        let mut delivery2 = RenderDelivery::new("render456".to_string(), "owner789".to_string(), true);
        
        assert_eq!(delivery1.is_delivery_complete(), false);
        assert_eq!(delivery2.is_delivery_complete(), false);
        
        delivery1.set_delivery_status(DeliveryStatus::Delivered);
        assert_eq!(delivery1.is_delivery_complete(), true);
    }

    #[test]
    fn validate_render_readiness_success() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        let version = SemVer::new(1, 0, 0);
        
        delivery.add_locale_version(Locale::Ru, version);
        delivery.add_locale_version(Locale::En, version);
        
        let result = delivery.validate_render_readiness();
        assert!(result.is_ok());
    }

    #[test]
    fn validate_render_readiness_missing_locale() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        let version = SemVer::new(1, 0, 0);
        
        delivery.add_locale_version(Locale::Ru, version);
        // Missing EN locale
        
        let result = delivery.validate_render_readiness();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required locale variants"));
    }

    #[test]
    fn validate_render_readiness_already_delivered() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        let version = SemVer::new(1, 0, 0);
        
        delivery.add_locale_version(Locale::Ru, version);
        delivery.add_locale_version(Locale::En, version);
        delivery.set_delivery_status(DeliveryStatus::Delivered);
        
        let result = delivery.validate_render_readiness();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Render already delivered"));
    }

    #[test]
    fn trigger_bot_delivery_success() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        let version = SemVer::new(1, 0, 0);
        
        delivery.add_locale_version(Locale::Ru, version);
        delivery.add_locale_version(Locale::En, version);
        
        let result = delivery.trigger_bot_delivery();
        assert!(result.is_ok());
        assert_eq!(delivery.delivery_status, DeliveryStatus::Delivered);
    }

    #[test]
    fn trigger_bot_delivery_disabled() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), false);
        let version = SemVer::new(1, 0, 0);
        
        delivery.add_locale_version(Locale::Ru, version);
        delivery.add_locale_version(Locale::En, version);
        
        let result = delivery.trigger_bot_delivery();
        assert!(result.is_err());
        assert_eq!(delivery.delivery_status, DeliveryStatus::Pending);
    }

    #[test]
    fn trigger_bot_delivery_no_versions() {
        let mut delivery = RenderDelivery::new("render123".to_string(), "owner456".to_string(), true);
        
        let result = delivery.trigger_bot_delivery();
        assert!(result.is_err());
        assert_eq!(delivery.delivery_status, DeliveryStatus::Pending);
    }
}

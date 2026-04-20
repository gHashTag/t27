use anti_ban_audit::{checks, AuditReport, CheckResult};
use std::path::PathBuf;

#[test]
fn test_check_result_serialization() {
    let result = CheckResult {
        name: "test_check".into(),
        passed: true,
        message: "All good".into(),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("test_check"));
    assert!(json.contains("true"));
}

#[test]
fn test_audit_report_serialization() {
    let report = AuditReport {
        timestamp: "2026-04-21T00:00:00".into(),
        checks: vec![
            CheckResult {
                name: "check1".into(),
                passed: true,
                message: "pass".into(),
            },
            CheckResult {
                name: "check2".into(),
                passed: false,
                message: "fail".into(),
            },
        ],
        passed: false,
    };
    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("check1"));
    assert!(json.contains("check2"));
    assert!(json.contains("\"passed\": false"));
}

#[test]
fn test_no_sh_files_in_vendor() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let result = checks::check_no_sh_files(&root);
    assert!(result.name == "no_sh_files");
}

#[test]
fn test_no_force_merge_patterns() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let result = checks::check_no_force_merge(&root);
    assert!(result.name == "no_force_merge");
    assert!(result.passed, "Should have no force merge patterns in anti-ban crate");
}

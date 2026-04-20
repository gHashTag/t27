use anti_ban_audit::{checks, AuditReport};
use anyhow::Result;

fn main() -> Result<()> {
    let root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| {
            std::path::PathBuf::from(p)
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        })
        .unwrap_or(std::path::PathBuf::from("."));

    let checks = vec![
        checks::check_no_sh_files(&root),
        checks::check_cargo_clippy(&root),
    ];

    let passed = checks.iter().all(|c| c.passed);

    let report = AuditReport {
        timestamp: chrono_now(),
        checks,
        passed,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);

    if !passed {
        std::process::exit(1);
    }

    Ok(())
}

fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
        .ok();
    output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

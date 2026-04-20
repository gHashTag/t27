use std::path::Path;

pub fn check_no_sh_files(root: &Path) -> super::CheckResult {
    let mut found: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".sh") {
                    found.push(p.display().to_string());
                }
            }
            if p.is_dir() && !p.file_name().map(|n| n == ".git").unwrap_or(false) {
                let sub = check_no_sh_files(&p);
                found.append(
                    &mut sub
                        .message
                        .split('\n')
                        .filter(|l| !l.is_empty())
                        .map(String::from)
                        .collect(),
                );
            }
        }
    }
    if found.is_empty() {
        super::CheckResult {
            name: "no_sh_files".into(),
            passed: true,
            message: "No .sh files found".into(),
        }
    } else {
        super::CheckResult {
            name: "no_sh_files".into(),
            passed: false,
            message: found.join("\n"),
        }
    }
}

pub fn check_cargo_test(root: &Path) -> super::CheckResult {
    let output = std::process::Command::new("cargo")
        .args(["test", "--workspace", "--", "--test-threads=1"])
        .current_dir(root)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let count = String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| l.starts_with("test result:") && l.contains("passed"))
                .count();
            super::CheckResult {
                name: "cargo_test".into(),
                passed: true,
                message: format!("{} suites passed", count),
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let first_err = stderr
                .lines()
                .find(|l| l.contains("error"))
                .unwrap_or("unknown");
            super::CheckResult {
                name: "cargo_test".into(),
                passed: false,
                message: first_err.into(),
            }
        }
        Err(e) => super::CheckResult {
            name: "cargo_test".into(),
            passed: false,
            message: format!("cargo failed: {}", e),
        },
    }
}

pub fn check_cargo_clippy(root: &Path) -> super::CheckResult {
    let output = std::process::Command::new("cargo")
        .args([
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .current_dir(root)
        .output();

    match output {
        Ok(out) if out.status.success() => super::CheckResult {
            name: "cargo_clippy".into(),
            passed: true,
            message: "0 warnings".into(),
        },
        Ok(_) => super::CheckResult {
            name: "cargo_clippy".into(),
            passed: false,
            message: "clippy violations found".into(),
        },
        Err(e) => super::CheckResult {
            name: "cargo_clippy".into(),
            passed: false,
            message: format!("clippy failed: {}", e),
        },
    }
}

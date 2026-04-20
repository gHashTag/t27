use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

/// Scan directory recursively for files matching predicate
fn scan_files<F>(root: &Path, pred: F) -> Vec<String>
where
    F: Fn(&Path) -> bool + Copy,
{
    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if pred(&p) {
                found.push(p.display().to_string());
            }
            if p.is_dir() && !p.file_name().map(|n| n == ".git").unwrap_or(false) {
                found.extend(scan_files(&p, pred));
            }
        }
    }
    found
}

/// Read file content as string
fn read_file_content(path: &Path) -> String {
    File::open(path)
        .ok()
        .and_then(|f| BufReader::new(f).lines().collect::<Result<_, _>>().ok())
        .unwrap_or_default()
}

/// Check 1: NO .SH FILES - No shell scripts (Rust or TypeScript only)
pub fn check_no_sh_files(root: &Path) -> super::CheckResult {
    let found = scan_files(root, |p| {
        p.extension().map(|e| e == "sh").unwrap_or(false)
    });

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
            message: format!("Found {} .sh files:\n{}", found.len(), found.join("\n")),
        }
    }
}

/// Check 2: CARGO CLIPPY ZERO WARNINGS
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

/// Check 3: NO FIXED PORTS - No hardcoded ports (except 9005 for trios-server)
pub fn check_no_fixed_ports(root: &Path) -> super::CheckResult {
    let mut violations = Vec::new();
    let port_regex_1 = regex::Regex::new(r":\s*([0-9]{4,5})\b").unwrap();
    let port_regex_2 = regex::Regex::new(r#"port\s*[=:]\s*["']?([0-9]{4,5})"#).unwrap();

    for path in scan_files(root, |p| {
        p.extension()
            .map(|e| matches!(e.to_str(), Some("rs" | "toml" | "yaml" | "yml" | "ts" | "js")))
            .unwrap_or(false)
    }) {
        let content = read_file_content(Path::new(&path));
        for cap in port_regex_1.captures_iter(&content).chain(port_regex_2.captures_iter(&content)) {
            let port = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            // Allow 9005 (trios-server), 80, 443 (standard web)
            if port != "9005" && port != "80" && port != "443" && !port.is_empty() {
                violations.push(format!("{}: port {}", path, port));
            }
        }
    }

    if violations.is_empty() {
        super::CheckResult {
            name: "no_fixed_ports".into(),
            passed: true,
            message: "No fixed ports found (except 9005)".into(),
        }
    } else {
        super::CheckResult {
            name: "no_fixed_ports".into(),
            passed: false,
            message: format!("Found {} fixed port violations:\n{}", violations.len(), violations.join("\n")),
        }
    }
}

/// Check 4: NO UUID USAGE - No hardcoded UUIDs
pub fn check_no_uuid_usage(root: &Path) -> super::CheckResult {
    let uuid_regex = regex::Regex::new(
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
    ).unwrap();
    let mut violations = Vec::new();

    for path in scan_files(root, |p| {
        p.extension()
            .map(|e| matches!(e.to_str(), Some("rs" | "ts" | "js" | "json")))
            .unwrap_or(false)
    }) {
        let content = read_file_content(Path::new(&path));
        if uuid_regex.is_match(&content) {
            violations.push(path);
        }
    }

    if violations.is_empty() {
        super::CheckResult {
            name: "no_uuid_usage".into(),
            passed: true,
            message: "No hardcoded UUIDs found".into(),
        }
    } else {
        super::CheckResult {
            name: "no_uuid_usage".into(),
            passed: false,
            message: format!("Found UUIDs in {} files:\n{}", violations.len(), violations.join("\n")),
        }
    }
}

/// Check 5: NO SEQUENTIAL NAMING - No sequential file naming (test1.rs, test2.rs)
pub fn check_no_sequential_naming(root: &Path) -> super::CheckResult {
    let sequential_regex = regex::Regex::new(r"[a-z_]+(\d+)\.(rs|ts|js)$").unwrap();
    let mut sequences: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for path in scan_files(root, |p| {
        p.extension()
            .map(|e| matches!(e.to_str(), Some("rs" | "ts" | "js")))
            .unwrap_or(false)
    }) {
        if let Some(name) = Path::new(&path).file_name().and_then(|n| n.to_str()) {
            if let Some(cap) = sequential_regex.captures(name) {
                let prefix = cap.get(0).map(|m| m.as_str()).unwrap_or("").to_string();
                sequences
                    .entry(prefix)
                    .or_default()
                    .push(path.clone());
            }
        }
    }

    let violations: Vec<_> = sequences
        .values()
        .filter(|v| v.len() > 1)
        .flat_map(|v| v.iter().cloned())
        .collect();

    if violations.is_empty() {
        super::CheckResult {
            name: "no_sequential_naming".into(),
            passed: true,
            message: "No sequential naming found".into(),
        }
    } else {
        super::CheckResult {
            name: "no_sequential_naming".into(),
            passed: false,
            message: format!("Found {} sequentially named files:\n{}", violations.len(), violations.join("\n")),
        }
    }
}

/// Check 6: NO ENV LEAKAGE - No .env files committed to repo
pub fn check_no_env_leakage(root: &Path) -> super::CheckResult {
    let found = scan_files(root, |p| {
        p.file_name()
            .map(|n| {
                let s = n.to_str().unwrap_or("");
                // Allow .env.example, .env.local.example, .env.template
                (s == ".env" || s.starts_with(".env.") || s == "env.local")
                    && !s.ends_with(".example")
                    && !s.ends_with(".template")
            })
            .unwrap_or(false)
    });

    if found.is_empty() {
        super::CheckResult {
            name: "no_env_leakage".into(),
            passed: true,
            message: "No .env files found".into(),
        }
    } else {
        super::CheckResult {
            name: "no_env_leakage".into(),
            passed: false,
            message: format!("Found {} .env files:\n{}", found.len(), found.join("\n")),
        }
    }
}

/// Check 7: CARGO TEST PASSES
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
                message: format!("{} test suites passed", count),
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let first_err = stderr
                .lines()
                .find(|l| l.contains("error") || l.contains("FAILED"))
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
            message: format!("cargo test failed: {}", e),
        },
    }
}

/// Check 8: NO FORCE MERGE - No force push patterns in workflow files
pub fn check_no_force_merge(root: &Path) -> super::CheckResult {
    let mut violations = Vec::new();
    let force_patterns = [
        "git push --force",
        "git push -f",
        "force: true",
        "force-push",
        "git merge --force",
    ];

    for path in scan_files(root, |p| {
        p.extension()
            .map(|e| matches!(e.to_str(), Some("yml" | "yaml")))
            .unwrap_or(false)
            || p.to_str().map(|s| s.contains(".github/workflows")).unwrap_or(false)
    }) {
        let content = read_file_content(Path::new(&path));
        for pattern in &force_patterns {
            if content.contains(pattern) {
                violations.push(format!("{}: contains '{}'", path, pattern));
            }
        }
    }

    if violations.is_empty() {
        super::CheckResult {
            name: "no_force_merge".into(),
            passed: true,
            message: "No force merge patterns found".into(),
        }
    } else {
        super::CheckResult {
            name: "no_force_merge".into(),
            passed: false,
            message: format!("Found {} force merge violations:\n{}", violations.len(), violations.join("\n")),
        }
    }
}

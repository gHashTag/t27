// Trinity Experience CLI - Ring-011
// Commands: save, list

use std::fs;
use std::path::Path;
use chrono::Utc;
use serde_json::json;

const EXPERIENCE_DIR: &str = ".trinity/experience";

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.is_empty() {
        eprintln!("Usage: tri experience <save|list> [args...]");
        std::process::exit(1);
    }

    match args[0].as_str() {
        "save" => {
            if args.len() < 3 {
                eprintln!("Usage: tri experience save <skill> <payload>");
                std::process::exit(1);
            }
            let skill = &args[1];
            let payload = &args[2];

            if let Err(e) = experience_save(skill, payload) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }

            println!("✅ Experience saved: {} -> {}", skill, payload);
        }
        "list" => {
            if let Err(e) = experience_list() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[0]);
            std::process::exit(1);
        }
    }
}

pub fn experience_save(skill: &str, payload: &str) -> Result<(), String> {
    fs::create_dir_all(EXPERIENCE_DIR).map_err(|e| e.to_string())?;

    let timestamp = Utc::now().format("%Y%m%dT%H:%M:%SZ").to_string();
    let commit = git_head_short()?;
    let ring = current_ring()?;

    let fname = format!("{}/{}.{}.json", EXPERIENCE_DIR, skill, timestamp);

    let entry = json!({
        "skill": skill,
        "payload": payload,
        "timestamp": timestamp,
        "ring": ring,
        "commit": commit,
    });

    fs::write(&fname, entry.to_string()).map_err(|e| e.to_string())?;

    println!("Experience saved: {}", fname);
    Ok(())
}

pub fn experience_list() -> Result<(), String> {
    let dir = Path::new(EXPERIENCE_DIR);

    if !dir.exists() {
        println!("(empty)");
        return Ok(());
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                let skill = v.get("skill").and_then(|s| s.as_str()).unwrap_or("?");
                let timestamp = v.get("timestamp").and_then(|t| t.as_str()).unwrap_or("?");
                println!("{:<40} {:<30} {:<30}", skill, timestamp);
                entries.push((skill, timestamp, content));
            }
        }
    }

    if entries.is_empty() {
        return Ok(());
    }

    // Sort by timestamp
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", "-".repeat(90));
    for entry in &entries {
        let (skill, timestamp, content) = entry;
        println!("{:<40} {:<30} {}",
            skill,
            timestamp,
            content.lines().take(10).join("...")
        );
        if entries.len() > 1 {
            println!("{}", "-".repeat(90));
        }
    }

    Ok(())
}

fn git_head_short() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;

    let hash = output.trim().chars().take(7).collect::<String>();
    Ok(hash)
}

fn current_ring() -> Result<String, String> {
    // Simple heuristic: check ring-000 through ring-012 completion
    let mut current = "000";
    let experience_dir = Path::new(EXPERIENCE_DIR);

    if experience_dir.exists() {
        for entry in fs::read_dir(experience_dir).map_err(|e| e.to_string())? {
            if let Ok(content) = fs::read_to_string(&entry.path()) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(ring) = v.get("ring").and_then(|r| r.as_str()) {
                        let ring_num = ring.parse::<u32>().unwrap_or(0);
                        if ring_num >= current.parse::<u32>().unwrap_or(0) {
                            current = ring.clone();
                        }
                    }
                }
            }
        }
    }

    Ok(format!("ring-{}", current))
}
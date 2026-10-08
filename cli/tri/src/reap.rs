use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Subcommand)]
pub enum ReapCmd {
    /// Remove old target directories from /data that haven't been written to in 48h
    Run {
        /// Path to the data directory (default: /data)
        #[arg(long, default_value = "/data")]
        data_path: String,
        /// Age threshold in hours (default: 48)
        #[arg(long, default_value = "48")]
        age_hours: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ReapReport {
    dirs_kept: Vec<String>,
    dirs_removed: Vec<String>,
    space_freed: u64,
    timestamp: String,
}

fn get_file_mtime(path: &Path) -> Result<SystemTime> {
    path.metadata()
        .and_then(|meta| meta.modified())
        .context("failed to get file modification time")
}

fn is_old_target_dir(path: &Path, age_threshold: Duration) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let dir_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    // Skip protected directories
    if dir_name == "src" || dir_name == "ref" || dir_name == "elan" {
        return Ok(false);
    }

    // Only process directories ending with -target
    if !dir_name.ends_with("-target") {
        return Ok(false);
    }

    // Check if directory is old enough
    let mtime = get_file_mtime(path)?;
    let age = SystemTime::now().duration_since(mtime)?;
    
    Ok(age > age_threshold)
}

fn get_directory_size(path: &Path) -> Result<u64> {
    let mut total = 0;
    
    for entry in walkdir::WalkDir::new(path).into_iter().filter_entry(|e| {
        // Skip .git directories to avoid traversing large repos
        e.path() != path && e.path().file_name() == Some(std::ffi::OsStr::new(".git"))
    }) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total += entry.metadata()?.len();
        }
    }
    
    Ok(total)
}

pub fn run(action: &ReapCmd) -> Result<()> {
    match action {
        ReapCmd::Run { data_path, age_hours } => {
            let data_path = PathBuf::from(data_path);
            let age_threshold = Duration::hours(*age_hours as i64);
            
            println!("🌾 Running reap on {} (threshold: {}h)", data_path.display(), age_hours);
            
            if !data_path.exists() {
                bail!("data directory does not exist: {}", data_path.display());
            }
            
            if !data_path.is_dir() {
                bail!("data path is not a directory: {}", data_path.display());
            }

            let mut dirs_kept = Vec::new();
            let mut dirs_removed = Vec::new();
            let mut space_freed = 0;

            // Read all directories in /data
            for entry in fs::read_dir(&data_path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    if is_old_target_dir(&path, age_threshold)? {
                        let dir_name = path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown");
                        
                        let size = get_directory_size(&path)?;
                        space_freed += size;
                        
                        println!("🗑️  Removing old target directory: {} ({} bytes)", dir_name, size);
                        
                        fs::remove_dir_all(&path)
                            .with_context(|| format!("failed to remove directory: {}", path.display()))?;
                        
                        dirs_removed.push(dir_name.to_string());
                    } else {
                        let dir_name = path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown");
                        
                        dirs_kept.push(dir_name.to_string());
                    }
                }
            }

            let report = ReapReport {
                dirs_kept,
                dirs_removed,
                space_freed,
                timestamp: Utc::now().to_rfc3339(),
            };

            println!("\n📊 Reap Report:");
            println!("================");
            println!("Timestamp: {}", report.timestamp);
            println!("Directories kept: {}", report.dirs_kept.len());
            for dir in &report.dirs_kept {
                println!("  ✓ {}", dir);
            }
            println!("\nDirectories removed: {}", report.dirs_removed.len());
            for dir in &report.dirs_removed {
                println!("  🗑️  {}", dir);
            }
            println!("\nSpace freed: {} bytes ({:.2} MB)", report.space_freed, report.space_freed as f64 / 1024.0 / 1024.0);
            println!("================");

            // Save report to file
            let report_path = data_path.join("reap-report.json");
            let report_content = serde_json::to_string_pretty(&report)?;
            fs::write(&report_path, report_content)
                .with_context(|| format!("failed to write report: {}", report_path.display()))?;
            
            println!("📄 Report saved to: {}", report_path.display());
            
            Ok(())
        }
    }
}
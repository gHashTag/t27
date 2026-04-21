//! File locking for issue #143 race condition prevention
//!
//! Uses `.trinity/tri.lock` to prevent concurrent updates
//! to the #143 table from multiple agents.

use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

#[cfg(unix)]
use libc::kill;

const LOCK_FILE: &str = ".trinity/tri.lock";
const LOCK_TIMEOUT_MS: u128 = 30000; // 30 seconds

/// Acquire file lock for #143 updates
pub struct LockGuard {
    _file: File,
}

impl LockGuard {
    /// Acquire lock with timeout
    ///
    /// Creates lock file with PID. Waits up to LOCK_TIMEOUT_MS
    /// if lock is held by another process.
    pub fn acquire() -> Result<Self> {
        let start = std::time::Instant::now();

        loop {
            match Self::try_acquire() {
                Ok(guard) => return Ok(guard),
                Err(e) => {
                    if start.elapsed().as_millis() > LOCK_TIMEOUT_MS {
                        anyhow::bail!(
                            "Failed to acquire lock after {}ms: {}",
                            LOCK_TIMEOUT_MS,
                            e
                        );
                    }

                    // Check if lock is stale (PID no longer running)
                    if let Ok(stale) = Self::is_lock_stale() {
                        if stale {
                            println!("Lock is stale, removing...");
                            fs::remove_file(LOCK_FILE).ok();
                            continue;
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    /// Try to acquire lock without waiting
    fn try_acquire() -> Result<Self> {
        let path = Path::new(LOCK_FILE);

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true) // Fails if exists
            .open(&path)
            .context("Failed to create lock file")?;

        // Write our PID
        let pid = std::process::id();
        writeln!(file, "{}", pid)?;

        Ok(LockGuard { _file: file })
    }

    /// Check if lock file is stale (PID not running)
    fn is_lock_stale() -> Result<bool> {
        let content = fs::read_to_string(LOCK_FILE)?;
        let pid: u32 = content.trim().parse()
            .context("Failed to parse lock PID")?;

        // Try to send signal 0 to check if process exists
        // This doesn't kill the process, just checks
        #[cfg(unix)]
        {
            if unsafe { kill(pid as i32, 0) } == 0 {
                return Ok(false); // Process exists
            }
            return Ok(true); // Process doesn't exist
        }

        #[cfg(not(unix))]
        {
            // On non-unix, assume stale if lock is old
            let metadata = fs::metadata(LOCK_FILE)?;
            let age = metadata.modified()?
                .elapsed()
                .context("Failed to get lock age")?;

            Ok(age.as_secs() > 3600) // 1 hour stale threshold
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Release lock by removing file
        fs::remove_file(LOCK_FILE).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_guard() {
        let guard1 = LockGuard::acquire().unwrap();
        let lock_file = Path::new(LOCK_FILE);
        assert!(lock_file.exists());

        // Second acquire should wait (we won't actually test timeout)
        drop(guard1);
        assert!(!lock_file.exists());
    }
}

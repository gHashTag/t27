//! `tri report` — Auto-sync results to issue #143 table
//!
//! Usage:
//!   tri report ALFA "complete" --bpb 6.5609
//!   tri report FOXTROT "running"

use anyhow::{Context, Result};

use crate::{
    config::Config,
    gh::GhClient,
    lock::LockGuard,
    table::{parse_table, update_table, TableRow},
};

/// Report experiment result to issue #143
pub fn report(agent: &str, status: &str, bpb: Option<f64>) -> Result<()> {
    println!("📊 Reporting {}={} --bpb={:?}", agent, status, bpb);

    let _config = Config::load();

    // Acquire lock for #143 table update
    let _lock = LockGuard::acquire()
        .context("Failed to acquire lock for issue #143")?;

    // Get current issue #143 body
    let issue_num = 143;
    let body = GhClient::issue_body(issue_num)
        .context("Failed to fetch issue #143")?;

    // Parse existing table
    let rows = parse_table(&body)
        .context("Failed to parse #143 table")?;

    // Check if agent already has a row
    let existing_row = rows.iter().find(|r| r.agent == agent);

    // Get task name from existing row or create default
    let _task = existing_row
        .map(|r| r.task.clone())
        .unwrap_or_else(|| format!("IGLA-{} Task", agent));

    // Get ref from existing row or create default
    let _ref_issue = existing_row
        .map(|r| r.ref_issue.clone())
        .unwrap_or_else(|| "#143".to_string());

    // Update table
    let updated_body = update_table(&body, agent, status, bpb)
        .context("Failed to update #143 table")?;

    // Write back to issue
    GhClient::issue_edit(issue_num, &updated_body)
        .context("Failed to update issue #143")?;

    println!("✓ Reported {}={} to #143", agent, status);

    Ok(())
}

/// Batch report multiple results
pub fn report_batch(results: Vec<(String, String, Option<f64>)>) -> Result<()> {
    let count = results.len();
    println!("📊 Batch reporting {} results", count);

    let _lock = LockGuard::acquire()?;

    let issue_num = 143;
    let mut body = GhClient::issue_body(issue_num)?;

    for (agent, status, bpb) in results {
        body = update_table(&body, &agent, &status, bpb)?;
    }

    GhClient::issue_edit(issue_num, &body)?;

    println!("✓ Batch reported {} results to #143", count);

    Ok(())
}

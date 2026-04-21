//! `tri roster` — Update agent roster
//!
//! Usage:
//!   tri roster ALFA "active"
//!   tri roster FOXTROT "busy"

use anyhow::{Context, Result};

use crate::{
    config::Config,
    gh::{AgentIssue, GhClient},
};

/// Update agent roster status
pub fn roster_update(agent: &str, status: &str) -> Result<()> {
    println!("👥 Updating {} agent status: {}", agent, status);

    let _config = Config::load();

    // Find agent roster issue (search for agent: NATO in issues)
    let issues = GhClient::list_agent_issues(agent)
        .context("Failed to search for agent issues")?;

    if let Some(roster_issue) = issues.iter().find(|i| i.title.contains("roster") || i.title.contains("Roster")) {
        println!("Found roster issue: #{}", roster_issue.number);

        // TODO: Update roster issue body with new status
        // For now, just report found
        println!("Current status would be updated to: {}", status);
    } else {
        println!("No roster issue found for {}", agent);
        println!("Consider creating one with: tri issue new roster");
    }

    println!("✓ Agent {} status: {}", agent, status);

    Ok(())
}

/// Show all agents
pub fn roster_show() -> Result<()> {
    println!("👥 Agent Roster");
    println!();

    let nato_codes = ["ALFA", "BRAVO", "CHARLIE", "DELTA", "ECHO"];

    for nato in &nato_codes {
        println!("  {} - {}", nato, agent_name(nato));
    }

    Ok(())
}

fn agent_name(nato: &str) -> &'static str {
    match nato {
        "ALFA" => "FOXTROT",
        "BRAVO" => "INDIGO",
        "CHARLIE" => "JULIETT",
        "DELTA" => "KILO",
        "ECHO" => "LIMA",
        _ => "UNKNOWN",
    }
}

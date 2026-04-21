//! `tri issue` — Issue management (new, close)
//!
//! Usage:
//!   tri issue new experiment "ALFA BigramHash(729)" --label agent --label experiment
//!   tri issue close 123 --bpb 6.5609

use anyhow::{Context, Result};

use crate::gh::GhClient;

pub enum IssueCmd {
    New { template: String, args: Vec<String> },
    Close { num: u32, bpb: Option<f64> },
}

/// Create new issue from template
pub fn issue_new(template: &str, args: &[String]) -> Result<u32> {
    println!("📝 Creating issue from template: {}", template);

    let (title, body, labels) = match template {
        "experiment" => {
            let agent = args.get(0).context("Missing agent name")?;
            let description = args.get(1).context("Missing description")?;

            (
                format!("{}: {}", agent, description),
                format!(
                    "## Experiment: {}\n\nAgent: {}\n\n### Goal\n\n{}\n\n### Config\n\nL10 (CPU-only)\n\n### Results\n\nPending",
                    description, agent, description
                ),
                &["agent", "experiment"][..],
            )
        }
        "bug" => {
            let title = args.get(0).context("Missing bug title")?;
            (
                format!("BUG: {}", title),
                format!(
                    "## Bug Report\n\n### Description\n\n{}\n\n### Steps to Reproduce\n\n1.\n2.\n3.\n\n### Expected Behavior\n\n<!-- What should happen -->\n\n### Actual Behavior\n\n<!-- What actually happens -->",
                    title
                ),
                &["bug"][..],
            )
        }
        "feature" => {
            let title = args.get(0).context("Missing feature title")?;
            (
                format!("FEATURE: {}", title),
                format!(
                    "## Feature Request\n\n### Description\n\n{}\n\n### Motivation\n\n<!-- Why is this needed? -->\n\n### Proposed Solution\n\n<!-- How should this work? -->",
                    title
                ),
                &["enhancement"][..],
            )
        }
        _ => anyhow::bail!("Unknown template: {}. Use: experiment, bug, feature", template),
    };

    let num = GhClient::issue_create(&title, &body, labels)?;

    println!("✓ Created issue #{}", num);

    Ok(num)
}

/// Close issue with optional BPB comment
pub fn issue_close(num: u32, bpb: Option<f64>) -> Result<()> {
    println!("🔒 Closing issue #{}", num);

    let comment = bpb.map(|b| format!("Final result: val_bpb={:.4}", b));

    if let Some(ref c) = comment {
        GhClient::comment(num, c)?;
    }

    // Close the issue via gh CLI
    std::process::Command::new("gh")
        .args(["issue", "close", &num.to_string()])
        .status()?;

    println!("✓ Closed issue #{}", num);

    Ok(())
}

use anyhow::Result;
use std::process::Command;

/// GitHub issue reference
#[derive(Debug, Clone)]
pub struct AgentIssue {
    pub number: u32,
    pub title: String,
}

/// GitHub CLI wrapper
pub struct GhClient;

impl GhClient {
    /// Create new client (no-op for gh CLI wrapper)
    pub fn new() -> Self {
        Self
    }

    pub fn issue_body(num: u32) -> Result<String> {
        let output = Command::new("gh")
            .args(["issue", "view", &num.to_string(), "--json", "body", "-q", ".body"])
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("gh issue view failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn issue_edit(num: u32, body: &str) -> Result<()> {
        let temp = std::env::temp_dir().join("tri-issue-body.md");
        std::fs::write(&temp, body)?;
        
        let status = Command::new("gh")
            .args(["issue", "edit", &num.to_string(), "--body-file", temp.to_str().unwrap()])
            .status()?;
        
        if !status.success() {
            anyhow::bail!("gh issue edit failed");
        }
        
        Ok(())
    }

    pub fn issue_create(title: &str, body: &str, labels: &[&str]) -> Result<u32> {
        let label_arg = labels.join(",");
        let output = Command::new("gh")
            .args(["issue", "create", "-R", "gHashTag/trios"])
            .args(["-t", title])
            .args(["-b", body])
            .args(["-l", &label_arg])
            .output()?;
        
        if !output.status.success() {
            anyhow::bail!("gh issue create failed");
        }
        
        // Parse URL to get issue number
        let url = String::from_utf8_lossy(&output.stdout);
        url.split("/issues/")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| anyhow::anyhow!("Failed to parse issue number"))
    }

    pub fn comment(issue: u32, body: &str) -> Result<()> {
        Command::new("gh")
            .args(["issue", "comment", &issue.to_string(), "-b", body])
            .status()?;
        
        Ok(())
    }

    /// List issues for a specific agent
    pub fn list_agent_issues(agent: &str) -> Result<Vec<AgentIssue>> {
        let output = Command::new("gh")
            .args([
                "issue", "list",
                "-R", "gHashTag/trios",
                "--search", agent,
                "--limit", "50",
                "--json", "number,title",
                "-q", ".[] | \"\\(.number) \\(.title)\""
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("gh issue list failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut issues = Vec::new();

        for line in stdout.lines() {
            if let Some(space_pos) = line.find(' ') {
                let number: u32 = line[..space_pos].parse().unwrap_or(0);
                let title = line[space_pos + 1..].to_string();
                if number > 0 {
                    issues.push(AgentIssue { number, title });
                }
            }
        }

        Ok(issues)
    }
}

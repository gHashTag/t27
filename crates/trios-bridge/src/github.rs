//! GitHub API integration — parse issue comments for agent status.

use anyhow::Result;
use crate::protocol::{AgentStatus, BridgeMessage};

/// GitHub API client for issue comment parsing.
pub struct GitHubClient {
    repo: String,
    token: Option<String>,
}

impl GitHubClient {
    /// Create a new GitHub client.
    /// `repo` should be in "owner/repo" format (e.g., "gHashTag/trios").
    pub fn new(repo: &str, token: Option<String>) -> Self {
        Self {
            repo: repo.to_string(),
            token,
        }
    }

    /// Fetch comments from a specific issue using octocrab.
    pub async fn get_issue_comments(&self, issue_number: u64) -> Result<Vec<IssueComment>> {
        let octocrab = match &self.token {
            Some(token) => octocrab::Octocrab::builder()
                .personal_token(token.clone())
                .build()?,
            None => octocrab::Octocrab::builder().build()?,
        };

        let (owner, repo) = self.repo
            .split_once('/')
            .unwrap_or(("gHashTag", "trios"));

        let page = octocrab
            .issues(owner, repo)
            .list_comments(issue_number)
            .send()
            .await?;

        let comments: Vec<octocrab::models::issues::Comment> = page
            .items
            .into_iter()
            .collect();

        Ok(comments
            .into_iter()
            .map(|c| IssueComment {
                id: c.id.0,
                issue_number,
                author: c.user.login.clone(),
                body: c.body.unwrap_or_default(),
            })
            .collect())
    }

    /// Parse agent commands from issue comments.
    /// Looks for patterns like `@agent-1 deploy` or `@all broadcast: message`.
    pub fn parse_agent_commands(comments: &[IssueComment]) -> Vec<BridgeMessage> {
        let mut messages = Vec::new();

        for comment in comments {
            for line in comment.body.lines() {
                let line = line.trim();

                // Parse @all broadcast commands
                if let Some(rest) = line.strip_prefix("@all ") {
                    messages.push(BridgeMessage::send_command(
                        "broadcast".into(),
                        rest.to_string(),
                        false,
                    ));
                } else if let Some(rest) = line.strip_prefix('@') {
                    // Parse @agent-id command
                    if let Some((agent_id, cmd)) = rest.split_once(' ') {
                        messages.push(BridgeMessage::send_command(
                            agent_id.to_string(),
                            cmd.to_string(),
                            false,
                        ));
                    }
                }

                // Parse status markers: `[STATUS: agent-1 WORKING training]`
                if line.starts_with("[STATUS:") {
                    let inner = line.trim_start_matches("[STATUS:").trim_end_matches(']');
                    let parts: Vec<&str> = inner.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let status = match parts[1] {
                            "IDLE" => AgentStatus::Idle,
                            "CLAIMING" => AgentStatus::Claiming,
                            "WORKING" => AgentStatus::Working,
                            "BLOCKED" => AgentStatus::Blocked,
                            "DONE" => AgentStatus::Done,
                            _ => AgentStatus::Idle,
                        };
                        let message = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
                        messages.push(BridgeMessage::update_status(
                            parts[0].to_string(),
                            status,
                            message,
                        ));
                    }
                }
            }
        }

        messages
    }
}

/// GitHub issue comment.
#[derive(Debug, Clone)]
pub struct IssueComment {
    pub id: u64,
    pub issue_number: u64,
    pub author: String,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_broadcast_command() {
        let comments = vec![IssueComment {
            id: 1,
            issue_number: 30,
            author: "general".to_string(),
            body: "@all STOP asking questions, just push".to_string(),
        }];

        let messages = GitHubClient::parse_agent_commands(&comments);
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            BridgeMessage::SendCommand(cmd) => {
                assert_eq!(cmd.target, "broadcast");
                assert_eq!(cmd.command, "STOP asking questions, just push");
            }
            _ => panic!("Expected SendCommand message"),
        }
    }

    #[test]
    fn parse_send_command() {
        let comments = vec![IssueComment {
            id: 2,
            issue_number: 30,
            author: "general".to_string(),
            body: "@agent-1 deploy to production".to_string(),
        }];

        let messages = GitHubClient::parse_agent_commands(&comments);
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            BridgeMessage::SendCommand(cmd) => {
                assert_eq!(cmd.target, "agent-1");
                assert_eq!(cmd.command, "deploy to production");
            }
            _ => panic!("Expected SendCommand message"),
        }
    }

    #[test]
    fn parse_status_marker() {
        let comments = vec![IssueComment {
            id: 3,
            issue_number: 30,
            author: "agent-1".to_string(),
            body: "[STATUS: agent-1 WORKING training]".to_string(),
        }];

        let messages = GitHubClient::parse_agent_commands(&comments);
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            BridgeMessage::UpdateStatus(cmd) => {
                assert_eq!(cmd.agent_id, "agent-1");
                assert_eq!(cmd.status, AgentStatus::Working);
                assert_eq!(cmd.message, "training");
            }
            _ => panic!("Expected UpdateStatus message"),
        }
    }
}

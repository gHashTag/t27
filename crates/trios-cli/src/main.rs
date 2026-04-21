//! `tri` — Trinity IGLA Needle Hunt CLI
//!
//! Main entry point for the tri CLI tool.

use anyhow::Result;
use clap::{Parser, Subcommand};

use trios_cli::{
    cmd::{
        agent::agent_dispatch,
        commit::commit,
        dash::{dash_refresh, dash_sync},
        gates::{gate_check, GateStatus},
        issue::{issue_close, issue_new},
        leaderboard::leaderboard_show,
        report::report,
        roster::roster_update,
        run::run,
        sweep::sweep,
        submit::submit,
    },
};

#[derive(Parser)]
#[command(name = "tri", version = "0.1.0", about = "Trinity IGLA CLI — Needle Hunt Automation")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run experiment and parse BPB
    Run {
        exp_id: String,
        #[arg(long, default_value_t = 1)]
        seeds: u32,
    },

    /// Parameter sweep
    Sweep {
        param: String,
        values: Vec<String>,
    },

    /// Report result to issue #143
    Report {
        agent: String,
        status: String,
        #[arg(long)]
        bpb: Option<f64>,
    },

    /// Issue management
    Issue {
        #[command(subcommand)]
        sub: IssueSub,
    },

    /// Update agent roster
    Roster {
        agent: String,
        status: String,
    },

    /// Dashboard operations
    Dash {
        #[command(subcommand)]
        sub: DashSub,
    },

    /// Check quality gates
    Gates {
        gate: String,
        #[arg(long)]
        value: Option<f64>,
    },

    /// Submit parameters
    Submit {
        #[arg(long)]
        bpb: f64,
        #[arg(long)]
        artifact: String,
    },

    /// Show leaderboard
    Leaderboard {
        #[arg(long)]
        agent: Option<String>,
    },

    /// Dispatch task to agent
    Agent {
        nato: String,
        task: String,
    },

    /// Git commit (atomic)
    Commit {
        msg: String,
    },
}

#[derive(Subcommand)]
enum IssueSub {
    New { template: String, args: Vec<String> },
    Close { num: u32, #[arg(long)] bpb: Option<f64> },
}

#[derive(Subcommand)]
enum DashSub {
    Sync,
    Refresh,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match Cli::parse().cmd {
        Cmd::Run { exp_id, seeds } => {
            run(&exp_id, seeds)?;
        }
        Cmd::Sweep { param, values } => {
            let results = sweep(&param, values)?;
            println!();
            println!("{}", results.to_markdown(&param));
        }
        Cmd::Report { agent, status, bpb } => {
            report(&agent, &status, bpb)?;
        }
        Cmd::Issue { sub } => match sub {
            IssueSub::New { template, args } => {
                issue_new(&template, &args)?;
            }
            IssueSub::Close { num, bpb } => {
                issue_close(num, bpb)?;
            }
        },
        Cmd::Roster { agent, status } => {
            roster_update(&agent, &status)?;
        }
        Cmd::Dash { sub } => match sub {
            DashSub::Sync => dash_sync()?,
            DashSub::Refresh => dash_refresh()?,
        },
        Cmd::Gates { gate, value } => {
            let status = gate_check(&gate, value)?;
            println!();
            match status {
                GateStatus::Pass => println!("✅ All gates passed"),
                GateStatus::Warn => println!("⚠️  Some gates warn"),
                GateStatus::Fail => println!("❌ Gates failed"),
                GateStatus::Unknown => println!("❓ Gates unknown"),
            }
        }
        Cmd::Submit { bpb, artifact } => {
            submit(bpb, &artifact)?;
        }
        Cmd::Leaderboard { agent } => {
            leaderboard_show(agent.as_deref())?;
        }
        Cmd::Agent { nato, task } => {
            agent_dispatch(&nato, &task)?;
        }
        Cmd::Commit { msg } => {
            commit(&msg)?;
        }
    }
    Ok(())
}

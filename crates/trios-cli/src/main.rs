use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "tri", version = "0.1.0", about = "Trinity IGLA CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Run {
        exp_id: String,
        #[arg(long, default_value_t = 1)]
        seeds: u32,
    },
    Sweep {
        param: String,
        values: Vec<String>,
    },
    Report {
        agent: String,
        status: String,
        #[arg(long)]
        bpb: Option<f64>,
    },
    Issue {
        #[command(subcommand)]
        sub: IssueCmd,
    },
    Roster {
        agent: String,
        status: String,
    },
    Dash {
        #[command(subcommand)]
        sub: DashCmd,
    },
    Gates {
        gate: String,
    },
    Submit {
        #[arg(long)]
        bpb: f64,
        #[arg(long)]
        artifact: String,
    },
    Leaderboard,
    Agent {
        nato: String,
        task: String,
    },
    Commit {
        msg: String,
    },
}

#[derive(Subcommand)]
enum IssueCmd {
    New { template: String, args: Vec<String> },
    Close { num: u32, #[arg(long)] bpb: Option<f64> },
}

#[derive(Subcommand)]
enum DashCmd {
    Sync,
    Refresh,
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Run { exp_id, seeds } => todo!("tri run {exp_id} --seeds {seeds}"),
        Cmd::Sweep { param, values } => todo!("tri sweep {param} {:?}", values),
        Cmd::Report { agent, status, bpb } => todo!("tri report {agent} {status} --bpb {:?}", bpb),
        Cmd::Issue { sub } => todo!("tri issue {:?}", sub),
        Cmd::Roster { agent, status } => todo!("tri roster {agent} {status}"),
        Cmd::Dash { sub } => todo!("tri dash {:?}", sub),
        Cmd::Gates { gate } => todo!("tri gates check {gate}"),
        Cmd::Submit { bpb, artifact } => todo!("tri submit --bpb {bpb} --artifact {artifact}"),
        Cmd::Leaderboard => todo!("tri leaderboard"),
        Cmd::Agent { nato, task } => todo!("tri agent dispatch {nato} {task}"),
        Cmd::Commit { msg } => todo!("tri commit \"{msg}\""),
    }
}

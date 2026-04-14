//! Math CLI - PSLQ, Bayes, and Compare commands
//! Bypasses pre-existing main.rs compilation issues

use anyhow::Context;
use clap::{Parser, Subcommand};

mod pslq;
mod bayes;
mod compare;

#[derive(Parser)]
#[command(name = "math-cli")]
struct Cli {
    #[command(subcommand)]
    command: MathCommands,
}

#[derive(Subcommand)]
enum MathCommands {
    /// PSLQ integer relation finding
    #[command(name = "pslq")]
    Pslq(pslq::PslqCommands),

    /// Bayes factor calculation
    #[command(name = "bayes")]
    Bayes(bayes::BayesCommands),

    /// Compare L5 anchors and formula metrics
    #[command(name = "compare")]
    Compare(compare::MathCommands),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        MathCommands::Pslq(cmd) => pslq::run_pslq_command(cmd, ".")?,
        MathCommands::Bayes(cmd) => bayes::run_bayes_command(cmd, ".")?,
        MathCommands::Compare(cmd) => compare::run_math_command(cmd, ".")?,
    }
}

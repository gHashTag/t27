// NotebookLM bridge for Trinity T27
//
// Provides CLI commands for managing NotebookLM notebooks:
// - Populate: Create notebooks from GitHub issues
// - Enrich: Add contextual sources (YouTube, podcasts, docs)
// - Dashboard: View all notebooks with enrichment status
//
// phi^2 + 1/phi^2 = 3 | TRINITY

use clap::Subcommand;
use colored::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// NotebookLM commands for managing and enriching notebooks
#[derive(Subcommand, Debug)]
pub enum NbCommands {
    /// Create notebooks from GitHub issues and populate with issue content
    Populate {
        /// Specific issue number to populate
        #[arg(long)]
        issue: Option<u32>,
        /// Populate all issues
        #[arg(long)]
        all: bool,
        /// Repository name (default: gHashTag/t27)
        #[arg(long, default_value = "gHashTag/t27")]
        repo: String,
    },

    /// Enrich notebooks with contextual content (YouTube, podcasts, docs)
    Enrich {
        /// Enrich specific issue number
        #[arg(long)]
        issue: Option<u32>,
        /// Enrich all notebooks
        #[arg(long)]
        all: bool,
        /// Force re-add sources even if previously enriched
        #[arg(long)]
        force: bool,
    },

    /// Show enrichment dashboard in browser
    Dashboard {
        /// Export dashboard data to JSON file
        #[arg(long)]
        export: Option<String>,
        /// Port for HTTP server (default: 8080)
        #[arg(long, default_value = "8080")]
        port: u16,
    },

    /// Continuous sync — Keep notebooks in sync with repo changes
    Sync {
        /// Sync specific issue
        #[arg(long)]
        issue: Option<u32>,
        /// Sync all enriched notebooks
        #[arg(long)]
        all: bool,
        /// Watch mode — continuous sync
        #[arg(long)]
        watch: bool,
        /// Sync activity.md
        #[arg(long)]
        activity: bool,
        /// Event type
        #[arg(long)]
        event: Option<String>,
        /// Event trigger
        #[arg(long)]
        trigger: Option<String>,
    },

    /// List available topics for enrichment
    ListTopics,

    /// Generate AI presentations and audio overviews (podcast-style)
    Presentations {
        /// Generate for specific issue number
        #[arg(long)]
        issue: Option<u32>,
        /// Generate for specific notebook ID
        #[arg(long)]
        notebook_id: Option<String>,
        /// Generate for all enriched notebooks
        #[arg(long)]
        all: bool,
        /// Limit number of notebooks to process (0 = all)
        #[arg(long, default_value = "0")]
        limit: String,
        /// Regenerate existing presentations
        #[arg(long)]
        regenerate: bool,
    },
}

/// Run a NotebookLM command
pub fn run_nb(command: NbCommands, root: &Path) -> anyhow::Result<()> {
    let notebooklm_dir = root.join("contrib/backend/notebooklm");

    match command {
        NbCommands::Populate { issue, all, repo } => {
            handle_populate(&notebooklm_dir, issue, all, &repo)
        }
        NbCommands::Enrich { issue, all, force } => {
            handle_enrich(&notebooklm_dir, issue, all, force)
        }
        NbCommands::Dashboard { export, port } => {
            handle_dashboard(&notebooklm_dir, export, port)
        }
        NbCommands::Sync { issue, all, watch, activity, event, trigger } => {
            handle_sync(&notebooklm_dir, issue, all, watch, activity, event, trigger)
        }
        NbCommands::ListTopics => {
            handle_list_topics(&notebooklm_dir)
        }
        NbCommands::Presentations { issue, notebook_id, all, limit, regenerate } => {
            handle_presentations(&notebooklm_dir, issue, notebook_id, all, &limit, regenerate)
        }
    }
}

fn handle_populate(
    notebooklm_dir: &Path,
    issue: Option<u32>,
    all: bool,
    repo: &str,
) -> anyhow::Result<()> {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "NotebookLM - Populate".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let populate_script = notebooklm_dir.join("populate.py");
    if !populate_script.exists() {
        println!("{} populate.py not found at {}", "❌".red(), populate_script.display());
        println!("{} Create it to populate notebooks from GitHub issues", "ℹ".cyan());
        return Ok(());
    }

    let mut cmd = Command::new("python3.10");
    // Use absolute path to avoid Python path resolution issues
    let abs_script = match std::fs::canonicalize(&populate_script) {
        Ok(p) => p,
        Err(_) => populate_script.clone(),
    };
    cmd.arg(&abs_script);
    cmd.arg("--repo").arg(repo);
    if let Some(num) = issue {
        cmd.arg("--issue").arg(num.to_string());
    } else if all {
        cmd.arg("--all");
    }
    // Don't set current_dir to avoid Python path resolution issues

    println!("{} Running populate script...", "▶".cyan());
    println!();
    let status = cmd.status()?;

    if status.success() {
        println!();
        println!("{} Populate completed successfully", "✅".green());
    } else {
        println!();
        println!("{} Populate failed with exit code: {}", "❌".red(), status);
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn handle_enrich(
    notebooklm_dir: &Path,
    issue: Option<u32>,
    all: bool,
    force: bool,
) -> anyhow::Result<()> {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "NotebookLM - Enrich".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let enrich_script = notebooklm_dir.join("enrich.py");
    if !enrich_script.exists() {
        println!("{} enrich.py not found at {}", "❌".red(), enrich_script.display());
        return Ok(());
    }

    let mut cmd = Command::new("python3.10");
    // Use absolute path to avoid Python path resolution issues
    let abs_script = match std::fs::canonicalize(&enrich_script) {
        Ok(p) => p,
        Err(_) => enrich_script.clone(),
    };
    cmd.arg(&abs_script);
    if let Some(num) = issue {
        cmd.arg("--issue").arg(num.to_string());
    } else if all {
        cmd.arg("--all");
    }
    if force {
        cmd.arg("--force");
    }
    // Don't set current_dir to avoid Python path resolution issues

    println!("{} Running enrich script...", "▶".cyan());
    println!();
    let status = cmd.status()?;

    if status.success() {
        println!();
        println!("{} Enrich completed successfully", "✅".green());
    } else {
        println!();
        println!("{} Enrich failed with exit code: {}", "❌".red(), status);
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn handle_dashboard(
    notebooklm_dir: &Path,
    _export: Option<String>,
    port: u16,
) -> anyhow::Result<()> {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "NotebookLM - Dashboard".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let dashboard_path = notebooklm_dir.join("dashboard.html");
    if !dashboard_path.exists() {
        println!("{} dashboard.html not found at {}", "❌".red(), dashboard_path.display());
        println!("{} Run enrich --export-dashboard first", "ℹ".cyan());
        return Ok(());
    }

    let url = format!("http://localhost:{}", port);
    println!("{} Starting dashboard at {}", "▶".cyan(), url.cyan().underline());
    println!("{} Press Ctrl+C to stop", "ℹ".cyan());
    println!();

    let mut cmd = Command::new("python3.10");
    cmd.arg("-m")
        .arg("http.server")
        .arg(port.to_string())
        .arg("--directory")
        .arg(notebooklm_dir);

    if let Err(e) = cmd.status() {
        println!("{} Dashboard server error: {}", "❌".red(), e);
        std::process::exit(1);
    }

    Ok(())
}

fn handle_sync(
    notebooklm_dir: &Path,
    issue: Option<u32>,
    all: bool,
    watch: bool,
    activity: bool,
    event: Option<String>,
    trigger: Option<String>,
) -> anyhow::Result<()> {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "NotebookLM - Continuous Sync".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let sync_script = notebooklm_dir.join("sync.py");
    if !sync_script.exists() {
        println!("{} sync.py not found at {}", "❌".red(), sync_script.display());
        return Ok(());
    }

    let mut cmd = Command::new("python3.10");
    // Use absolute path explicitly to avoid relative resolution issues
    let abs_script = match std::fs::canonicalize(&sync_script) {
        Ok(p) => p,
        Err(_) => sync_script.clone(),
    };
    cmd.arg(&abs_script);
    // Don't set current_dir to avoid Python path resolution issues

    if let Some(num) = issue {
        cmd.arg("--issue").arg(num.to_string());
    }
    if all {
        cmd.arg("--all");
    }
    if watch {
        cmd.arg("--auto");
        println!("{} Starting watch mode (Ctrl+C to stop)...", "▶".cyan());
    }
    if activity {
        cmd.arg("--activity");
    }
    if let Some(evt) = event {
        cmd.arg("--event").arg(&evt);
    }
    if let Some(trig) = trigger {
        cmd.arg("--trigger").arg(&trig);
    }

    cmd.current_dir(notebooklm_dir);

    let status = cmd.status()?;

    if status.success() {
        println!();
        println!("{} Sync completed", "✅".green());
    } else {
        println!();
        println!("{} Sync failed with exit code: {}", "❌".red(), status);
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn handle_presentations(
    notebooklm_dir: &Path,
    issue: Option<u32>,
    notebook_id: Option<String>,
    all: bool,
    limit: &str,
    regenerate: bool,
) -> anyhow::Result<()> {
    println!("{}", "═════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "NotebookLM - Presentations".bright_yellow().bold());
    println!("{}", "═════════════════════════════════════════".bright_yellow());
    println!();

    let presentations_script = notebooklm_dir.join("presentations.py");
    if !presentations_script.exists() {
        println!("{} presentations.py not found at {}", "❌".red(), presentations_script.display());
        println!("{} Create it to generate presentations", "ℹ".cyan());
        return Ok(());
    }

    let mut cmd = Command::new("python3.10");
    // Use absolute path to avoid Python path resolution issues
    let abs_script = match std::fs::canonicalize(&presentations_script) {
        Ok(p) => p,
        Err(_) => presentations_script.clone(),
    };
    cmd.arg(&abs_script);

    if let Some(num) = issue {
        cmd.arg("--issue").arg(num.to_string());
    } else if let Some(id) = notebook_id {
        cmd.arg("--notebook-id").arg(&id);
    } else if all {
        cmd.arg("--all");
        cmd.arg("--limit").arg(limit);
    }

    if regenerate {
        cmd.arg("--regenerate");
    }

    // Don't set current_dir to avoid Python path resolution issues

    println!("{} Generating presentations and audio overviews...", "▶".cyan());
    println!();
    let status = cmd.status()?;

    if status.success() {
        println!();
        println!("{} Presentation generation completed", "✅".green());
    } else {
        println!();
        println!("{} Presentation generation failed with exit code: {}", "❌".red(), status);
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn handle_list_topics(notebooklm_dir: &Path) -> anyhow::Result<()> {
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "NotebookLM - Available Topics".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════".bright_yellow());
    println!();

    let enrich_script = notebooklm_dir.join("enrich.py");
    if !enrich_script.exists() {
        println!("{} enrich.py not found at {}", "❌".red(), enrich_script.display());
        return Ok(());
    }

    let mut cmd = Command::new("python3.10");
    cmd.arg(&enrich_script).arg("--list-topics");

    let status = cmd.status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

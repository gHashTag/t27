// Enrichment module
// Ring 090 - Fallback for blocked YouTube URL uploads
// phi^2 + 1/phi^2 = 3 | TRINITY

use anyhow::Result;
use colored::*;

pub mod youtube_transcript;
pub mod audio_overview;

/// Run enrich command (Python script wrapper)
pub fn run_enrich(
    notebook: Option<String>,
    all: bool,
    force: bool,
    token: String,
    lang: String,
) -> Result<()> {
    let notebooklm_dir = std::env::current_dir()?.join("contrib/backend/notebooklm");
    let enrich_script = notebooklm_dir.join("enrich.py");

    if !enrich_script.exists() {
        eprintln!("{} enrich.py not found at {}", "❌".red(), enrich_script.display());
        eprintln!("{} Please ensure contrib/backend/notebooklm/enrich.py exists", "ℹ".cyan());
        return Err(anyhow::anyhow!("enrich.py not found"));
    }

    println!("{}", "═════════════════════════════════".bright_yellow());
    println!("  {} {}", "Ϯ".bold(), "NotebookLM - Transcript Enrichment".bright_yellow().bold());
    println!("{}", "═════════════════════════════════════════".bright_yellow());
    println!();

    // Set API token via environment variable
    std::env::set_var("NOTEBOOKLM_TOKEN", &token);

    // Add language parameter if specified (otherwise default to Python script behavior)
    if lang != "both" {
        // Note: Python enrich.py doesn't support --lang yet
        // This is for future compatibility when Python script is updated
        eprintln!("{} Language parameter '{}' requires audio command. Use 't27c audio' instead.", "⚠".yellow(), lang);
    }

    let mut cmd = std::process::Command::new("python3.10");
    cmd.arg(&enrich_script);

    if all {
        cmd.arg("--all");
    } else if let Some(nb_id) = notebook {
        cmd.arg("--issue");
        cmd.arg(&nb_id);
    }
    if force {
        cmd.arg("--force");
    }

    println!("{} Running: {}",
        "▶".cyan(),
        format!("python3.10 {}", enrich_script.to_string_lossy())
    );

    let status = cmd.status()?;

    if status.success() {
        println!();
        println!("{} Enrichment completed successfully", "✅".green());
    } else {
        println!();
        println!("{} Enrichment failed with exit code: {}", "⚠".yellow(), status);
    }

    Ok(())
}

/// Run audio overview generation
pub fn run_audio(
    notebook: Option<String>,
    all: bool,
    dry_run: bool,
    _bilingual: bool,
    workers: usize,
    token: String,
    project: Option<String>,
    location: Option<String>,
    region: Option<String>,
) -> Result<()> {
    let region_val = region.unwrap_or_else(|| "us".to_string());
    let location_val = location.unwrap_or_else(|| "global".to_string());
    let project_val = project.unwrap_or_else(|| String::new());
    let base_url = format!(
        "https://{}-discoveryengine.googleapis.com/v1alpha/projects/{}/locations/{}",
        region_val,
        project_val,
        location_val
    );

    // Convert Option<String> and bool to &[String]
    let notebooks = if all {
        println!("{} Fetching notebook list...", "ℹ".cyan());
        let all_notebooks = audio_overview::list_all_notebooks(&base_url, &token)?;
        // Filter to only notebooks with sources
        let with_sources: Vec<String> = all_notebooks
            .iter()
            .filter(|nb| nb.has_sources())
            .map(|nb| nb.id().to_string())
            .collect();
        println!("{} Found {} notebooks with sources (skipping {} without)",
            "ℹ".cyan(),
            with_sources.len(),
            all_notebooks.len() - with_sources.len(),
        );
        with_sources
    } else if let Some(nb) = notebook {
        vec![nb]
    } else {
        return Err(anyhow::anyhow!("Either --notebook or --all must be specified"));
    };

    // Dry run mode - just list, don't generate
    if dry_run {
        println!();
        println!("{}", "═══════════════════════════════════════".bright_yellow());
        println!("  {} {}", "🔊".bold(), "Audio Overview Generation".bright_yellow().bold());
        println!("{}", "═══════════════════════════════════════".bright_yellow());
        println!();
        println!("{} DRY RUN MODE — No API calls will be made", "⚠".yellow());
        println!("{} Scanning notebooks with sources...", "ℹ".cyan());
        println!();
        for nb in &notebooks {
            println!("  {} {}", "→".cyan(), nb);
        }
        println!();
        println!("{} Notebooks with sources: {}", "✓".green(), notebooks.len());
        println!();
        return Ok(());
    }

    let _report = audio_overview::generate_all(&notebooks, workers, token);
    Ok(())
}

//! `tri elab` — classify a compiler's error output before you quote a number
//! from it.
//!
//! This command exists because of a specific failure. `tools/check_elab_ratchet.py`
//! counted every stderr line containing the substring `" error"`. iverilog closes
//! each failing file with `N error(s) during elaboration.` — a TOTAL, which
//! matched. One phantom per failing module: 25 of a reported 186, and the number
//! reached commit messages, an issue comment and a status page before anyone
//! looked at what it was made of.
//!
//! Counting lines is not classifying. What caught it was one pass that printed
//! the distribution BY MESSAGE SHAPE and read every row. That pass was typed by
//! hand, from memory, under time pressure — which is exactly the kind of step
//! that does not get repeated, so it becomes a command.
//!
//! Two verbs:
//!
//!   * `classify` — the distribution of every elaboration diagnostic by shape,
//!     with iverilog's own summary lines separated out rather than counted.
//!     Run this BEFORE quoting a total anywhere a human will read it.
//!   * `secondary` — how many diagnostics sit on the same source line as another
//!     diagnostic above them. A condition-expression error under an unbound
//!     identifier is not independent work: it disappears with its cause. In this
//!     corpus 57 of 58 were secondary, so a third of the "remaining errors" were
//!     never remaining at all.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum ElabCmd {
    /// Distribution of elaboration diagnostics by message shape.
    Classify {
        /// Only this module stem (default: every generated module).
        #[arg(long)]
        module: Option<String>,
        /// Also print the distinct unbound identifier names.
        #[arg(long)]
        names: bool,
    },
    /// How many diagnostics are secondary -- same source line as one above.
    Secondary,
}

pub fn run(cmd: &ElabCmd) -> Result<()> {
    match cmd {
        ElabCmd::Classify { module, names } => classify(module.as_deref(), *names),
        ElabCmd::Secondary => secondary(),
    }
}

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("git rev-parse failed")?;
    if !out.status.success() {
        bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

/// Every `file:line: error: message` diagnostic iverilog emits for the
/// generated set, as (file, line, message).
fn diagnostics(root: &Path, only: Option<&str>) -> Result<Vec<(String, u32, String)>> {
    let gen = root.join("build/fpga/generated");
    if !gen.is_dir() {
        bail!(
            "no generated Verilog at {} -- run `t27c fpga-build --smoke` first",
            gen.display()
        );
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&gen)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "v").unwrap_or(false))
        .collect();
    files.sort();

    let mut out = Vec::new();
    for f in files {
        let stem = f.file_stem().unwrap_or_default().to_string_lossy().to_string();
        if let Some(m) = only {
            if stem != m {
                continue;
            }
        }
        let proc = Command::new("iverilog")
            .args(["-g2012", "-DSIMULATION", "-o", "/dev/null"])
            .arg(&f)
            .output()
            .context("iverilog not found on PATH")?;
        for ln in String::from_utf8_lossy(&proc.stderr).lines() {
            if !ln.contains(" error") {
                continue;
            }
            // `file:line: error: message` -- and the summary line, which has
            // neither a file nor a line and is the whole reason this exists.
            let mut parts = ln.splitn(3, ':');
            let (a, b, rest) = (parts.next(), parts.next(), parts.next());
            match (a, b, rest) {
                (Some(_), Some(l), Some(r)) if l.trim().parse::<u32>().is_ok() => {
                    out.push((
                        stem.clone(),
                        l.trim().parse().unwrap(),
                        r.trim().trim_start_matches("error:").trim().to_string(),
                    ));
                }
                _ => out.push((stem.clone(), 0, ln.trim().to_string())),
            }
        }
    }
    Ok(out)
}

/// Collapse a message to its SHAPE: quoted names and numbers become
/// placeholders, so `Unable to bind wire/reg/memory \`a_name'` and the same for
/// `b_name` land in one row instead of two.
fn shape(msg: &str) -> String {
    let mut s = String::with_capacity(msg.len());
    let mut in_quote = false;
    let mut prev_digit = false;
    for c in msg.chars() {
        // iverilog opens with ` and closes with ' -- but not always ONE of
        // each. `Enable of unknown task ``mac_multiply''.` doubles both, and a
        // first version of this function that toggled on either character read
        // the second backtick as a CLOSE, printed the name, and split one row
        // into three. Enter on `, leave on ', ignore the extras.
        if c == '`' {
            if !in_quote {
                s.push_str("`X'");
                in_quote = true;
            }
            continue;
        }
        if c == '\'' {
            in_quote = false;
            continue;
        }
        if in_quote {
            continue;
        }
        if c.is_ascii_digit() {
            if !prev_digit {
                s.push('N');
            }
            prev_digit = true;
            continue;
        }
        prev_digit = false;
        s.push(c);
    }
    s.trim().to_string()
}

fn is_summary(msg: &str) -> bool {
    msg.contains("error(s) during elaboration")
}

fn classify(only: Option<&str>, show_names: bool) -> Result<()> {
    let root = repo_root()?;
    let diags = diagnostics(&root, only)?;

    let (summaries, real): (Vec<_>, Vec<_>) =
        diags.iter().partition(|(_, _, m)| is_summary(m));

    let mut by_shape: BTreeMap<String, usize> = BTreeMap::new();
    for (_, _, m) in &real {
        *by_shape.entry(shape(m)).or_default() += 1;
    }
    let mut rows: Vec<_> = by_shape.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    println!("real diagnostics: {}", real.len());
    println!(
        "summary lines NOT counted: {}  (iverilog prints one per failing file;",
        summaries.len()
    );
    println!("  counting it inflates the total by exactly the number of failing files)");
    println!();
    for (sh, n) in &rows {
        println!("{:>5}  {}", n, sh);
    }

    if show_names {
        let mut names: BTreeMap<String, usize> = BTreeMap::new();
        for (_, _, m) in &real {
            if !m.contains("Unable to bind") {
                continue;
            }
            // Same doubled-quote hazard as `shape`: trim any repeat of the
            // delimiter rather than assuming exactly one.
            if let Some(rest) = m.split('`').nth(1) {
                let name = rest
                    .trim_start_matches('`')
                    .split('\'')
                    .next()
                    .unwrap_or_default();
                if !name.is_empty() {
                    *names.entry(name.to_string()).or_default() += 1;
                }
            }
        }
        println!();
        println!("unbound identifiers: {} distinct names", names.len());
        let mut ns: Vec<_> = names.into_iter().collect();
        ns.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for (n, c) in ns {
            println!("  {:>3}x  {}", c, n);
        }
    }

    println!();
    println!("A row you did not mean to count is the whole point of this command.");
    Ok(())
}

fn secondary() -> Result<()> {
    let root = repo_root()?;
    let diags = diagnostics(&root, None)?;
    let real: Vec<_> = diags
        .iter()
        .filter(|(_, l, m)| *l > 0 && !is_summary(m))
        .collect();

    // A line that already carries a "cannot resolve this name" diagnostic
    // explains every later diagnostic on the SAME line.
    let mut causes = std::collections::HashSet::new();
    for (f, l, m) in &real {
        if m.contains("Unable to bind") || m.contains("needs an array index") {
            causes.insert((f.clone(), *l));
        }
    }

    let mut derived = 0usize;
    let mut independent: Vec<&(String, u32, String)> = Vec::new();
    for d in &real {
        let (f, l, m) = *d;
        let is_cause = m.contains("Unable to bind") || m.contains("needs an array index");
        if !is_cause && causes.contains(&(f.clone(), *l)) {
            derived += 1;
        } else if !is_cause {
            independent.push(d);
        }
    }

    println!("diagnostics with a source line: {}", real.len());
    println!("  causes (unbound / array-index):  {}", causes.len());
    println!("  SECONDARY -- same line as a cause: {}", derived);
    println!("  independent, needing their own explanation: {}", independent.len());
    if !independent.is_empty() {
        println!();
        println!("The independent ones, which is where the real work is:");
        for (f, l, m) in independent.iter().take(40) {
            println!("  {}:{}  {}", f, l, m);
        }
        if independent.len() > 40 {
            println!("  ... and {} more", independent.len() - 40);
        }
    }
    println!();
    println!("Secondary diagnostics disappear with their cause. Counting them as");
    println!("remaining work overstates what is left -- here by a factor of ~1.5.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_collapses_a_quoted_name() {
        assert_eq!(
            shape("Unable to bind wire/reg/memory `a_name' in `ClockDomain.f.f_body'"),
            "Unable to bind wire/reg/memory `X' in `X'"
        );
    }

    #[test]
    fn shape_survives_iverilogs_doubled_quotes() {
        // The real reason this test exists. iverilog writes this ONE diagnostic
        // with doubled delimiters:
        //
        //     Enable of unknown task ``mac_multiply''.
        //
        // A first version toggled on either quote character, so the second
        // backtick read as a CLOSE, the name was emitted into the shape, and one
        // row became three -- in the very command whose job is to stop a
        // distribution from lying about itself.
        let one = shape("Enable of unknown task ``mac_multiply''.");
        let two = shape("Enable of unknown task ``mac_cycle''.");
        assert_eq!(one, "Enable of unknown task `X'.");
        assert_eq!(one, two, "two names must collapse to one shape");
    }

    #[test]
    fn shape_collapses_digits() {
        assert_eq!(
            shape("the number of indices (2) is greater than the number of dimensions (1)."),
            "the number of indices (N) is greater than the number of dimensions (N)."
        );
    }

    #[test]
    fn summary_line_is_recognised_as_a_total() {
        assert!(is_summary("25 error(s) during elaboration."));
        assert!(!is_summary(
            "Unable to bind wire/reg/memory `a_name' in `M.f.f_body'"
        ));
    }
}

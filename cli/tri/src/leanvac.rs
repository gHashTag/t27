//! `tri lean vacuous` -- completeness theorems whose model is empty.
//!
//! `proofs/lean4/.../Completeness.lean` holds 250 hand-transcribed models, one per
//! spec, each with a theorem asserting the module is Icarus-lowerable. A model with
//! `functions := []` makes its theorem true by construction: `native_decide` on an
//! empty structure proves something, and that something is not about the spec.
//!
//! 114 of the 250 are empty. The ledger's `max_vacuous` ratchet, which exists to stop
//! that number growing, counts **44** -- because it only sees models that ALSO disagree
//! with the Rust classifier. The other 70 are vacuous and invisible to it.
//!
//! A ratchet measuring a subset of its own subject is worse than no ratchet, because
//! the number it reports looks like the number you care about.

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum LeanCmd {
    /// Completeness theorems whose model has no functions.
    Vacuous {
        /// Print every name, not just the count and the gap.
        #[arg(long)]
        all: bool,
    },
    /// Proof files the build root does not reach, and so nothing compiles.
    Reach {
        /// Also list what the root does reach.
        #[arg(long)]
        all: bool,
    },
}

/// One model as the file states it.
pub struct Model {
    pub name: String,
    pub empty_fns: bool,
    pub empty_env: bool,
}

/// Every `<name>_module` in the file, with whether its function list and its Env are
/// empty.
///
/// Text-scanned rather than parsed, and that is a limit worth stating: this reads what
/// the file SAYS, and only a Lean build can say what it MEANS. No workflow in this
/// repository builds these proofs (#2747), so a text scan is the strongest instrument
/// available here, not the weakest one chosen.
pub fn models_in(src: &str) -> Vec<Model> {
    let mut envs: Vec<(String, bool)> = Vec::new();
    let mut out: Vec<Model> = Vec::new();
    let mut cur: Option<(String, bool, bool)> = None; // name, is_module, saw_empty
    let mut empty_env_names: Vec<String> = Vec::new();
    for line in src.split('\n') {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("def ") {
            // flush
            if let Some((name, is_module, saw)) = cur.take() {
                if is_module {
                    out.push(Model {
                        empty_env: empty_env_names.contains(&name),
                        name,
                        empty_fns: saw,
                    });
                } else if saw {
                    empty_env_names.push(name);
                }
            }
            if let Some(n) = rest.strip_suffix(" : Env := {") {
                cur = Some((n.trim_end_matches("_env").to_string(), false, false));
            } else if let Some(n) = rest.strip_suffix(" : Module := {") {
                cur = Some((n.trim_end_matches("_module").to_string(), true, false));
            }
            continue;
        }
        if let Some((_, is_module, saw)) = cur.as_mut() {
            if (*is_module && t.starts_with("functions := []"))
                || (!*is_module && t.starts_with("structs := []"))
            {
                *saw = true;
            }
        }
    }
    if let Some((name, is_module, saw)) = cur.take() {
        if is_module {
            out.push(Model {
                empty_env: empty_env_names.contains(&name),
                name,
                empty_fns: saw,
            });
        }
    }
    let _ = &mut envs;
    out
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

pub fn run(cmd: &LeanCmd) -> Result<()> {
    let all = match cmd {
        LeanCmd::Reach { all } => return crate::leanreach::run(*all),
        LeanCmd::Vacuous { all } => all,
    };
    let root = repo_root()?;
    let lean = root.join("proofs/lean4/Trinity/IcarusLowerable/Completeness.lean");
    let src = std::fs::read_to_string(&lean)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {}", lean.display(), e))?;
    let models = models_in(&src);
    if models.is_empty() {
        anyhow::bail!(
            "read {} and found no `<name>_module : Module := {{` -- the file's shape \
             changed, and a count of zero here would be the scanner, not the proofs",
            lean.display()
        );
    }
    let vacuous: Vec<&Model> = models.iter().filter(|m| m.empty_fns).collect();
    let both: usize = vacuous.iter().filter(|m| m.empty_env).count();

    let ledger = root.join("docs/reports/lean_completeness_mismatches.json");
    let marked: usize = std::fs::read_to_string(&ledger)
        .ok()
        .and_then(|r| serde_json::from_str::<serde_json::Value>(&r).ok())
        .and_then(|v| {
            v.get("entries")?.as_object().map(|o| {
                o.values()
                    .filter(|e| e.get("model_empty").and_then(|b| b.as_bool()) == Some(true))
                    .count()
            })
        })
        .unwrap_or(0);

    println!("VACUOUS COMPLETENESS THEOREMS");
    println!();
    println!("  models in the file            {}", models.len());
    println!("  with `functions := []`        {}", vacuous.len());
    println!("  ...and an empty Env too       {}", both);
    println!("  counted by max_vacuous        {}", marked);
    println!(
        "  vacuous and INVISIBLE to it   {}",
        vacuous.len().saturating_sub(marked)
    );
    println!();
    if *all {
        for m in &vacuous {
            println!("    {}{}", m.name, if m.empty_env { "  (env empty too)" } else { "" });
        }
        println!();
    }
    println!(
        "`max_vacuous` counts only models that ALSO disagree with the Rust classifier.\n\
         A theorem about an empty module says nothing about its spec whether or not the\n\
         classifier happens to disagree, so the ratchet measures a subset of its own\n\
         subject -- and the number it reports looks like the number you care about."
    );
    println!();
    println!(
        "Read from the file's text. Only a Lean build can say what these theorems MEAN,\n\
         and no workflow in this repository builds them (#2747)."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
def a_env : Env := {
  structs := [],
  enums := []
}

def a_module : Module := {
  name := \"a\",
  functions := [],
  tests := []
}

def b_env : Env := {
  structs := [(\"S\", [])],
  enums := []
}

def b_module : Module := {
  name := \"b\",
  functions := [{ name := \"f\" }],
  tests := []
}
";

    #[test]
    fn an_empty_function_list_is_vacuous_and_a_populated_one_is_not() {
        let ms = models_in(SAMPLE);
        assert_eq!(ms.len(), 2, "two modules");
        assert!(ms[0].empty_fns, "a has functions := []");
        assert!(!ms[1].empty_fns, "b lists a function");
    }

    #[test]
    fn an_empty_env_is_reported_separately_from_an_empty_module() {
        let ms = models_in(SAMPLE);
        assert!(ms[0].empty_env, "a's Env has structs := []");
        assert!(!ms[1].empty_env, "b's Env declares a struct");
    }

    #[test]
    fn a_file_with_no_modules_yields_nothing_rather_than_a_wrong_zero() {
        // The command turns this into a refusal, because a zero from a scanner
        // that matched nothing is indistinguishable from a clean file.
        assert!(models_in("-- just a comment\n").is_empty());
    }

    #[test]
    fn the_env_belonging_to_a_module_is_the_one_with_its_name() {
        // `a_env` empty, `b_env` not: the pairing is by name, not by position,
        // so a file that declares them out of order still pairs correctly.
        let reordered = "\
def b_env : Env := {
  structs := [(\"S\", [])]
}

def a_env : Env := {
  structs := []
}

def a_module : Module := {
  functions := []
}
";
        let ms = models_in(reordered);
        assert_eq!(ms.len(), 1);
        assert!(ms[0].empty_env);
    }
}

//! `tri commit` — Atomic git commit via libgit2
//!
//! Usage:
//!   tri commit "Add FOXTROT BigramHash(729) baseline"

use anyhow::{Context, Result};

use git2::{
    Repository, Signature, Time, Oid,
    Status,
};

/// Create atomic git commit with staged changes
pub fn commit(msg: &str) -> Result<Oid> {
    println!("📝 Committing: {}", msg);

    let repo = Repository::open_from_env()
        .context("Failed to open git repo")?;

    // Check if there are staged changes
    let mut statuses = repo.statuses(None)
        .context("Failed to get git status")?;

    if statuses.is_empty() {
        anyhow::bail!("No staged changes to commit. Run `git add` first.");
    }

    // Get head commit for parent
    let head = repo.head()
        .context("Failed to get HEAD")?;

    let parent_commit = head.peel_to_commit()
        .context("Failed to get HEAD commit")?;

    // Create signature
    let sig = Signature::now("Trios CLI", "trios@localhost")
        .context("Failed to create signature")?;

    // Create tree from index
    let tree_id = repo.index()
        .and_then(|mut index| index.write_tree())
        .context("Failed to write tree")?;

    let tree = repo.find_tree(tree_id)
        .context("Failed to find tree")?;

    // Create commit
    let oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        msg,
        &tree,
        &[&parent_commit],
    )
    .context("Failed to create commit")?;

    println!("✓ Committed: {}", oid);

    Ok(oid)
}

/// Stage and commit in one operation
pub fn commit_add(paths: &[&str], msg: &str) -> Result<Oid> {
    println!("📝 Staging and committing: {}", msg);

    let repo = Repository::open_from_env()
        .context("Failed to open git repo")?;

    let mut index = repo.index()
        .context("Failed to get index")?;

    // Stage paths
    for path in paths {
        index.add_path(std::path::Path::new(path))
            .with_context(|| format!("Failed to stage {}", path))?;
    }

    index.write()
        .context("Failed to write index")?;

    // Now commit
    commit(msg)
}

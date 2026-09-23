use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::to_string_pretty;
use thiserror::Error;

use crate::normalise::NormalisedRecord;

/// Writes each record to data/<jurisdiction>/<council-id>/<record-type>/
/// <source-id>.json, overwriting whatever was there before. A source's
/// `modified`/`deleted` marker becoming a new commit, rather than a
/// separate correction mechanism, is what makes this a plain overwrite
/// safe: git keeps the history, this function only needs to keep the
/// current state.
pub fn write_records(root: &Path, jurisdiction: &str, records: &[NormalisedRecord]) -> Result<usize, StoreError> {
    let mut written = 0;

    for record in records {
        let record_type = to_string_pretty(&record.record_type)
            .map_err(|source| StoreError::Serialise { source })?
            .trim_matches('"')
            .to_string();

        let path = root
            .join(jurisdiction)
            .join(&record.council_id)
            .join(record_type)
            .join(format!("{}.json", slug(&record.source_id)));

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| StoreError::Io { path: parent.to_path_buf(), source })?;
        }

        let contents = to_string_pretty(record).map_err(|source| StoreError::Serialise { source })?;
        std::fs::write(&path, contents).map_err(|source| StoreError::Io { path: path.clone(), source })?;
        written += 1;
    }

    Ok(written)
}

/// Stages data/ and commits, if there is anything to commit. Returns None
/// when the working tree already matched what was just written (a pull that
/// found no changes), rather than creating an empty commit.
pub fn commit(root: &Path, message: &str) -> Result<Option<String>, StoreError> {
    run_git(root, &["add", "data"])?;

    let status = run_git(root, &["status", "--porcelain", "--", "data"])?;
    if status.trim().is_empty() {
        return Ok(None);
    }

    run_git(root, &["commit", "--quiet", "-m", message])?;
    let hash = run_git(root, &["rev-parse", "--short", "HEAD"])?;
    Ok(Some(hash.trim().to_string()))
}

fn run_git(root: &Path, args: &[&str]) -> Result<String, StoreError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|source| StoreError::GitSpawn { source })?;

    if !output.status.success() {
        return Err(StoreError::GitFailed {
            args: args.iter().map(|arg| arg.to_string()).collect(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Source ids are OParl object URLs (e.g.
/// "https://oparl.example.de/bodies/1/meetings/42"). Keeping the full path,
/// character-substituted rather than hashed, keeps filenames traceable back
/// to their source without needing a lookup table.
fn slug(source_id: &str) -> String {
    source_id
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '_' })
        .collect()
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("failed to write {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("failed to serialise a record: {source}")]
    Serialise { source: serde_json::Error },
    #[error("failed to run git: {source}")]
    GitSpawn { source: std::io::Error },
    #[error("git {args:?} failed: {stderr}")]
    GitFailed { args: Vec<String>, stderr: String },
}

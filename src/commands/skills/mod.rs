//! Installs the embedded skill bundle into the current working directory,
//! points agents at the `rex codebase` and `rex smells` commands through
//! `AGENTS.md`, keeps the installed files out of git through `.gitignore`,
//! and sets up pre-commit hooks (gitleaks, trivy, cargo fmt, clippy, test).

mod agents_md;
mod file_change;
mod gitignore;
mod pre_commit;

use std::fs;
use std::path::Path;

use super::stdout::print_line_to_stdout;
use crate::bundle;
use crate::error::RexError;
use file_change::FileChange;
use pre_commit::{ConfigWrite, HookInstall};

/// Which agent tool the skill bundle is installed for. Each vendor gets the
/// same content tree under its own dot-directory.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Vendor {
    Claude,
    Codex,
    Agents,
}

impl Vendor {
    /// The dot-directory this vendor installs into, relative to the working directory.
    #[must_use]
    pub fn dir_name(self) -> &'static str {
        match self {
            Self::Claude => ".claude",
            Self::Codex => ".codex",
            Self::Agents => ".agents",
        }
    }
}

/// Write every file from the embedded skill bundle into `cwd`, under the
/// given vendor's dot-directory. A file already present on disk is left
/// untouched; only missing files are written.
///
/// Then make sure `cwd/AGENTS.md` has the rex sections, `cwd/.gitignore`
/// ignores the installed skill files, and `cwd/.pre-commit-config.yaml` is
/// there, and run `pre-commit install` when the tool is on PATH. Each file is
/// added once and an existing file is never replaced.
///
/// # Errors
/// [`RexError::Io`] if a parent directory or file cannot be created or written,
/// or if `AGENTS.md`, `.gitignore` or the pre-commit config cannot be read or
/// written; [`RexError::PreCommitInstall`] if `pre-commit install` fails;
/// [`RexError::Stdout`] if a progress line cannot be written to stdout.
pub fn run(cwd: &Path, vendor: Vendor) -> Result<(), RexError> {
    let InstallCounts { written, skipped } = install_bundle(&cwd.join(vendor.dir_name()))?;

    print_line_to_stdout(&format!("rex skills: {written} written, {skipped} skipped"))?;

    report_file_change(
        agents_md::AGENTS_FILENAME,
        agents_md::ensure_rex_sections(cwd)?,
    )?;
    report_file_change(
        gitignore::GITIGNORE_FILENAME,
        gitignore::ensure_ignore_block(cwd, vendor)?,
    )?;

    let config = pre_commit::CONFIG_FILENAME;
    match pre_commit::ensure_config(cwd)? {
        ConfigWrite::Created => print_line_to_stdout(&format!("rex skills: {config} created"))?,
        ConfigWrite::AlreadyWritten => {}
        ConfigWrite::SkippedExists => {
            print_line_to_stdout(&format!("rex skills: {config} skipped (already exists)"))?;
        }
    }

    let install = pre_commit::INSTALL_COMMAND;
    match pre_commit::install_hooks(cwd)? {
        HookInstall::Installed => print_line_to_stdout("rex skills: pre-commit hooks installed")?,
        HookInstall::SkippedNoGitDir => {
            print_line_to_stdout("rex skills: pre-commit hooks skipped (no .git directory)")?;
        }
        HookInstall::SkippedNoTool => print_line_to_stdout(&format!(
            "rex skills: pre-commit hooks skipped (pre-commit not found; install it, then run: {install})"
        ))?,
    }
    Ok(())
}

/// How many bundle files an install wrote and how many it left alone.
#[derive(Default)]
struct InstallCounts {
    written: u32,
    skipped: u32,
}

/// Write each bundle file that is not on disk yet under `target_root`.
fn install_bundle(target_root: &Path) -> Result<InstallCounts, RexError> {
    let mut counts = InstallCounts::default();

    for (rel_path, contents) in bundle::walk() {
        let dest = target_root.join(&rel_path);
        if dest.exists() {
            counts.skipped += 1;
            continue;
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|source| RexError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&dest, contents).map_err(|source| RexError::Io {
            path: dest.clone(),
            source,
        })?;
        counts.written += 1;
    }

    Ok(counts)
}

fn report_file_change(filename: &str, change: FileChange) -> Result<(), RexError> {
    match change {
        FileChange::Created => print_line_to_stdout(&format!("rex skills: {filename} created")),
        FileChange::Updated => print_line_to_stdout(&format!("rex skills: {filename} updated")),
        FileChange::Unchanged => Ok(()),
    }
}

//! Installs the embedded skill bundle into the current working directory,
//! points agents at the `rex codebase` and `rex smells` commands through
//! `AGENTS.md`, keeps the installed files out of git through `.gitignore`,
//! and installs a trivy pre-commit hook.

mod agents_md;
mod file_change;
mod gitignore;
mod pre_commit;

use std::fs;
use std::path::Path;

use crate::bundle;
use crate::error::RexError;
use file_change::FileChange;
use pre_commit::HookInstall;

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
/// ignores the installed skill files, and `cwd/.git/hooks/pre-commit` runs
/// trivy. Each of those is added once and an existing file is never replaced.
///
/// # Errors
/// [`RexError::Io`] if a parent directory or file cannot be created or written,
/// or if `AGENTS.md` or `.gitignore` cannot be read or written.
pub fn run(cwd: &Path, vendor: Vendor) -> Result<(), RexError> {
    let target_root = cwd.join(vendor.dir_name());
    let mut written = 0u32;
    let mut skipped = 0u32;

    for (rel_path, contents) in bundle::walk() {
        let dest = target_root.join(&rel_path);
        if dest.exists() {
            skipped += 1;
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
        written += 1;
    }

    println!("rex skills: {written} written, {skipped} skipped");

    report_file_change(
        agents_md::AGENTS_FILENAME,
        agents_md::ensure_rex_sections(cwd)?,
    );
    report_file_change(
        gitignore::GITIGNORE_FILENAME,
        gitignore::ensure_ignore_block(cwd, vendor)?,
    );

    let hook = pre_commit::HOOK_PATH;
    match pre_commit::install_hook(cwd)? {
        HookInstall::Installed => println!("rex skills: {hook} installed"),
        HookInstall::AlreadyInstalled => {}
        HookInstall::SkippedExists => println!("rex skills: {hook} skipped (already exists)"),
        HookInstall::SkippedNoGitDir => println!("rex skills: {hook} skipped (no .git directory)"),
    }
    Ok(())
}

fn report_file_change(filename: &str, change: FileChange) {
    match change {
        FileChange::Created => println!("rex skills: {filename} created"),
        FileChange::Updated => println!("rex skills: {filename} updated"),
        FileChange::Unchanged => {}
    }
}

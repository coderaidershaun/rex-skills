//! Installs the embedded skill bundle into the current working directory,
//! points agents at the `rex codebase` and `rex smells` commands through
//! `AGENTS.md`, keeps the installed files out of git through `.gitignore`,
//! and sets up pre-commit hooks (gitleaks, trivy, cargo fmt, clippy, test).

mod agents_md;
mod file_change;
mod gitignore;
mod pre_commit;
mod retarget;

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

/// What `rex skills` installs and whether it replaces files it wrote before.
#[derive(clap::Args, Clone, Copy, Debug)]
pub struct SkillsOptions {
    /// Which agent tool to install the bundle for.
    #[arg(long, value_enum, default_value_t = Vendor::Claude)]
    pub vendor: Vendor,

    /// Overwrite installed skill files that differ from this version of rex.
    #[arg(long = "force")]
    pub is_forced: bool,
}

/// Write every file from the embedded skill bundle into `cwd`, under the
/// vendor's dot-directory. A file already present on disk is left untouched
/// and counted, unless it differs from the bundle and `is_forced` is set, in
/// which case it is overwritten.
///
/// Then make sure `cwd/AGENTS.md` has the rex sections, `cwd/.gitignore`
/// ignores the installed skill files, and `cwd/.pre-commit-config.yaml` is
/// there, and run `pre-commit install` when the tool is on PATH. Each file is
/// added once and an existing file is never replaced, with or without
/// `is_forced`.
///
/// # Errors
/// [`RexError::Io`] if a parent directory or file cannot be created or written,
/// or if `AGENTS.md`, `.gitignore` or the pre-commit config cannot be read or
/// written; [`RexError::PreCommitInstall`] if `pre-commit install` fails;
/// [`RexError::Stdout`] if a progress line cannot be written to stdout.
pub fn run(cwd: &Path, options: SkillsOptions) -> Result<(), RexError> {
    let SkillsOptions { vendor, is_forced } = options;
    let counts = install_bundle(&cwd.join(vendor.dir_name()), vendor, is_forced)?;

    print_line_to_stdout(&summary(counts))?;

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
        ConfigWrite::Created => {
            print_line_to_stdout(&format!("rex skills: {config} created"))?;
            pre_commit::warn_about_missing_hook_tools();
        }
        ConfigWrite::AlreadyWritten => pre_commit::warn_about_missing_hook_tools(),
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

/// How many bundle files an install wrote, overwrote and left alone.
#[derive(Clone, Copy, Default)]
struct InstallCounts {
    written: u32,
    overwritten: u32,
    /// Files that were already right or were left alone, including `differing`.
    skipped: u32,
    /// Files left alone although they differ from the bundle.
    differing: u32,
}

/// Write each bundle file that is not on disk yet under `target_root`, with
/// the folder names in its text changed to match `vendor`. A file that is
/// there but differs from the bundle is overwritten only when `is_forced`.
fn install_bundle(
    target_root: &Path,
    vendor: Vendor,
    is_forced: bool,
) -> Result<InstallCounts, RexError> {
    let mut counts = InstallCounts::default();

    for (rel_path, contents) in bundle::walk() {
        let dest = target_root.join(&rel_path);
        let contents = vendor.retarget(contents);

        if !dest.exists() {
            write_file(&dest, &contents)?;
            counts.written += 1;
        } else if fs::read(&dest).is_ok_and(|existing| existing == *contents) {
            counts.skipped += 1;
        } else if is_forced {
            write_file(&dest, &contents)?;
            counts.overwritten += 1;
        } else {
            counts.skipped += 1;
            counts.differing += 1;
        }
    }

    Ok(counts)
}

fn write_file(dest: &Path, contents: &[u8]) -> Result<(), RexError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|source| RexError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(dest, contents).map_err(|source| RexError::Io {
        path: dest.to_path_buf(),
        source,
    })
}

/// The first line `rex skills` prints. It names the files that differ only
/// when there are some, so an up-to-date install reads as it always did.
fn summary(counts: InstallCounts) -> String {
    let InstallCounts {
        written,
        overwritten,
        skipped,
        differing,
    } = counts;

    let base = if overwritten == 0 {
        format!("rex skills: {written} written, {skipped} skipped")
    } else {
        format!("rex skills: {written} written, {overwritten} overwritten, {skipped} skipped")
    };
    match differing {
        0 => base,
        1 => format!("{base} (1 differs from this version; use --force to overwrite)"),
        _ => format!("{base} ({differing} differ from this version; use --force to overwrite)"),
    }
}

fn report_file_change(filename: &str, change: FileChange) -> Result<(), RexError> {
    match change {
        FileChange::Created => print_line_to_stdout(&format!("rex skills: {filename} created")),
        FileChange::Updated => print_line_to_stdout(&format!("rex skills: {filename} updated")),
        FileChange::Unchanged => Ok(()),
    }
}

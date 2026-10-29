//! Writes a `.pre-commit-config.yaml` for the pre-commit framework and
//! installs its git hooks, so leaked secrets, vulnerable dependencies,
//! misconfigured files, and unformatted or warning-laden Rust are caught
//! before they are committed or pushed.

use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::process::Command;

use super::file_change::contains_marker;
use crate::error::RexError;

pub(super) const CONFIG_FILENAME: &str = ".pre-commit-config.yaml";

/// The command that turns the config into git hooks. Printed as a hint
/// when the tool is missing, so the text is kept in one place.
pub(super) const INSTALL_COMMAND: &str =
    "pre-commit install --hook-type pre-commit --hook-type pre-push";

/// The first line of the config. Finding it means rex wrote the file, so a
/// second run has nothing to say about it.
const CONFIG_MARKER: &str = "# rex:pre-commit";

/// The programs the hooks in the config run, besides pre-commit itself.
const HOOK_TOOLS: [&str; 2] = ["gitleaks", "trivy"];

const CONFIG: &str = "\
# rex:pre-commit
# Installed by `rex skills`. Needs pre-commit, gitleaks and trivy on PATH.
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v6.0.0
    hooks:
      - id: check-added-large-files
        args: [--maxkb=1024]
      - id: check-merge-conflict
      - id: check-toml
      - id: check-yaml
      - id: detect-private-key
      - id: end-of-file-fixer
      - id: trailing-whitespace

  - repo: local
    hooks:
      - id: gitleaks
        name: gitleaks
        entry: gitleaks git --pre-commit --staged --redact --verbose
        language: system
        pass_filenames: false
      - id: trivy
        name: trivy secrets, vulnerabilities and misconfigurations
        entry: trivy fs --timeout 15m --quiet --skip-version-check --scanners secret,vuln,misconfig --severity HIGH,CRITICAL --exit-code 1 --skip-dirs target,node_modules,.claude,.codex,.agents --skip-files .env .
        language: system
        pass_filenames: false
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all -- --check
        language: system
        types: [rust]
        pass_filenames: false
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --workspace --all-targets -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
      - id: cargo-test
        name: cargo test
        entry: cargo test --workspace
        language: system
        types: [rust]
        pass_filenames: false
        stages: [pre-push]
";

/// The outcome of making sure the config file is there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConfigWrite {
    Created,
    /// Rex wrote the config on an earlier run.
    AlreadyWritten,
    /// Someone else's config is there. It is never replaced.
    SkippedExists,
}

/// The outcome of asking the pre-commit tool to install its hooks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HookInstall {
    Installed,
    /// `cwd/.git` is not a directory: not a repository, or a worktree or
    /// submodule whose `.git` is a file.
    SkippedNoGitDir,
    /// `pre-commit` is not on PATH.
    SkippedNoTool,
}

/// Write `cwd/.pre-commit-config.yaml` unless a config is already there.
pub(super) fn ensure_config(cwd: &Path) -> Result<ConfigWrite, RexError> {
    let path = cwd.join(CONFIG_FILENAME);
    match fs::read(&path) {
        Ok(existing) if contains_marker(&existing, CONFIG_MARKER) => {
            return Ok(ConfigWrite::AlreadyWritten);
        }
        Ok(_) => return Ok(ConfigWrite::SkippedExists),
        Err(source) if source.kind() == ErrorKind::NotFound => {}
        Err(source) => return Err(RexError::Io { path, source }),
    }
    fs::write(&path, CONFIG).map_err(|source| RexError::Io { path, source })?;
    Ok(ConfigWrite::Created)
}

/// Say on stderr which of the programs the config's hooks run are not on
/// PATH. Without one of them, its hook fails on every commit.
pub(super) fn warn_about_missing_hook_tools() {
    for tool in HOOK_TOOLS.into_iter().filter(|tool| is_missing(tool)) {
        // A warning that cannot be shown must not stop the install.
        let _ = writeln!(
            io::stderr().lock(),
            "rex skills: warning: `{tool}` not found on PATH. Its hook in {CONFIG_FILENAME} fails every commit until it is installed"
        );
    }
}

fn is_missing(tool: &str) -> bool {
    matches!(
        Command::new(tool).arg("--version").output(),
        Err(source) if source.kind() == ErrorKind::NotFound
    )
}

/// Run the pre-commit tool so the config becomes real git hooks. Its own
/// output is not shown; a non-zero exit is an error.
///
/// # Errors
/// [`RexError::Io`] if the tool cannot be started for a reason other than
/// being missing, and [`RexError::PreCommitInstall`] if it exits non-zero.
pub(super) fn install_hooks(cwd: &Path) -> Result<HookInstall, RexError> {
    if !cwd.join(".git").is_dir() {
        return Ok(HookInstall::SkippedNoGitDir);
    }

    let mut parts = INSTALL_COMMAND.split(' ');
    let tool = parts.next().unwrap_or_default();
    let output = match Command::new(tool).args(parts).current_dir(cwd).output() {
        Ok(output) => output,
        Err(source) if source.kind() == ErrorKind::NotFound => {
            return Ok(HookInstall::SkippedNoTool);
        }
        Err(source) => {
            return Err(RexError::Io {
                path: cwd.to_path_buf(),
                source,
            });
        }
    };

    if !output.status.success() {
        return Err(RexError::PreCommitInstall {
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(HookInstall::Installed)
}

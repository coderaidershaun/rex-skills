//! Installs a git pre-commit hook that runs trivy, so leaked secrets and
//! misconfigured files are caught before they are committed.

use std::fs;
use std::path::Path;

use super::file_change::contains_marker;
use crate::error::RexError;

pub(super) const HOOK_PATH: &str = ".git/hooks/pre-commit";

/// The second line of the hook. Finding it means rex wrote the hook, so a
/// second run has nothing to do.
const HOOK_MARKER: &str = "# rex:pre-commit";

/// A missing trivy fails the commit on purpose: a security hook that passes
/// in silence is worse than no hook.
const HOOK_SCRIPT: &str = "\
#!/bin/sh
# rex:pre-commit
# Installed by `rex skills`. Runs trivy on the working tree before each commit.
if ! command -v trivy >/dev/null 2>&1; then
  echo \"pre-commit: trivy is not installed; see https://trivy.dev/latest/getting-started/installation/\" >&2
  exit 1
fi
exec trivy fs --scanners secret,misconfig --exit-code 1 --quiet --skip-dirs target,node_modules,.git .
";

/// The outcome of trying to install the hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HookInstall {
    Installed,
    /// Rex wrote the hook on an earlier run.
    AlreadyInstalled,
    /// Someone else's hook is there. It is never replaced.
    SkippedExists,
    /// `cwd/.git` is not a directory: not a repository, or a worktree or
    /// submodule whose `.git` is a file.
    SkippedNoGitDir,
}

/// Write the trivy hook to `cwd/.git/hooks/pre-commit` and make it executable,
/// unless there is no `.git` directory or a hook is already there.
pub(super) fn install_hook(cwd: &Path) -> Result<HookInstall, RexError> {
    if !cwd.join(".git").is_dir() {
        return Ok(HookInstall::SkippedNoGitDir);
    }

    let path = cwd.join(HOOK_PATH);
    if let Ok(existing) = fs::read(&path) {
        return Ok(if contains_marker(&existing, HOOK_MARKER) {
            HookInstall::AlreadyInstalled
        } else {
            HookInstall::SkippedExists
        });
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| RexError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&path, HOOK_SCRIPT).map_err(|source| RexError::Io {
        path: path.clone(),
        source,
    })?;
    make_executable(&path)?;
    Ok(HookInstall::Installed)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), RexError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|source| RexError::Io {
        path: path.to_path_buf(),
        source,
    })
}

// Git on Windows runs hooks through sh, so no mode bit is needed there.
#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), RexError> {
    Ok(())
}

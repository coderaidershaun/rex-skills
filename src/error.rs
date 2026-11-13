//! Errors for every command. A failure that involves a file carries its path,
//! so the user learns where it happened, not just what went wrong.

use std::path::PathBuf;

/// Failure from any `rex` command. File failures carry the path where they happened.
#[derive(thiserror::Error, Debug)]
pub enum RexError {
    /// A file could not be read or written. A file that cannot be read because
    /// of its permissions is skipped with a warning instead of this error.
    #[error("io error at {}", path.display())]
    Io {
        /// The file rex was reading or writing.
        path: PathBuf,
        /// What the operating system said.
        #[source]
        source: std::io::Error,
    },

    /// The directory walk itself failed: a symbolic link loop, or a gitignore
    /// pattern that cannot be compiled.
    #[error("walk failed at {}", path.display())]
    Walk {
        /// The path the walk failed on, or the directory it started from when
        /// the failure names no path.
        path: PathBuf,
        /// What the walk reported.
        #[source]
        source: ignore::Error,
    },

    /// `pre-commit install` ran and exited non-zero, so the hooks in the
    /// config were not installed.
    #[error("`pre-commit install` exited with {status}: {stderr}")]
    PreCommitInstall {
        /// How the tool exited.
        status: std::process::ExitStatus,
        /// What the tool printed on stderr, which says why it failed.
        stderr: String,
    },

    /// A file rex was asked to write is a symbolic link, so nothing was
    /// written. The write would land wherever the link points, which can be
    /// outside the project.
    #[error("{} is a symbolic link; rex does not write through one", path.display())]
    SymbolicLink {
        /// The link rex refused to write through.
        path: PathBuf,
    },

    /// Stdout could not be written. A reader that closes the pipe early, such
    /// as `head`, is not this error: that is a success.
    #[error("failed to write to stdout")]
    Stdout {
        /// What the operating system said.
        #[source]
        source: std::io::Error,
    },
}

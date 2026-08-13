//! Errors for every command. Each variant carries the path involved, so a
//! failure always tells the user where it happened, not just what went wrong.

use std::path::PathBuf;

/// Failure from any `rex` command, carrying the path where it happened.
#[derive(thiserror::Error, Debug)]
pub enum RexError {
    #[error("io error at {}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("walk failed at {}", path.display())]
    Walk {
        path: PathBuf,
        #[source]
        source: ignore::Error,
    },
}

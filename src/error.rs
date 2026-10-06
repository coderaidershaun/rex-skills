//! Errors for every command. A failure that involves a file carries its path,
//! so the user learns where it happened, not just what went wrong.

use std::path::PathBuf;

/// Failure from any `rex` command. File failures carry the path where they happened.
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

    #[error("failed to write the tree to stdout")]
    Stdout {
        #[source]
        source: std::io::Error,
    },
}

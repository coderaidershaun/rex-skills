//! Walks the working directory the same way for every command that reads it:
//! dotfiles and gitignored paths are skipped, and entries come sorted by name.

use std::path::Path;

use ignore::{DirEntry, WalkBuilder};

use crate::error::RexError;

/// Visits every entry under `cwd` that is not hidden or gitignored. A
/// directory comes before what is inside it, and `cwd` itself comes first, at
/// depth 0.
///
/// # Errors
/// An item is [`RexError::Walk`] if that step of the walk fails (loop, glob,
/// permissions).
pub(super) fn visible_entries(
    cwd: &Path,
) -> impl Iterator<Item = Result<DirEntry, RexError>> + use<> {
    let root = cwd.to_path_buf();

    // require_git(false) so the command works in non-git dirs (tests, fresh
    // checkouts). Without it, `ignore` only honors .gitignore inside a git repo.
    WalkBuilder::new(cwd)
        .hidden(true)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .require_git(false)
        .sort_by_file_name(|a, b| a.cmp(b))
        .build()
        .map(move |entry| {
            entry.map_err(|source| RexError::Walk {
                path: root.clone(),
                source,
            })
        })
}

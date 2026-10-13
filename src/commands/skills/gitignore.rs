//! Adds a block to `.gitignore` so the installed skill files, which `rex
//! skills` can always write again, are not committed.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::Vendor;
use super::file_change::{FileChange, contains_marker};
use crate::error::RexError;

pub(super) const GITIGNORE_FILENAME: &str = ".gitignore";

impl Vendor {
    /// The first line of this vendor's block. Finding it in `.gitignore` is
    /// what stops a second run from adding the block again. Each vendor has
    /// its own line so a project can install more than one vendor.
    fn ignore_marker(self) -> String {
        let name = match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Agents => "agents",
        };
        format!("# rex:ignore {name}")
    }

    /// The whole block, marker line included, ending in a newline.
    fn ignore_block(self) -> String {
        let mut block = format!("{}\n{}/skills/\n", self.ignore_marker(), self.dir_name());
        if self == Self::Claude {
            block.push_str(".claude/settings.local.json\n");
        }
        block
    }
}

/// Make sure `cwd/.gitignore` has the vendor's block: create the file if it is
/// missing, append the block after the existing lines if it is not there yet,
/// and leave the file alone if it is.
pub(super) fn ensure_ignore_block(cwd: &Path, vendor: Vendor) -> Result<FileChange, RexError> {
    let path = cwd.join(GITIGNORE_FILENAME);

    let existing = match fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(source) if source.kind() == ErrorKind::NotFound => None,
        Err(source) => return Err(RexError::Io { path, source }),
    };

    let block = vendor.ignore_block();
    let (contents, change) = match existing {
        None => (block.into_bytes(), FileChange::Created),
        Some(old) if contains_marker(&old, &vendor.ignore_marker()) => {
            return Ok(FileChange::Unchanged);
        }
        Some(mut old) => {
            if !old.ends_with(b"\n") {
                old.push(b'\n');
            }
            old.push(b'\n');
            old.extend_from_slice(block.as_bytes());
            (old, FileChange::Updated)
        }
    };

    fs::write(&path, contents).map_err(|source| RexError::Io { path, source })?;
    Ok(change)
}

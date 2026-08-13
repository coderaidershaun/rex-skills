//! Installs the embedded skill bundle into the current working directory.

use std::fs;
use std::path::Path;

use crate::bundle;
use crate::error::RexError;

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
/// # Errors
/// [`RexError::Io`] if a parent directory or file cannot be created or written.
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
    Ok(())
}

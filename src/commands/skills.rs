//! Installs the embedded skill bundle into the current working directory and
//! points agents at the `rex codebase` command through `AGENTS.md`.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::bundle;
use crate::error::RexError;

const AGENTS_FILENAME: &str = "AGENTS.md";

/// The codebase section starts with this line. Finding it in `AGENTS.md` is
/// what stops a second run from adding the section again.
const CODEBASE_MARKER: &str = "<!-- rex:codebase -->";

const CODEBASE_SECTION_BODY: &str = "\
## Codebase map

Run one of these commands before you search the tree by hand. Each one prints a tree of the working directory and skips the files that git ignores.

- `rex codebase`: every file.
- `rex codebase --rust-only`: only `.rs` files.
- `rex codebase --with-context`: adds the first sentence of each Rust module's `//!` doc comment to its line.
- `rex codebase --rust-only --with-context`: the Rust modules and what each one is for. Start here in a Rust crate.
";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentsMdChange {
    Created,
    Updated,
    Unchanged,
}

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
/// Then make sure `cwd/AGENTS.md` has the codebase-map section: create the
/// file if it is missing, put the section above the existing text if it has
/// no section yet, and leave the file alone if it does.
///
/// # Errors
/// [`RexError::Io`] if a parent directory or file cannot be created or written,
/// or if `AGENTS.md` cannot be read or written.
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

    match ensure_codebase_section(cwd)? {
        AgentsMdChange::Created => println!("rex skills: {AGENTS_FILENAME} created"),
        AgentsMdChange::Updated => println!("rex skills: {AGENTS_FILENAME} updated"),
        AgentsMdChange::Unchanged => {}
    }
    Ok(())
}

fn ensure_codebase_section(cwd: &Path) -> Result<AgentsMdChange, RexError> {
    let path = cwd.join(AGENTS_FILENAME);

    // The file is read as bytes because a file that is not valid UTF-8 must
    // not fail the command after the bundle is already installed.
    let existing = match fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(source) if source.kind() == ErrorKind::NotFound => None,
        Err(source) => return Err(RexError::Io { path, source }),
    };

    let mut contents = format!("{CODEBASE_MARKER}\n{CODEBASE_SECTION_BODY}").into_bytes();
    let change = match existing {
        None => AgentsMdChange::Created,
        Some(old) if contains_marker(&old) => return Ok(AgentsMdChange::Unchanged),
        Some(old) => {
            contents.push(b'\n');
            contents.extend_from_slice(&old);
            AgentsMdChange::Updated
        }
    };

    // The file is written in place, not through a temp file and a rename, so
    // an `AGENTS.md` that is a symlink stays a symlink.
    fs::write(&path, contents).map_err(|source| RexError::Io { path, source })?;
    Ok(change)
}

fn contains_marker(contents: &[u8]) -> bool {
    contents
        .windows(CODEBASE_MARKER.len())
        .any(|window| window == CODEBASE_MARKER.as_bytes())
}

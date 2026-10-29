//! Writes the embedded skill bundle into a folder: the files that are missing,
//! and the ones that differ when the caller asks for it.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::Vendor;
use crate::bundle;
use crate::error::RexError;

/// How many bundle files an install wrote, overwrote and left alone.
#[derive(Clone, Copy, Default)]
pub(super) struct InstallCounts {
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
///
/// A file is never written through a symbolic link, with or without `is_forced`,
/// because the write would land on whatever the link points at. That covers the
/// file itself and the folders of its own skill. The `skills` folder is
/// followed, because people link a shared one on purpose.
pub(super) fn install_bundle(
    target_root: &Path,
    vendor: Vendor,
    is_forced: bool,
) -> Result<InstallCounts, RexError> {
    let mut counts = InstallCounts::default();
    let skills_dir = target_root.join("skills");

    for (rel_path, contents) in bundle::walk() {
        let dest = target_root.join(&rel_path);
        let contents = vendor.retarget(contents);

        let is_present = dest.exists();
        if is_present && fs::read(&dest).is_ok_and(|existing| existing == *contents) {
            counts.skipped += 1;
        } else if is_present && !is_forced {
            counts.skipped += 1;
            counts.differing += 1;
        } else if is_linked(&dest, &skills_dir) {
            warn_linked(&dest);
            counts.skipped += 1;
        } else {
            write_file(&dest, &contents)?;
            if is_present {
                counts.overwritten += 1;
            } else {
                counts.written += 1;
            }
        }
    }

    Ok(counts)
}

/// True when `dest`, or a folder between it and `skills_dir`, is a symbolic
/// link, whether or not the link points at something that exists.
fn is_linked(dest: &Path, skills_dir: &Path) -> bool {
    dest.ancestors()
        .take_while(|path| *path != skills_dir)
        .any(|path| {
            fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
        })
}

fn warn_linked(path: &Path) {
    // A warning that cannot be shown must not stop the install.
    let _ = writeln!(
        io::stderr().lock(),
        "rex skills: skipped {}: it or a folder above it is a symbolic link",
        path.display()
    );
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
pub(super) fn summary(counts: InstallCounts) -> String {
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

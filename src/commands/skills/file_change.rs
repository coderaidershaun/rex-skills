//! What `rex skills` did to one project file it owns a block in, and the
//! marker search that decides whether the block is already there.

/// The outcome of making sure a file has the rex block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FileChange {
    Created,
    Updated,
    Unchanged,
}

/// True when `contents` holds `marker` anywhere. Works on bytes so a file
/// that is not valid UTF-8 can still be checked.
pub(super) fn contains_marker(contents: &[u8], marker: &str) -> bool {
    contents
        .windows(marker.len())
        .any(|window| window == marker.as_bytes())
}

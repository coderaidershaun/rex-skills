//! Embeds the `.claude` skill tree into the binary at compile time, so the
//! install command works anywhere without files shipped alongside it.

use std::path::PathBuf;

use include_dir::{Dir, include_dir};

static EMBEDDED_CLAUDE: Dir = include_dir!("$CARGO_MANIFEST_DIR/.claude");

/// List every file in the embedded skill bundle as a path relative to the
/// bundle root, paired with its contents.
///
/// Paths always start with `skills/`, since that is the only directory
/// under `.claude` at embed time.
#[must_use]
pub fn walk() -> Vec<(PathBuf, &'static [u8])> {
    let mut entries = Vec::new();
    walk_dir(&EMBEDDED_CLAUDE, &mut entries);
    entries
}

fn walk_dir(dir: &'static Dir<'static>, entries: &mut Vec<(PathBuf, &'static [u8])>) {
    // File::path() is relative to the include_dir! root, not to `dir`, so we
    // use it directly and never join it onto a per-level prefix.
    for file in dir.files() {
        entries.push((file.path().to_path_buf(), file.contents()));
    }
    for subdir in dir.dirs() {
        walk_dir(subdir, entries);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    // This test must never be deleted. A file like .claude/settings.local.json
    // can be hidden from `git status` by the user's global git ignore rules,
    // so it could end up embedded in the published crate without ever
    // showing up as a change. This is the only check that would catch it.
    #[test]
    fn bundle_contains_only_skills_paths() {
        let entries = walk();
        assert!(!entries.is_empty(), "embedded bundle must not be empty");

        let has_index = entries
            .iter()
            .any(|(path, _)| path == Path::new("skills/INDEX.md"));
        assert!(has_index, "embedded bundle must contain skills/INDEX.md");

        for (path, _) in &entries {
            let first_component = path
                .components()
                .next()
                .expect("bundle path must have at least one component");
            assert_eq!(
                first_component.as_os_str(),
                "skills",
                "every embedded path must start with skills/, found {}",
                path.display()
            );
        }
    }
}

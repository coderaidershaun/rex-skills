use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::tempdir;

fn install(dir: &Path, vendor: Option<&str>) {
    let mut command = Command::cargo_bin("rex").expect("find rex binary");
    command.arg("skills");
    if let Some(vendor) = vendor {
        command.args(["--vendor", vendor]);
    }
    command.current_dir(dir).assert().success();
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("read entry").path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

// This test must never be deleted. Skill text names the folder that holds the
// skills, and a copy that names another tool's folder sends an agent to files
// that are not there.
#[test]
fn a_vendor_install_names_its_own_skills_folder_in_every_file() {
    for (vendor, dot_dir) in [("codex", ".codex"), ("agents", ".agents")] {
        let dir = tempdir().expect("create tempdir");
        install(dir.path(), Some(vendor));

        let installed = files_under(&dir.path().join(dot_dir));
        assert!(!installed.is_empty(), "{vendor} install wrote no files");

        for file in installed {
            let text = fs::read(&file).expect("read installed file");
            assert!(
                !contains(&text, ".claude"),
                "{} still names .claude",
                file.display()
            );
        }

        let lead = fs::read(
            dir.path()
                .join(dot_dir)
                .join("skills/rex-code-lead/SKILL.md"),
        )
        .expect("read lead skill");
        assert!(
            contains(&lead, &format!("Read {dot_dir}/skills/INDEX.md.")),
            "the lead must be told to read the index in {dot_dir}"
        );
    }
}

#[test]
fn the_claude_install_is_byte_for_byte_the_bundle() {
    let dir = tempdir().expect("create tempdir");
    install(dir.path(), None);

    for (path, contents) in rex_skills::bundle::walk() {
        let installed = fs::read(dir.path().join(".claude").join(&path)).expect("read installed");
        assert_eq!(installed, contents, "{} was changed", path.display());
    }
}

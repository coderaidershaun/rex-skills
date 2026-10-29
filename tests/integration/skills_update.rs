use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

const HOOKS_LINE: &str = "rex skills: pre-commit hooks skipped (no .git directory)\n";

fn bundle_file_count() -> usize {
    rex_skills::bundle::walk().len()
}

fn bundle_index() -> &'static [u8] {
    rex_skills::bundle::walk()
        .into_iter()
        .find(|(path, _)| path == Path::new("skills/INDEX.md"))
        .expect("bundle must contain skills/INDEX.md")
        .1
}

fn rex_skills_stdout(dir: &Path, args: &[&str]) -> String {
    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .args(args)
        .current_dir(dir)
        .assert()
        .success();
    String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout")
}

#[test]
fn skipped_files_that_differ_from_the_bundle_are_counted_and_left_alone() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();
    rex_skills_stdout(dir.path(), &[]);
    let index = dir.path().join(".claude/skills/INDEX.md");
    fs::write(&index, b"my own edit").expect("edit INDEX.md");

    let stdout = rex_skills_stdout(dir.path(), &[]);

    assert_eq!(
        stdout,
        format!(
            "rex skills: 0 written, {n} skipped (1 differs from this version; use --force to overwrite)\n{HOOKS_LINE}"
        )
    );
    assert_eq!(fs::read(index).expect("read INDEX.md"), b"my own edit");
}

#[test]
fn several_differing_files_are_counted_together() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();
    rex_skills_stdout(dir.path(), &[]);
    for edited in ["INDEX.md", "rex-code-lead/SKILL.md"] {
        fs::write(
            dir.path().join(".claude/skills").join(edited),
            b"my own edit",
        )
        .expect("edit");
    }

    let stdout = rex_skills_stdout(dir.path(), &[]);

    assert_eq!(
        stdout,
        format!(
            "rex skills: 0 written, {n} skipped (2 differ from this version; use --force to overwrite)\n{HOOKS_LINE}"
        )
    );
}

#[test]
fn force_overwrites_only_the_files_that_differ() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();
    rex_skills_stdout(dir.path(), &[]);
    let index = dir.path().join(".claude/skills/INDEX.md");
    fs::write(&index, b"my own edit").expect("edit INDEX.md");

    let stdout = rex_skills_stdout(dir.path(), &["--force"]);

    assert_eq!(
        stdout,
        format!(
            "rex skills: 0 written, 1 overwritten, {} skipped\n{HOOKS_LINE}",
            n - 1
        )
    );
    assert_eq!(fs::read(index).expect("read INDEX.md"), bundle_index());
}

#[test]
fn force_on_an_up_to_date_install_changes_nothing() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();
    rex_skills_stdout(dir.path(), &[]);

    let stdout = rex_skills_stdout(dir.path(), &["--force"]);

    assert_eq!(
        stdout,
        format!("rex skills: 0 written, {n} skipped\n{HOOKS_LINE}")
    );
}

// This test must never be deleted. --force is about the skill files only. A
// project file that rex also manages, but that the user has taken over, must
// stay exactly as the user left it.
#[test]
fn force_never_replaces_the_project_files_a_user_may_have_edited() {
    let dir = tempdir().expect("create tempdir");
    rex_skills_stdout(dir.path(), &[]);
    let edited = [
        ("AGENTS.md", "my own agent notes\n"),
        (".gitignore", "/my-own-ignore\n"),
        (".pre-commit-config.yaml", "my own hooks\n"),
    ];
    for (name, contents) in edited {
        fs::write(dir.path().join(name), contents).expect("write project file");
    }

    rex_skills_stdout(dir.path(), &["--force"]);

    for (name, contents) in edited {
        let after = fs::read_to_string(dir.path().join(name)).expect("read project file");
        assert!(
            after.contains(contents),
            "{name} lost the user's text: {after:?}"
        );
    }
    let config = fs::read_to_string(dir.path().join(".pre-commit-config.yaml")).expect("read");
    assert_eq!(config, "my own hooks\n");
}

#[test]
fn force_works_for_another_vendor_too() {
    let dir = tempdir().expect("create tempdir");
    rex_skills_stdout(dir.path(), &["--vendor", "codex"]);
    let index = dir.path().join(".codex/skills/INDEX.md");
    fs::write(&index, b"my own edit").expect("edit INDEX.md");

    let stdout = rex_skills_stdout(dir.path(), &["--vendor", "codex", "--force"]);

    assert!(stdout.contains(", 1 overwritten, "), "got: {stdout}");
    let installed = fs::read_to_string(index).expect("read INDEX.md");
    assert!(
        installed.contains(".codex/skills/") && !installed.contains(".claude"),
        "the overwrite must use the retargeted text"
    );
}

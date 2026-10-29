use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

fn rex_skills(cwd: &Path, args: &[&str]) -> String {
    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .args(args)
        .current_dir(cwd)
        .assert()
        .success();
    String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr")
}

// This test must never be deleted. A project can hold a symlink where a skill
// file belongs. Writing through it would replace a file outside the project,
// such as a shell profile.
#[test]
fn force_never_overwrites_the_file_a_symlink_points_at() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    let skills = project.join(".claude/skills");
    fs::create_dir_all(&skills).expect("create skills dir");
    let outside = dir.path().join("outside.txt");
    fs::write(&outside, "precious").expect("write outside file");
    symlink(&outside, skills.join("INDEX.md")).expect("create symlink");

    let stderr = rex_skills(&project, &["--force"]);

    assert_eq!(
        fs::read_to_string(&outside).expect("read outside file"),
        "precious"
    );
    assert!(
        fs::symlink_metadata(skills.join("INDEX.md"))
            .expect("stat INDEX.md")
            .file_type()
            .is_symlink(),
        "the symlink must still be a symlink"
    );
    assert!(
        stderr.contains("rex skills: skipped ./.claude/skills/INDEX.md: it or a folder above it is a symbolic link"),
        "got: {stderr}"
    );
}

#[test]
fn a_symlink_to_a_missing_path_is_not_written_through() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    let skill_dir = project.join(".claude/skills/rex-caveman");
    fs::create_dir_all(&skill_dir).expect("create skill dir");
    let outside = dir.path().join("created-outside.txt");
    symlink(&outside, skill_dir.join("SKILL.md")).expect("create dangling symlink");

    rex_skills(&project, &[]);

    assert!(
        !outside.exists(),
        "rex must not create a file where the symlink points"
    );
    assert!(
        project
            .join(".claude/skills/rex-code-lead/SKILL.md")
            .is_file(),
        "the other skill files must still be installed"
    );
}

// A skill folder that is a symlink would send every file of that skill to the
// folder the link points at.
#[test]
fn a_symlinked_skill_folder_is_not_written_into() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    let skills = project.join(".claude/skills");
    fs::create_dir_all(&skills).expect("create skills dir");
    let outside = dir.path().join("outside-dir");
    fs::create_dir_all(&outside).expect("create outside dir");
    symlink(&outside, skills.join("rex-code-lead")).expect("create folder symlink");

    let stderr = rex_skills(&project, &[]);

    assert_eq!(fs::read_dir(&outside).expect("read outside dir").count(), 0);
    assert!(
        stderr.contains("rex skills: skipped ./.claude/skills/rex-code-lead/SKILL.md: it or a folder above it is a symbolic link"),
        "got: {stderr}"
    );
    assert!(
        skills.join("INDEX.md").is_file(),
        "the other skills must still be installed"
    );
}

#[test]
fn a_symlink_to_an_up_to_date_file_is_left_alone_without_a_warning() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    rex_skills(&project_with_install(&project), &[]);
    let index = project.join(".claude/skills/INDEX.md");
    let copy = dir.path().join("index-copy.md");
    fs::copy(&index, &copy).expect("copy INDEX.md");
    fs::remove_file(&index).expect("remove INDEX.md");
    symlink(&copy, &index).expect("create symlink");

    let stderr = rex_skills(&project, &["--force"]);

    assert!(!stderr.contains("symbolic link"), "got: {stderr}");
}

// People link a shared skills folder into a project on purpose. That folder is
// followed, and everything in it is installed as usual.
#[test]
fn a_symlinked_skills_folder_still_receives_the_install() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    let shared = dir.path().join("shared");
    fs::create_dir_all(project.join(".claude")).expect("create .claude");
    fs::create_dir_all(&shared).expect("create shared dir");
    symlink(&shared, project.join(".claude/skills")).expect("create folder symlink");

    let stderr = rex_skills(&project, &[]);

    assert!(shared.join("INDEX.md").is_file());
    assert!(!stderr.contains("symbolic link"), "got: {stderr}");
}

fn project_with_install(project: &Path) -> std::path::PathBuf {
    fs::create_dir_all(project).expect("create project");
    project.to_path_buf()
}

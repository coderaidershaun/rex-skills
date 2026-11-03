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

// This test must never be deleted. The same hazard as a linked skill file:
// a write through a link lands on whatever it points at, outside the project.
// AGENTS.md is the one rex changes in place rather than only creating, so a
// link here would edit a file the person never named.
#[test]
fn a_symlinked_agents_md_is_left_alone_and_its_target_untouched() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    fs::create_dir_all(&project).expect("create project");
    let outside = dir.path().join("shared-notes.md");
    fs::write(&outside, "precious").expect("write outside file");
    symlink(&outside, project.join("AGENTS.md")).expect("create symlink");

    let stderr = rex_skills(&project, &[]);

    assert_eq!(
        fs::read_to_string(&outside).expect("read outside file"),
        "precious",
        "rex must not write through the link"
    );
    assert!(
        stderr.contains("rex skills: skipped ./AGENTS.md: it is a symbolic link"),
        "got: {stderr}"
    );
}

// This test must never be deleted. A link that points at nothing is the worst
// case: the write creates a file at a path outside the project that nobody
// asked for, and the person has no warning that it happened.
#[test]
fn a_dangling_link_never_creates_a_project_file_outside_the_project() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    fs::create_dir_all(&project).expect("create project");
    let linked = [".gitignore", ".pre-commit-config.yaml"];
    for name in linked {
        symlink(dir.path().join(name), project.join(name)).expect("create dangling symlink");
    }

    let stderr = rex_skills(&project, &[]);

    for name in linked {
        assert!(
            !dir.path().join(name).exists(),
            "rex created {name} outside the project"
        );
        assert!(
            stderr.contains(&format!(
                "rex skills: skipped ./{name}: it is a symbolic link"
            )),
            "got: {stderr}"
        );
    }
    assert!(
        project.join(".claude/skills/INDEX.md").is_file(),
        "the skill bundle must still be installed"
    );
}

// Writing CODEBASE.md is all `rex codebase --for-human` does, so a link it
// must not write through has to fail. Exiting 0 would tell a script the file
// was written.
#[test]
fn codebase_for_human_fails_on_a_symlinked_output_and_writes_nothing_outside() {
    let dir = tempdir().expect("create tempdir");
    let project = dir.path().join("project");
    fs::create_dir_all(&project).expect("create project");
    let outside = dir.path().join("outside-codebase.md");
    symlink(&outside, project.join("CODEBASE.md")).expect("create dangling symlink");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .args(["codebase", "--for-human"])
        .current_dir(&project)
        .assert()
        .failure();

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("./CODEBASE.md is a symbolic link; rex does not write through one"),
        "got: {stderr}"
    );
    assert!(
        !outside.exists(),
        "rex created a file where the link points"
    );
}

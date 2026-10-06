use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

fn bundle_file_count() -> usize {
    rex_skills::bundle::walk().len()
}

fn find_index_md_contents() -> &'static [u8] {
    rex_skills::bundle::walk()
        .into_iter()
        .find(|(path, _)| path == Path::new("skills/INDEX.md"))
        .expect("bundle must contain skills/INDEX.md")
        .1
}

#[test]
fn fresh_install_writes_every_bundle_file() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert_eq!(
        stdout,
        format!("rex skills: {n} written, 0 skipped\nrex skills: AGENTS.md created\n")
    );

    let installed = fs::read(dir.path().join(".claude/skills/INDEX.md")).expect("read INDEX.md");
    assert_eq!(installed, find_index_md_contents());
}

// This test must never be deleted. It is the only check that running the
// install command a second time does not rewrite files it already wrote.
#[test]
fn second_run_writes_nothing() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let before = fs::read(dir.path().join(".claude/skills/INDEX.md")).expect("read INDEX.md");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert_eq!(stdout, format!("rex skills: 0 written, {n} skipped\n"));

    let after = fs::read(dir.path().join(".claude/skills/INDEX.md")).expect("read INDEX.md");
    assert_eq!(
        before, after,
        "second run must not rewrite an already-installed file"
    );
}

// This test must never be deleted. It is the only check that a file the
// user has already edited is never overwritten by the install command.
#[test]
fn existing_modified_file_is_never_touched() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();

    fs::create_dir_all(dir.path().join(".claude/skills")).expect("create .claude/skills");
    fs::write(
        dir.path().join(".claude/skills/INDEX.md"),
        b"sentinel content",
    )
    .expect("write sentinel INDEX.md");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert_eq!(
        stdout,
        format!(
            "rex skills: {} written, 1 skipped\nrex skills: AGENTS.md created\n",
            n - 1
        )
    );

    let contents = fs::read(dir.path().join(".claude/skills/INDEX.md")).expect("read INDEX.md");
    assert_eq!(contents, b"sentinel content");
}

#[test]
fn missing_file_is_backfilled() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    fs::remove_file(dir.path().join(".claude/skills/INDEX.md")).expect("remove INDEX.md");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert_eq!(
        stdout,
        format!("rex skills: 1 written, {} skipped\n", n - 1)
    );

    let restored = fs::read(dir.path().join(".claude/skills/INDEX.md")).expect("read INDEX.md");
    assert_eq!(restored, find_index_md_contents());
}

#[test]
fn vendor_codex_installs_under_dot_codex() {
    let dir = tempdir().expect("create tempdir");

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .args(["skills", "--vendor", "codex"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join(".codex/skills/INDEX.md").is_file());
    assert!(!dir.path().join(".claude").exists());
}

#[test]
fn vendor_agents_installs_under_dot_agents() {
    let dir = tempdir().expect("create tempdir");

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .args(["skills", "--vendor", "agents"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join(".agents/skills/INDEX.md").is_file());
    assert!(!dir.path().join(".claude").exists());
}

#[test]
fn unknown_vendor_is_rejected() {
    let dir = tempdir().expect("create tempdir");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .args(["skills", "--vendor", "cursor"])
        .current_dir(dir.path())
        .assert()
        .failure();

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(stderr.contains("possible values"));
}

#[test]
fn init_alias_installs_like_skills() {
    let dir = tempdir().expect("create tempdir");
    let n = bundle_file_count();

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert_eq!(
        stdout,
        format!("rex skills: {n} written, 0 skipped\nrex skills: AGENTS.md created\n")
    );

    assert!(dir.path().join(".claude/skills/INDEX.md").is_file());
}

#[test]
fn fresh_install_creates_agents_md_with_codebase_section() {
    let dir = tempdir().expect("create tempdir");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .args(["skills", "--vendor", "codex"])
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(stdout.ends_with("rex skills: AGENTS.md created\n"));

    let agents_md = fs::read_to_string(dir.path().join("AGENTS.md")).expect("read AGENTS.md");
    assert!(agents_md.starts_with("<!-- rex:codebase -->\n## Codebase map\n"));
    assert!(agents_md.contains("`rex codebase --rust-only --with-context`"));
    assert!(agents_md.contains("`rex codebase --for-human --rust-only --with-context`"));
}

#[test]
fn existing_agents_md_gets_section_on_top_once() {
    let dir = tempdir().expect("create tempdir");
    let old: &[u8] = b"# My project\n\nKeep me exactly as I am.\n";
    fs::write(dir.path().join("AGENTS.md"), old).expect("write AGENTS.md");

    let first = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(first.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(stdout.ends_with("rex skills: AGENTS.md updated\n"));

    let after_first = fs::read(dir.path().join("AGENTS.md")).expect("read AGENTS.md");
    assert!(after_first.starts_with(b"<!-- rex:codebase -->\n## Codebase map\n"));
    assert!(
        after_first.ends_with(&[b"\n\n", old].concat()),
        "old bytes must follow the section after one blank line"
    );

    let second = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(second.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(!stdout.contains("AGENTS.md"));

    let after_second = fs::read(dir.path().join("AGENTS.md")).expect("read AGENTS.md");
    assert_eq!(after_first, after_second);
}

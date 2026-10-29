use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

/// Puts a `pre-commit` on PATH that runs `body` as a shell script.
fn fake_pre_commit(dir: &Path, body: &str) -> std::path::PathBuf {
    let bin = dir.join("fakebin");
    fs::create_dir_all(&bin).expect("create fakebin");
    let tool = bin.join("pre-commit");
    fs::write(&tool, format!("#!/bin/sh\n{body}\n")).expect("write fake pre-commit");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("chmod fake tool");
    }
    bin
}

// This test pins how a failed `pre-commit install` looks to the caller. The
// real tool refuses when git's core.hooksPath is set, and the person running
// rex must see why, not just that it failed.
#[test]
fn a_failed_pre_commit_install_exits_with_its_error_text_after_the_rest_is_done() {
    let dir = tempdir().expect("create tempdir");
    fs::create_dir_all(dir.path().join(".git/hooks")).expect("create .git/hooks");
    let bin = fake_pre_commit(
        dir.path(),
        "echo 'Cowardly refusing to install hooks with core.hooksPath set' >&2\nexit 1",
    );

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", bin)
        .current_dir(dir.path())
        .assert()
        .failure();

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains("`pre-commit install` exited with"),
        "got: {stderr}"
    );
    assert!(
        stderr.contains("Cowardly refusing to install hooks"),
        "got: {stderr}"
    );
    assert!(dir.path().join(".claude/skills/INDEX.md").is_file());
    assert!(dir.path().join("AGENTS.md").is_file());
    assert!(dir.path().join(".pre-commit-config.yaml").is_file());
}

#[test]
fn a_git_file_instead_of_a_git_directory_skips_the_hooks_and_never_runs_pre_commit() {
    let dir = tempdir().expect("create tempdir");
    fs::write(dir.path().join(".git"), "gitdir: ../elsewhere\n").expect("write .git file");
    let ran = dir.path().join("pre-commit-ran");
    // The marker is made with a shell redirect, not `touch`: PATH holds only the
    // fake folder, so an external command would not be found.
    let bin = fake_pre_commit(dir.path(), &format!(": > '{}'", ran.display()));

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", bin)
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(!ran.exists(), "pre-commit must not run when .git is a file");
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(
        stdout.ends_with("rex skills: pre-commit hooks skipped (no .git directory)\n"),
        "got: {stdout}"
    );
}

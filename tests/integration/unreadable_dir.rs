use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::tempdir;

/// A directory that nobody can read. It is made readable again when dropped,
/// so the temp dir can be removed even if the test fails.
struct LockedDir(PathBuf);

impl LockedDir {
    /// `None` when the lock has no effect, as when the tests run as root.
    fn create(path: PathBuf) -> Option<Self> {
        fs::create_dir_all(path.join("inner")).expect("create locked dir");
        fs::set_permissions(&path, Permissions::from_mode(0o000)).expect("lock dir");
        let locked = Self(path);
        fs::read_dir(&locked.0).is_err().then_some(locked)
    }
}

impl Drop for LockedDir {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, Permissions::from_mode(0o755));
    }
}

#[test]
fn codebase_skips_an_unreadable_directory_and_warns() {
    let dir = tempdir().expect("create tempdir");
    fs::create_dir_all(dir.path().join("ok")).expect("create ok dir");
    fs::write(dir.path().join("ok/a.rs"), "fn a() {}\n").expect("write ok/a.rs");
    let Some(_locked) = LockedDir::create(dir.path().join("locked")) else {
        return;
    };

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("codebase")
        .current_dir(dir.path())
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).expect("utf8 stdout");
    let stderr = String::from_utf8(output.stderr.clone()).expect("utf8 stderr");
    assert_eq!(stdout, "locked/\nok/\n  a.rs\n");
    assert!(
        stderr.contains("rex: skipped ./locked: permission denied"),
        "warning must name the skipped path, got: {stderr}"
    );
}

#[test]
fn smells_still_counts_flags_in_readable_files_and_warns() {
    let dir = tempdir().expect("create tempdir");
    fs::write(dir.path().join("a.rs"), "// SMELL: bad name\nfn a() {}\n").expect("write a.rs");
    let Some(_locked) = LockedDir::create(dir.path().join("locked")) else {
        return;
    };

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("smells")
        .current_dir(dir.path())
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).expect("utf8 stdout");
    let stderr = String::from_utf8(output.stderr.clone()).expect("utf8 stderr");
    assert_eq!(stdout, "a.rs:1\nCode smells: 1\n");
    assert!(
        stderr.contains("rex: skipped ./locked: permission denied"),
        "warning must name the skipped path, got: {stderr}"
    );
}

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

/// A folder holding the named tools as scripts that do nothing, so a run can
/// find them on PATH without the real programs being installed.
fn bin_with(dir: &Path, tools: &[&str]) -> std::path::PathBuf {
    let bin = dir.join("fakebin");
    fs::create_dir_all(&bin).expect("create fakebin");
    for tool in tools {
        let path = bin.join(tool);
        fs::write(&path, "#!/bin/sh\nexit 0\n").expect("write fake tool");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
        }
    }
    bin
}

fn stderr_of_install(dir: &Path, path_env: &Path) -> String {
    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", path_env)
        .current_dir(dir)
        .assert()
        .success();
    String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr")
}

#[test]
fn a_missing_hook_tool_is_named_on_stderr() {
    let dir = tempdir().expect("create tempdir");
    let empty = bin_with(dir.path(), &[]);

    let stderr = stderr_of_install(dir.path(), &empty);

    assert!(
        stderr.contains("rex skills: warning: `gitleaks` not found on PATH"),
        "got: {stderr}"
    );
    assert!(
        stderr.contains("rex skills: warning: `trivy` not found on PATH"),
        "got: {stderr}"
    );
}

#[test]
fn only_the_tool_that_is_missing_is_named() {
    let dir = tempdir().expect("create tempdir");
    let bin = bin_with(dir.path(), &["gitleaks"]);

    let stderr = stderr_of_install(dir.path(), &bin);

    assert!(!stderr.contains("gitleaks"), "got: {stderr}");
    assert!(stderr.contains("`trivy` not found"), "got: {stderr}");
}

#[test]
fn no_warning_when_both_tools_are_installed() {
    let dir = tempdir().expect("create tempdir");
    let bin = bin_with(dir.path(), &["gitleaks", "trivy"]);

    let stderr = stderr_of_install(dir.path(), &bin);

    assert_eq!(stderr, "");
}

#[test]
fn a_config_the_user_wrote_gets_no_tool_warning() {
    let dir = tempdir().expect("create tempdir");
    let empty = bin_with(dir.path(), &[]);
    fs::write(dir.path().join(".pre-commit-config.yaml"), "repos: []\n").expect("write config");

    let stderr = stderr_of_install(dir.path(), &empty);

    assert_eq!(
        stderr, "",
        "rex does not know which tools the user's hooks need"
    );
}

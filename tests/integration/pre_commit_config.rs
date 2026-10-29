use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;
use yaml_rust2::YamlLoader;

/// The `stages` of the local hook called `id` in the config that `rex skills`
/// writes. `None` means the hook has no `stages` line, so it runs at commit.
fn stages_of(id: &str) -> Option<Vec<String>> {
    let dir = tempdir().expect("create tempdir");
    Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let text = fs::read_to_string(dir.path().join(".pre-commit-config.yaml")).expect("read config");
    let docs = YamlLoader::load_from_str(&text).expect("the config is valid YAML");

    let repos = docs[0]["repos"].as_vec().expect("repos is a list");
    let hook = repos
        .iter()
        .filter_map(|repo| repo["hooks"].as_vec())
        .flatten()
        .find(|hook| hook["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("the config has no hook `{id}`"));

    hook["stages"].as_vec().map(|stages| {
        stages
            .iter()
            .map(|stage| stage.as_str().expect("a stage is text").to_string())
            .collect()
    })
}

#[test]
fn the_secret_scan_of_the_staged_diff_runs_at_commit() {
    assert_eq!(stages_of("gitleaks"), None);
}

#[test]
fn the_slow_whole_tree_scan_runs_at_push() {
    assert_eq!(stages_of("trivy"), Some(vec!["pre-push".to_string()]));
}

#[test]
fn the_tests_run_at_push() {
    assert_eq!(stages_of("cargo-test"), Some(vec!["pre-push".to_string()]));
}

#[test]
fn the_fast_format_and_lint_checks_run_at_commit() {
    assert_eq!(stages_of("cargo-fmt"), None);
    assert_eq!(stages_of("cargo-clippy"), None);
}

#[test]
fn a_missing_trivy_is_reported_as_a_push_problem_and_gitleaks_as_a_commit_problem() {
    let dir = tempdir().expect("create tempdir");
    let empty = dir.path().join("emptybin");
    fs::create_dir_all(&empty).expect("create emptybin");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", &empty)
        .current_dir(dir.path())
        .assert()
        .success();

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    assert!(
        stderr.contains(
            "`gitleaks` not found on PATH. Its hook in .pre-commit-config.yaml fails every commit"
        ),
        "got: {stderr}"
    );
    assert!(
        stderr.contains(
            "`trivy` not found on PATH. Its hook in .pre-commit-config.yaml fails every push"
        ),
        "got: {stderr}"
    );
}

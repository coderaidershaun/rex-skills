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
        format!(
            "rex skills: {n} written, 0 skipped\nrex skills: AGENTS.md created\nrex skills: .gitignore created\nrex skills: .pre-commit-config.yaml created\nrex skills: pre-commit hooks skipped (no .git directory)\n"
        )
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
    assert_eq!(
        stdout,
        format!(
            "rex skills: 0 written, {n} skipped\nrex skills: pre-commit hooks skipped (no .git directory)\n"
        )
    );

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
            "rex skills: {} written, 1 skipped\nrex skills: AGENTS.md created\nrex skills: .gitignore created\nrex skills: .pre-commit-config.yaml created\nrex skills: pre-commit hooks skipped (no .git directory)\n",
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
        format!(
            "rex skills: 1 written, {} skipped\nrex skills: pre-commit hooks skipped (no .git directory)\n",
            n - 1
        )
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
        format!(
            "rex skills: {n} written, 0 skipped\nrex skills: AGENTS.md created\nrex skills: .gitignore created\nrex skills: .pre-commit-config.yaml created\nrex skills: pre-commit hooks skipped (no .git directory)\n"
        )
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
    assert!(stdout.contains("rex skills: AGENTS.md created\n"));

    let agents_md = fs::read_to_string(dir.path().join("AGENTS.md")).expect("read AGENTS.md");
    assert!(agents_md.starts_with("<!-- rex:codebase -->\n## Codebase map\n"));
    assert!(agents_md.contains("`rex codebase --rust-only --with-context`"));
    assert!(agents_md.contains("`rex codebase --for-human --rust-only --with-context`"));
    assert!(agents_md.contains("\n## Code smells\n"));
    assert!(agents_md.contains("- `rex smells`: "));
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
    assert!(stdout.contains("rex skills: AGENTS.md updated\n"));

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

#[test]
fn fresh_install_creates_gitignore_with_claude_block() {
    let dir = tempdir().expect("create tempdir");

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let gitignore = fs::read(dir.path().join(".gitignore")).expect("read .gitignore");
    assert_eq!(
        gitignore,
        b"# rex:ignore claude\n.claude/skills/\n.claude/settings.local.json\n"
    );
}

#[test]
fn existing_gitignore_gets_block_appended_once() {
    let dir = tempdir().expect("create tempdir");
    fs::write(dir.path().join(".gitignore"), b"/target").expect("write .gitignore");

    let first = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(first.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(stdout.contains("rex skills: .gitignore updated\n"));

    let after_first = fs::read(dir.path().join(".gitignore")).expect("read .gitignore");
    assert_eq!(
        after_first,
        b"/target\n\n# rex:ignore claude\n.claude/skills/\n.claude/settings.local.json\n"
    );

    let second = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(second.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(!stdout.contains(".gitignore"));

    let after_second = fs::read(dir.path().join(".gitignore")).expect("read .gitignore");
    assert_eq!(after_first, after_second);
}

#[test]
fn vendor_codex_gitignore_block_names_dot_codex() {
    let dir = tempdir().expect("create tempdir");

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .args(["skills", "--vendor", "codex"])
        .current_dir(dir.path())
        .assert()
        .success();

    let codex_only = fs::read(dir.path().join(".gitignore")).expect("read .gitignore");
    assert_eq!(codex_only, b"# rex:ignore codex\n.codex/skills/\n");

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .current_dir(dir.path())
        .assert()
        .success();

    let both = fs::read(dir.path().join(".gitignore")).expect("read .gitignore");
    assert_eq!(
        both,
        b"# rex:ignore codex\n.codex/skills/\n\n# rex:ignore claude\n.claude/skills/\n.claude/settings.local.json\n"
    );
}

/// Puts a fake `pre-commit` on PATH that records its arguments, so the test
/// checks what rex runs without needing the real tool.
fn fake_pre_commit(dir: &Path) -> (String, std::path::PathBuf) {
    let bin = dir.join("fakebin");
    fs::create_dir_all(&bin).expect("create fakebin");
    let args_file = bin.join("args");
    let script = format!(
        "#!/bin/sh\nprintf '%s' \"$*\" > '{}'\n",
        args_file.display()
    );
    let tool = bin.join("pre-commit");
    fs::write(&tool, script).expect("write fake pre-commit");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("chmod fake tool");
    }
    (format!("{}:/usr/bin:/bin", bin.display()), args_file)
}

#[test]
fn pre_commit_config_is_created_and_hooks_installed() {
    let dir = tempdir().expect("create tempdir");
    fs::create_dir_all(dir.path().join(".git/hooks")).expect("create .git/hooks");
    let (path_env, args_file) = fake_pre_commit(dir.path());

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", path_env)
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(stdout.ends_with(
        "rex skills: .pre-commit-config.yaml created\nrex skills: pre-commit hooks installed\n"
    ));

    let config = fs::read_to_string(dir.path().join(".pre-commit-config.yaml"))
        .expect("read .pre-commit-config.yaml");
    assert!(config.starts_with("# rex:pre-commit\n"));
    assert!(config.contains("repo: https://github.com/pre-commit/pre-commit-hooks\n"));
    assert!(config.contains("entry: gitleaks git --pre-commit --staged --redact --verbose\n"));
    assert!(
        config.contains("--scanners secret,vuln,misconfig --severity HIGH,CRITICAL --exit-code 1")
    );
    assert!(config.contains("stages: [pre-push]\n"));

    let args = fs::read_to_string(args_file).expect("fake pre-commit must have run");
    assert_eq!(args, "install --hook-type pre-commit --hook-type pre-push");
}

#[test]
fn existing_pre_commit_config_is_never_touched() {
    let dir = tempdir().expect("create tempdir");
    fs::create_dir_all(dir.path().join(".git/hooks")).expect("create .git/hooks");
    let (path_env, _) = fake_pre_commit(dir.path());
    fs::write(
        dir.path().join(".pre-commit-config.yaml"),
        b"sentinel content",
    )
    .expect("write sentinel config");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", path_env)
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(stdout.contains("rex skills: .pre-commit-config.yaml skipped (already exists)\n"));

    let config = fs::read(dir.path().join(".pre-commit-config.yaml")).expect("read config");
    assert_eq!(config, b"sentinel content");
}

#[test]
fn second_run_leaves_pre_commit_config_alone() {
    let dir = tempdir().expect("create tempdir");
    fs::create_dir_all(dir.path().join(".git/hooks")).expect("create .git/hooks");
    let (path_env, _) = fake_pre_commit(dir.path());

    Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", path_env.clone())
        .current_dir(dir.path())
        .assert()
        .success();

    let before = fs::read(dir.path().join(".pre-commit-config.yaml")).expect("read config");

    let second = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", path_env)
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(second.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(!stdout.contains(".pre-commit-config.yaml"));
    assert!(stdout.ends_with("rex skills: pre-commit hooks installed\n"));

    let after = fs::read(dir.path().join(".pre-commit-config.yaml")).expect("read config");
    assert_eq!(before, after);
}

#[test]
fn missing_pre_commit_tool_prints_the_install_command() {
    let dir = tempdir().expect("create tempdir");
    fs::create_dir_all(dir.path().join(".git/hooks")).expect("create .git/hooks");
    let empty_bin = dir.path().join("emptybin");
    fs::create_dir_all(&empty_bin).expect("create emptybin");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("skills")
        .env("PATH", empty_bin.as_os_str())
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert!(stdout.ends_with(
        "rex skills: pre-commit hooks skipped (pre-commit not found; install it, then run: pre-commit install --hook-type pre-commit --hook-type pre-push)\n"
    ));
    assert!(dir.path().join(".pre-commit-config.yaml").is_file());
}

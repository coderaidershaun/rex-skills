use std::fs;

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn smells_lists_each_flag_then_the_count() {
    let dir = tempdir().expect("create tempdir");

    fs::create_dir_all(dir.path().join("src/a")).expect("create src/a dir");
    fs::write(
        dir.path().join("src/a/deep.rs"),
        "fn deep() {\n    let total = 1; // SMELL: magic number\n}\n\n/// SMELL: does too much\nfn big() {}\n",
    )
    .expect("write src/a/deep.rs");
    fs::write(
        dir.path().join("src/b.rs"),
        "// SMELL: bad name\nfn b() {}\n",
    )
    .expect("write src/b.rs");
    fs::write(
        dir.path().join("src/clean.rs"),
        "// The SMELL: flag is a comment.\nconst FIXTURE: &str = \"// SMELL: in a string\";\n",
    )
    .expect("write src/clean.rs");

    fs::write(dir.path().join("notes.md"), "// SMELL: not a Rust file\n").expect("write notes.md");

    fs::create_dir_all(dir.path().join("target")).expect("create target dir");
    fs::write(dir.path().join("target/gen.rs"), "// SMELL: gitignored\n")
        .expect("write target/gen.rs");
    fs::write(dir.path().join(".gitignore"), "/target\n").expect("write .gitignore");

    fs::create_dir_all(dir.path().join(".hidden")).expect("create .hidden dir");
    fs::write(dir.path().join(".hidden/h.rs"), "// SMELL: hidden\n").expect("write .hidden/h.rs");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("smells")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert_eq!(
        stdout,
        "src/a/deep.rs:2\nsrc/a/deep.rs:5\nsrc/b.rs:1\nCode smells: 3\n"
    );
}

#[test]
fn smells_reports_zero_when_no_flag_exists() {
    let dir = tempdir().expect("create tempdir");
    fs::write(dir.path().join("main.rs"), "fn main() {}\n").expect("write main.rs");

    let assert = Command::cargo_bin("rex")
        .expect("find rex binary")
        .arg("smells")
        .current_dir(dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    assert_eq!(stdout, "Code smells: 0\n");
}

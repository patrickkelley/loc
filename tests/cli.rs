use assert_cmd::Command;
mod common;
use common::init_repo;
use std::path::Path;

fn run(dir: &Path, args: &[&str]) -> String {
    String::from_utf8(
        Command::cargo_bin("loc")
            .unwrap()
            .current_dir(dir)
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap()
}

#[test]
fn e2e_prints_table_with_total() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() {}\n// c\nlet s = \"http://x//no\";\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("app.py"),
        "import os\n# comment\nx = \"http://x#no\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("notes.txt"), "hello\n\nworld\n").unwrap();
    init_repo(dir.path());

    let out = run(dir.path(), &[]);
    let expected = concat!(
        "Language     Files  Lines\n",
        "other            1      2\n",
        "python           1      2\n",
        "rust             1      2\n",
        "Total            3      6\n",
    );
    assert_eq!(out, expected);
}

#[test]
fn files_flag_prints_per_file_breakdown() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.path().join("app.py"), "import os\n").unwrap();
    init_repo(dir.path());

    let out = run(dir.path(), &["--files"]);
    assert!(
        out.contains("main.rs"),
        "--files output should list files: {out}"
    );
    assert!(
        out.contains("app.py"),
        "--files output should list files: {out}"
    );
}

#[test]
fn version_flag_prints_version() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["--version"]);
    assert_eq!(out, format!("loc {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag_prints_usage() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["--help"]);
    assert!(out.contains("Usage:"), "help should show usage: {out}");
    assert!(
        out.contains("--files"),
        "help should document --files: {out}"
    );
}

#[test]
fn errors_with_nonzero_exit_outside_repo() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("loc")
        .unwrap()
        .current_dir(dir.path())
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a git repository"));
}

#[test]
fn empty_repo_prints_zero_total() {
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    let out = run(dir.path(), &[]);
    let expected = concat!("Language     Files  Lines\n", "Total            0      0\n",);
    assert_eq!(out, expected);
}

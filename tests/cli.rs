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
fn files_flag_sorts_by_path_and_aligns_long_paths() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a_script.sh"), "echo hi\n").unwrap();
    std::fs::write(dir.path().join("z_main.c"), "int main() { return 0; }\n").unwrap();
    let nested = dir
        .path()
        .join("very/long/nested/directory/path/for/testing");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("module.rs"), "fn a() {}\nfn b() {}\n").unwrap();
    init_repo(dir.path());

    let out = run(dir.path(), &["--files"]);
    let files_section = out.split("\nFiles:\n").nth(1).expect("Files section");
    let rows: Vec<&str> = files_section.lines().collect();
    assert_eq!(rows.len(), 3);

    // Must sort alphabetically by path, not by hidden language name ("c" < "rust" < "shell").
    assert!(rows[0].trim_start().starts_with("a_script.sh "));
    assert!(
        rows[1]
            .trim_start()
            .starts_with("very/long/nested/directory/path/for/testing/module.rs ")
    );
    assert!(rows[2].trim_start().starts_with("z_main.c "));

    // Every row in the Files section should right-align numbers to the same column width.
    let widths: Vec<usize> = rows.iter().map(|r| r.len()).collect();
    assert_eq!(widths[0], widths[1], "misaligned rows:\n{files_section}");
    assert_eq!(widths[1], widths[2], "misaligned rows:\n{files_section}");
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

#[test]
fn skips_deleted_files_symlinks_and_submodules() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("keep.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.path().join("deleted.rs"), "fn gone() {}\n").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        symlink("keep.rs", dir.path().join("dup.rs")).unwrap();
        symlink("nope.rs", dir.path().join("broken.rs")).unwrap();
        symlink("sub", dir.path().join("dir_link")).unwrap();
    }

    init_repo(dir.path());
    std::fs::remove_file(dir.path().join("deleted.rs")).unwrap();

    let out = run(dir.path(), &[]);
    let expected = concat!(
        "Language     Files  Lines\n",
        "rust             1      1\n",
        "Total            1      1\n",
    );
    assert_eq!(out, expected);
}

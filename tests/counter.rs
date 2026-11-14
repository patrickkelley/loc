use loc::counter::count_file;

fn count(src: &str) -> u64 {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("main.rs");
    std::fs::write(&path, src).unwrap();
    count_file(&path).unwrap().unwrap().lines
}

#[test]
fn rust_excludes_blank_and_comment_lines() {
    let src = "fn main() {\n    // comment\n    let s = 1;\n\n    /* block\n       spans */\n    let x = 5;\n}\n";
    // code lines: "fn main() {", "let s = 1;", "let x = 5;", "}"
    assert_eq!(count(src), 4);
}

#[test]
fn rust_string_with_comment_marker_is_code() {
    let src = "fn main() {\n    let s = \"http://x // not a comment\";\n}\n";
    assert_eq!(count(src), 3);
}

#[test]
fn rust_multiline_string_counts_each_line() {
    let src = "fn main() {\n    let s = r\"line one\nline two\";\n}\n";
    assert_eq!(count(src), 4);
}

#[test]
fn code_after_block_comment_on_same_line_counts() {
    let src = "/* c */ fn main() {}\n";
    assert_eq!(count(src), 1);
}

#[test]
fn binary_file_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("blob.rs");
    std::fs::write(&path, b"\x00\x01binary\xff").unwrap();
    assert!(count_file(&path).unwrap().is_none());
}

#[test]
fn unknown_extension_falls_back_to_nonblank_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.txt");
    std::fs::write(&path, "hello\n\n  \nworld\n").unwrap();
    let c = count_file(&path).unwrap().unwrap();
    assert_eq!(c.lines, 2);
    assert_eq!(c.language, "other");
}

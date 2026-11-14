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

#[test]
fn skips_missing_directory_and_symlink_entries() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        count_file(&dir.path().join("missing.rs"))
            .unwrap()
            .is_none()
    );
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    assert!(count_file(&sub).unwrap().is_none());

    #[cfg(unix)]
    {
        let target = dir.path().join("real.rs");
        std::fs::write(&target, "fn main() {}\n").unwrap();
        let link = dir.path().join("link.rs");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(count_file(&link).unwrap().is_none());
    }
}

#[test]
fn handles_files_larger_than_1kb_for_text_and_binary() {
    let dir = tempfile::tempdir().unwrap();

    let text_path = dir.path().join("large.rs");
    let text = "let value = 42;\n".repeat(200); // 3200 bytes > 1024
    std::fs::write(&text_path, &text).unwrap();
    let c = count_file(&text_path).unwrap().unwrap();
    assert_eq!(c.language, "rust");
    assert_eq!(c.lines, 200);

    let bin_path = dir.path().join("large_bin.rs");
    let mut bin = vec![b'a'; 4096];
    bin[512] = 0;
    std::fs::write(&bin_path, &bin).unwrap();
    assert!(count_file(&bin_path).unwrap().is_none());
}

#[test]
fn handles_utf8_bom_and_unicode_whitespace() {
    let dir = tempfile::tempdir().unwrap();

    let rs_empty = dir.path().join("bom_comment.rs");
    std::fs::write(&rs_empty, "\u{feff}// comment only\n\u{00a0}\t\n").unwrap();
    assert_eq!(count_file(&rs_empty).unwrap().unwrap().lines, 0);

    let rs_code = dir.path().join("bom_code.rs");
    std::fs::write(&rs_code, "\u{feff}fn main() {}\n").unwrap();
    assert_eq!(count_file(&rs_code).unwrap().unwrap().lines, 1);

    let txt_empty = dir.path().join("bom_blank.txt");
    std::fs::write(&txt_empty, "\u{feff}\n\u{00a0}\n").unwrap();
    assert_eq!(count_file(&txt_empty).unwrap().unwrap().lines, 0);
}

#[test]
fn counts_heavily_commented_file_quickly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("many_comments.rs");
    let mut src = String::new();
    for i in 0..2_500 {
        src.push_str(&format!("// line comment {i}\n"));
        src.push_str(&format!("let v_{i} = {i};\n"));
        src.push_str(&format!("/* block comment {i} */\n"));
        src.push_str(&format!("fn f_{i}() -> i32 {{ {i} }}\n"));
    }
    std::fs::write(&path, &src).unwrap();

    let start = std::time::Instant::now();
    let c = count_file(&path).unwrap().unwrap();
    let elapsed = start.elapsed();

    assert_eq!(c.lines, 5_000);
    assert!(
        elapsed < std::time::Duration::from_secs(2),
        "counting 10k lines / 5k comments took too long: {elapsed:?}"
    );
}

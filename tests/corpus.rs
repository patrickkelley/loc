//! Curated per-language corpus tests: tricky constructs, hand-checked against
//! the counting semantics (a counted line = any non-blank line with a
//! character outside a comment range; comment ranges come from tree-sitter
//! node kinds containing "comment", plus `hash_bang_line` for JS/TS).
use loc::counter::count_file;

fn count(src: &str, name: &str) -> (u64, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, src).unwrap();
    let c = count_file(&path).unwrap().unwrap();
    (c.lines, c.language)
}

fn assert_counts(name: &str, src: &str, lang: &str, expected: u64) {
    let (lines, language) = count(src, name);
    assert_eq!(language, lang, "language mismatch for {name}");
    assert_eq!(lines, expected, "line count mismatch for {name}:\n{src}");
}

#[test]
fn gosu_page_template_comments_are_excluded() {
    // .gsp is a Gosu *page* (template syntax), so `<%-- ... --%>` template
    // comments must be excluded exactly like in .gst templates.
    assert_counts(
        "P.gsp",
        "<%@ params(name: String) %>\n<%-- template\n     comment --%>\nHello ${name}\n",
        "gosu",
        2, // `<%@ params ... %>` + `Hello ${name}`
    );
}

#[test]
fn gosu_page_scriptlet_lines_count() {
    assert_counts(
        "Q.gsp",
        "<%\n   // scriptlet comment\n   print(\"http://x\")\n%>\n",
        "gosu",
        3, // `<%`, print line, `%>`; scriptlet comment excluded
    );
}

#[test]
fn js_shebang_line_is_excluded() {
    // A `#!` shebang is not code (same treatment as shell scripts).
    assert_counts(
        "bin.js",
        "#!/usr/bin/env node\nconsole.log(\"hi\");\n",
        "javascript",
        1, // only the console.log line
    );
}

#[test]
fn rust_doc_comments_excluded() {
    // `///` doc comments and `//!` inner doc comments are line_comment /
    // doc_comment nodes in tree-sitter-rust (kinds containing "comment").
    assert_counts(
        "d.rs",
        "/// doc\n//! inner\nfn main() { }\n",
        "rust",
        1, // only the fn line; `///` and `//!` are comment nodes
    );
}

#[test]
fn rust_nested_block_comments_excluded() {
    // tree-sitter-rust spans nested block comments in one block_comment node
    // (unlike C, whose block comments do not nest).
    assert_counts(
        "m.rs",
        "/* a /* b */ still comment */\nlet x = 1;\n",
        "rust",
        1, // the whole first line is one block_comment; only `let x = 1;` counts
    );
}

#[test]
fn rust_raw_string_with_hashes() {
    // Raw strings `r#"..."#` / `r##"..."##` keep `//`, `#` and quotes as
    // string content; parsing must not break and no comment range appears.
    assert_counts(
        "r.rs",
        "const s: &str = r#\"http://x # y\"#;\nconst t: &str = r##\"has \" inside\"##;\n",
        "rust",
        2, // both const lines count; hash-quoted raw strings parse cleanly
    );
}

#[test]
fn py_triple_single_quoted_multiline_string() {
    // Triple-quoted string content lines are code; a `#` inside the string
    // is string content, not a comment.
    assert_counts(
        "a.py",
        "t = '''line\n# inside'''\nx = 1\n",
        "python",
        3, // all three lines have non-whitespace chars outside comment ranges
    );
}

#[test]
fn py_fstring_hash_inside() {
    // A `#` inside an f-string is string content, not a comment start.
    assert_counts(
        "b.py",
        "x = f\"# {y}\"\ny = 1\n",
        "python",
        2, // both lines count; the `#` lives inside the string node
    );
}

#[test]
fn py_raw_string_hash() {
    // The raw prefix `r` changes escaping only; `#` inside the string is
    // still string content.
    assert_counts(
        "c.py",
        "s = r\"# no\"\nx = 1\n",
        "python",
        2, // both lines count
    );
}

#[test]
fn py_line_continuation_counts_both_lines() {
    // A backslash line continuation is not a comment range: both physical
    // lines are non-blank code.
    assert_counts(
        "d.py",
        "x = 1 + \\\n  2\n",
        "python",
        2, // continuation lines are ordinary code lines
    );
}

#[test]
fn js_regex_literal_with_slashes() {
    // A regex literal is one token: the `//` inside it (and inside a string)
    // is not a comment start; only the real `// comment` line is excluded.
    assert_counts(
        "r.js",
        "const re = /https?:\\/\\//;\nconst s = \"//\";\n// comment\n",
        "javascript",
        2, // regex line + string line; the trailing `// comment` is excluded
    );
}

#[test]
fn js_template_multiline_with_markers() {
    // template_string content is counted per line: `//` inside a template
    // literal is not a comment.
    assert_counts(
        "t.js",
        "const t = `line // inside\n${1 + 2} // after`;\nconst u = 1;\n",
        "javascript",
        3, // both template content lines + the `const u` line
    );
}

#[test]
fn ts_shebang_line_is_excluded() {
    // TypeScript files get the same hash_bang_line treatment as JavaScript.
    assert_counts(
        "a.ts",
        "#!/usr/bin/env node\nconst x = 1;\n",
        "typescript",
        1, // only the const line; the hash_bang_line node is excluded
    );
}

#[test]
fn jsx_comment_in_expression_is_excluded() {
    // tree-sitter-javascript parses JSX natively: `{/* comment */}` is a
    // comment node inside the jsx_expression and is excluded, while JSX text
    // lines are ordinary code.
    assert_counts(
        "App.jsx",
        "export function App() {\n  return (\n    <div>\n      {/* comment */}\n      Hello\n    </div>\n  );\n}\n",
        "javascript",
        8, // every line except the `{/* comment */}` line
    );
}

#[test]
fn go_backtick_raw_string_multiline() {
    // Go raw string (backtick) content is counted per line: `//` and `#`
    // inside backticks are not comments.
    assert_counts(
        "m.go",
        "package main\nconst s = `// raw\n# hash`\nfunc f() { }\n",
        "go",
        4, // all four lines; both raw-string interior lines count as code
    );
}

#[test]
fn c_line_splice_continues_comment() {
    // tree-sitter-c splices a backslash-continued line into the same
    // line_comment node, so "still comment" stays inside the comment range.
    assert_counts(
        "m.c",
        "// comment \\\nstill comment\nint main() { return 0; }\n",
        "c",
        1, // only the int main line; both comment lines are one comment node
    );
}

#[test]
fn c_block_comment_in_string_not_comment() {
    // `/* ... */` inside a string literal is string content; the real block
    // comment spans its own two lines.
    assert_counts(
        "m2.c",
        "const char* s = \"/* not a comment */\";\n/* block\n   span */\nint main() { return 0; }\n",
        "c",
        2, // const line + int main line; the block-comment lines are excluded
    );
}

#[test]
fn c_block_comments_do_not_nest() {
    // C block comments do not nest: the first `*/` ends the comment and the
    // rest of the line is code (contrast with Rust's nested comments).
    assert_counts(
        "m4.c",
        "/* a /* b */ still code */\nint main() { return 0; }\n",
        "c",
        2, // the `still code */` tail is code, so line 1 counts, as does line 2
    );
}

#[test]
fn cpp_raw_string_multiline() {
    // C++ raw string R"(...)" content is counted per line: `//` and `/*`
    // inside it are string content, not comments.
    assert_counts(
        "m.cpp",
        "const char* s = R\"(// raw\n/* not comment */\")\";\nint main() { return 0; }\n",
        "cpp",
        3, // both raw-string interior lines + the int main line
    );
}

#[test]
fn cpp_line_splice_continues_comment() {
    // Same splice rule as C: the backslash-continued line joins the same
    // line_comment node.
    assert_counts(
        "m3.cpp",
        "// comment \\\nstill comment\nint main() { return 0; }\n",
        "cpp",
        1, // only the int main line
    );
}

#[test]
fn java_javadoc_excluded() {
    // Javadoc is a block_comment node spanning all three doc lines.
    assert_counts(
        "B.java",
        "/** doc\n * line\n */\nclass B { }\n",
        "java",
        1, // only the class line
    );
}

#[test]
fn java_text_block_lines_count() {
    // Java text block (\"\"\") content is string: the `// not comment` line
    // counts as a code line.
    assert_counts(
        "A.java",
        "class A {\n  String s = \"\"\"\n    // not comment\n    \"\"\";\n}\n",
        "java",
        5, // all five lines; the text-block interior is string content
    );
}

#[test]
fn gosu_string_enhancement() {
    // ${...} string enhancement does not affect comment detection: the `//`
    // line is a real comment, the other two lines count.
    assert_counts(
        "e.gs",
        "var s = \"Hello ${name}\"\n// real comment\nprint(s)\n",
        "gosu",
        2, // var line + print line; the `//` comment line is excluded
    );
}

#[test]
fn gst_scriptlet_delimiters_count_with_only_comment() {
    // Characterization: the `<%`/`%>` delimiter chars lie outside the
    // line_comment range, so a scriptlet containing only a comment still
    // counts (non-blank, non-comment characters on the line).
    assert_counts(
        "T.gst",
        "<% // c %>\nHello\n<% print(\"x\") %>\n",
        "gosu",
        3, // scriptlet delimiters count even when the scriptlet is only a comment
    );
}

#[test]
fn ruby_begin_end_block_comments_excluded() {
    // tree-sitter-ruby emits a comment node spanning =begin..=end.
    assert_counts(
        "a.rb",
        "=begin\nblock comment\n=end\nputs 1\n",
        "ruby",
        1, // only the puts line; the =begin block is one comment node
    );
}

#[test]
fn ruby_heredoc_lines_count() {
    // Characterization, PRD-consistent: heredoc content is string, hence
    // code; the `EOS` terminator line is neither blank nor a comment, so it
    // also counts.
    assert_counts(
        "b.rb",
        "text = <<~EOS\n# not comment\nEOS\nputs text\n",
        "ruby",
        4, // assignment, heredoc interior, EOS terminator, puts
    );
}

#[test]
fn ruby_regex_hash_not_comment() {
    // `#` inside a regex literal and inside a string is not a comment start.
    assert_counts(
        "c.rb",
        "puts \"x\" if /#\\d/ =~ \"a#1\"\n",
        "ruby",
        1, // the single line counts
    );
}

#[test]
fn bash_heredoc_lines_count() {
    // Same characterization as the Ruby heredoc: heredoc content and the
    // `EOF` terminator line are code.
    assert_counts(
        "s.sh",
        "cat <<EOF\n# not comment\nEOF\necho done\n",
        "shell",
        4, // cat, heredoc interior, EOF terminator, echo
    );
}

#[test]
fn bash_hash_in_word_not_comment() {
    // In bash a comment starts with `#` at a word boundary: `a#b` is an
    // ordinary word, not a comment.
    assert_counts(
        "s2.sh",
        "echo a#b\n# real comment\necho done\n",
        "shell",
        2, // the echo lines count; the `#` comment line is excluded
    );
}

#[test]
fn bash_comment_inside_command_substitution() {
    // A comment node inside command_substitution is still a comment: the
    // `# comment` text is excluded while the surrounding shell code counts.
    assert_counts(
        "s3.sh",
        "x=$( # comment\n  echo hi )\necho $x\n",
        "shell",
        3, // x=$(, echo hi ), echo $x — the `# comment` text is excluded
    );
}

/// Uniform guard table: (file, code line, comment line) covering every
/// grammar variant. The comment line uses each language's real comment
/// syntax (`<%-- --%>` for the Gosu template variants).
const GUARDS: [(&str, &str, &str); 14] = [
    ("a.rs", "fn main() { }", "// c"),
    ("a.py", "x = 1", "# c"),
    ("a.js", "const x = 1;", "// c"),
    ("a.ts", "const x = 1;", "// c"),
    ("a.tsx", "const x = 1;", "// c"),
    ("a.go", "package main", "// c"),
    ("a.c", "int main() { return 0; }", "// c"),
    ("a.cpp", "int main() { return 0; }", "// c"),
    ("a.java", "class A { }", "// c"),
    ("a.gs", "var x = 1", "// c"),
    ("a.gsp", "Hello", "<%-- c --%>"),
    ("a.gst", "Hello", "<%-- c --%>"),
    ("a.rb", "x = 1", "# c"),
    ("a.sh", "echo hi", "# c"),
];

#[test]
fn comment_only_file_counts_zero_for_every_language() {
    // Guard for the AGENTS.md gotcha: comment exclusion relies on
    // tree-sitter node kinds whose name contains "comment". A grammar that
    // names its comment node differently would silently count comments as
    // code and fail here.
    for (file, _, comment) in GUARDS {
        let src = format!("{comment}\n");
        let (lines, _) = count(&src, file);
        assert_eq!(lines, 0, "comment-only {file} should count 0 lines:\n{src}");
    }
}

#[test]
fn bom_comment_only_counts_zero_for_every_language() {
    // The BOM char (`\u{feff}`) is skipped when counting, so a BOM-prefixed
    // comment-only file still counts 0 lines for every grammar.
    for (file, _, comment) in GUARDS {
        let src = format!("\u{feff}{comment}\n");
        let (lines, _) = count(&src, file);
        assert_eq!(
            lines, 0,
            "BOM comment-only {file} should count 0 lines:\n{src}"
        );
    }
}

#[test]
fn crlf_line_endings_count_code_lines_only() {
    // CRLF line endings must not merge the code line with the comment line:
    // only the code line counts, for every grammar.
    for (file, code, comment) in GUARDS {
        let src = format!("{code}\r\n{comment}\r\n");
        let (lines, _) = count(&src, file);
        assert_eq!(
            lines, 1,
            "CRLF {file} should count only the code line:\n{src}"
        );
    }
}

#[test]
fn missing_trailing_newline_counts_last_code_line() {
    // A final line without a trailing newline is still a counted line.
    for (file, code, _) in GUARDS {
        let (lines, _) = count(code, file);
        assert_eq!(
            lines, 1,
            "no-trailing-newline {file} should count the last line"
        );
    }
}

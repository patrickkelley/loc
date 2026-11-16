use loc::counter::count_file;

fn count_in(dir: &tempfile::TempDir, name: &str, src: &str) -> (u64, String) {
    let path = dir.path().join(name);
    std::fs::write(&path, src).unwrap();
    let c = count_file(&path).unwrap().unwrap();
    (c.lines, c.language)
}

macro_rules! lang_case {
    ($name:ident, $file:expr, $src:expr, $expected:expr, $lang:expr) => {
        #[test]
        fn $name() {
            let dir = tempfile::tempdir().unwrap();
            let (lines, language) = count_in(&dir, $file, $src);
            assert_eq!(lines, $expected);
            assert_eq!(language, $lang);
        }
    };
}

lang_case!(
    python_comments_and_docstrings,
    "m.py",
    "import os\n# comment\n\ndef f():\n    \"\"\"doc\n    string\"\"\"\n    url = \"http://x#no\"\n",
    5,
    "python"
);
// code lines: import, def f():, doc (non-blank), string line, url line = 5

lang_case!(
    bash_comments,
    "s.sh",
    "#!/bin/sh\n# comment\n    echo hello\nurl=\"http://x#no\"\n",
    2,
    "shell"
);

lang_case!(
    java_comments,
    "A.java",
    "class A {\n    // note\n    String s = \"http://x//no\";\n    /* block\n       span */\n}\n",
    3,
    "java"
);

lang_case!(
    ruby_comments,
    "a.rb",
    "# comment\ndef f\n  x = \"http://x#no\"\nend\n",
    3,
    "ruby"
);

lang_case!(
    javascript_comments,
    "a.js",
    "// comment\nlet s = \"http://x//no\";\n/* block\n   span */\nlet t = 1;\n",
    2,
    "javascript"
);

lang_case!(
    go_comments,
    "m.go",
    "package main\n// comment\nfunc main() {\n\t/* block */\n}\n",
    3,
    "go"
);

lang_case!(
    typescript_comments,
    "a.ts",
    "// comment\nconst s: string = \"http://x//no\";\n",
    1,
    "typescript"
);

lang_case!(
    tsx_jsx_comments_and_literal_text,
    "App.tsx",
    "export function Help() {\n  // line comment\n  return (\n    <pre>\n      {\n        /* comment inside jsx expression */\n        value\n      }\n      // literal text in JSX\n      /* also literal text */\n    </pre>\n  );\n}\n",
    11,
    "typescript"
);

lang_case!(
    c_comments,
    "m.c",
    "#include <stdio.h>\n// comment\nint main() { return 0; }\n",
    2,
    "c"
);

lang_case!(
    cpp_comments,
    "m.cpp",
    "// comment\nint main() { return 0; }\n",
    1,
    "cpp"
);

lang_case!(
    gosu_comments,
    "A.gs",
    "class A {\n    // note\n    function greet() {\n        var s = \"http://x//no\"\n    }\n    /* block\n       span */\n}\n",
    5,
    "gosu"
);

lang_case!(
    gosu_template_comments,
    "T.gst",
    "<%@ params(name: String) %>\n<%-- template\n     comment --%>\nHello ${name}\n<%\n   // scriptlet comment\n   print(\"http://x//no\")\n%>\n",
    5,
    "gosu"
);

/// Guards the extension set against drift from the PRD R3 table: every
/// documented extension must map, through the real `count_file` path, to its
/// documented language name and count its single code line. The source
/// "x\n" parses with errors for some grammars, but counting is text-based
/// and always yields 1.
#[test]
fn prd_r3_extension_matrix_maps_every_extension_to_language() {
    const R3: &[(&str, &str)] = &[
        ("rs", "rust"),
        ("py", "python"),
        ("js", "javascript"),
        ("mjs", "javascript"),
        ("cjs", "javascript"),
        ("jsx", "javascript"),
        ("ts", "typescript"),
        ("tsx", "typescript"),
        ("go", "go"),
        ("c", "c"),
        ("h", "c"),
        ("cpp", "cpp"),
        ("cc", "cpp"),
        ("cxx", "cpp"),
        ("hpp", "cpp"),
        ("hh", "cpp"),
        ("hxx", "cpp"),
        ("java", "java"),
        ("gs", "gosu"),
        ("gsp", "gosu"),
        ("gsx", "gosu"),
        ("gst", "gosu"),
        ("rb", "ruby"),
        ("sh", "shell"),
        ("bash", "shell"),
    ];
    let dir = tempfile::tempdir().unwrap();
    for &(ext, expected) in R3 {
        let (lines, language) = count_in(&dir, &format!("f.{ext}"), "x\n");
        assert_eq!(
            language, expected,
            "extension .{ext} should map to language {expected:?}"
        );
        assert_eq!(
            lines, 1,
            "extension .{ext} should count 1 line, got {lines}"
        );
    }
}

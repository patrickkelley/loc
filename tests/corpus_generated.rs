//! Artificial corpus: files composed from self-contained atoms with known
//! code-line contributions, shuffled with a deterministic PRNG. The expected
//! count is the sum of the chosen atoms' contributions (the oracle).
use loc::counter::count_file;

/// xorshift32 — deterministic, dependency-free PRNG.
fn next(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

/// Self-contained snippet: balanced quotes/comment delimiters, top-level valid
/// for its grammar. `{n}` is replaced by an incrementing counter to keep
/// identifiers unique. `code_lines` is the oracle contribution.
struct Atom {
    tpl: &'static str,
    code_lines: u64,
}

struct LangSpec {
    ext: &'static str,
    lang: &'static str,
    /// atoms[0] must be a 0-contribution comment atom (padding for `Big`).
    atoms: &'static [Atom],
}

/// File-shape variations layered on top of a Plain composition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    Plain,
    Crlf,
    NoTrailingNewline,
    Bom,
    /// Padded with 0-contribution comment atoms past the 1024-byte threshold
    /// where `count_file` switches to its two-phase read; oracle unchanged.
    Big,
}

/// Shared by javascript, typescript and tsx (same grammar family).
const WEB_ATOMS: &[Atom] = &[
    Atom {
        tpl: "// comment {n}",
        code_lines: 0,
    },
    Atom {
        tpl: "/* block {n} */",
        code_lines: 0,
    },
    Atom {
        tpl: "let v{n} = {n};",
        code_lines: 1,
    },
    Atom {
        tpl: "let s{n} = \"http://x/{n}\";",
        code_lines: 1,
    },
    Atom {
        tpl: "const t{n} = `tpl {n} // inside`;",
        code_lines: 1,
    },
    Atom {
        tpl: "",
        code_lines: 0,
    },
];

/// Gosu pages (.gsp) and templates (.gst) share the template grammar. The
/// `<%`/`%>` delimiter characters are code, so `<%-- ... --%>` is the only
/// usable comment atom.
const GOSU_TEMPLATE_ATOMS: &[Atom] = &[
    Atom {
        tpl: "<%-- template comment {n} --%>",
        code_lines: 0,
    },
    Atom {
        tpl: "<% var v{n} = {n} %>",
        code_lines: 1,
    },
    Atom {
        tpl: "<%= {n} %>",
        code_lines: 1,
    },
    Atom {
        tpl: "Hello {n}",
        code_lines: 1,
    },
    Atom {
        tpl: "",
        code_lines: 0,
    },
];

const SPECS: &[LangSpec] = &[
    LangSpec {
        ext: "rs",
        lang: "rust",
        atoms: &[
            Atom {
                tpl: "// line comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "/* block comment {n} */",
                code_lines: 0,
            },
            Atom {
                tpl: "static V{n}: u64 = {n};",
                code_lines: 1,
            },
            Atom {
                tpl: "const S{n}: &str = \"http://x/{n}\";",
                code_lines: 1,
            },
            Atom {
                tpl: "fn f{n}() -> u64 { {n} }",
                code_lines: 1,
            },
            Atom {
                tpl: "const R{n}: &str = r#\"raw {n} // inside\"#;",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "py",
        lang: "python",
        atoms: &[
            Atom {
                tpl: "# comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "x{n} = {n}",
                code_lines: 1,
            },
            Atom {
                tpl: "s{n} = \"http://x/{n}\"",
                code_lines: 1,
            },
            Atom {
                tpl: "t{n} = '''doc {n} # inside'''",
                code_lines: 1,
            },
            Atom {
                tpl: "f{n} = f\"# val {n}\"",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "js",
        lang: "javascript",
        atoms: WEB_ATOMS,
    },
    LangSpec {
        ext: "ts",
        lang: "typescript",
        atoms: WEB_ATOMS,
    },
    LangSpec {
        ext: "tsx",
        lang: "typescript",
        atoms: WEB_ATOMS,
    },
    LangSpec {
        ext: "go",
        lang: "go",
        atoms: &[
            Atom {
                tpl: "// comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "/* block {n} */",
                code_lines: 0,
            },
            Atom {
                tpl: "var v{n} = {n}",
                code_lines: 1,
            },
            Atom {
                tpl: "var s{n} = \"http://x/{n}\"",
                code_lines: 1,
            },
            Atom {
                tpl: "var r{n} = `raw {n} // inside`",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "c",
        lang: "c",
        atoms: &[
            Atom {
                tpl: "// comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "/* block {n} */",
                code_lines: 0,
            },
            Atom {
                tpl: "int v{n} = {n};",
                code_lines: 1,
            },
            Atom {
                tpl: "const char *s{n} = \"http://x/{n}\";",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "cpp",
        lang: "cpp",
        atoms: &[
            Atom {
                tpl: "// comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "/* block {n} */",
                code_lines: 0,
            },
            Atom {
                tpl: "int v{n} = {n};",
                code_lines: 1,
            },
            Atom {
                tpl: "const char *s{n} = \"http://x/{n}\";",
                code_lines: 1,
            },
            Atom {
                tpl: "auto r{n} = R\"(raw {n} // inside)\";",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "java",
        lang: "java",
        atoms: &[
            Atom {
                tpl: "// comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "/* block {n} */",
                code_lines: 0,
            },
            Atom {
                tpl: "class C{n} { int v{n} = {n}; }",
                code_lines: 1,
            },
            Atom {
                tpl: "class S{n} { String s{n} = \"http://x/{n}\"; }",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "gs",
        lang: "gosu",
        atoms: &[
            Atom {
                tpl: "// comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "/* block {n} */",
                code_lines: 0,
            },
            Atom {
                tpl: "var v{n} = {n}",
                code_lines: 1,
            },
            Atom {
                tpl: "var s{n} = \"http://x/{n}\"",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "gsp",
        lang: "gosu",
        atoms: GOSU_TEMPLATE_ATOMS,
    },
    LangSpec {
        ext: "gst",
        lang: "gosu",
        atoms: GOSU_TEMPLATE_ATOMS,
    },
    LangSpec {
        ext: "rb",
        lang: "ruby",
        atoms: &[
            Atom {
                tpl: "# comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "v{n} = {n}",
                code_lines: 1,
            },
            Atom {
                tpl: "s{n} = \"http://x/{n}\"",
                code_lines: 1,
            },
            // Squiggly heredoc: opener line, content line (`# inside` lives in
            // the string, not in a comment) and terminator line all count.
            Atom {
                tpl: "t{n} = <<~E{n}\ntext {n} # inside\nE{n}",
                code_lines: 3,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
    LangSpec {
        ext: "sh",
        lang: "shell",
        atoms: &[
            Atom {
                tpl: "# comment {n}",
                code_lines: 0,
            },
            Atom {
                tpl: "v{n}={n}",
                code_lines: 1,
            },
            Atom {
                tpl: "s{n}=\"http://x/{n}\"",
                code_lines: 1,
            },
            Atom {
                tpl: "",
                code_lines: 0,
            },
        ],
    },
];

fn render(atom: &Atom, counter: u32) -> String {
    atom.tpl.replace("{n}", &counter.to_string())
}

/// Compose a file by PRNG-shuffling `n_atoms` atom picks; returns the source
/// together with the oracle (the summed code-line contributions).
fn build(spec: &LangSpec, seed: u32, n_atoms: usize, variant: Variant) -> (String, u64) {
    let mut state = seed.wrapping_mul(2654435761) | 1; // xorshift needs a nonzero state
    let mut counter = 1u32;
    let mut expected = 0u64;
    let mut rendered: Vec<String> = Vec::new();
    for _ in 0..n_atoms {
        let atom = &spec.atoms[(next(&mut state) as usize) % spec.atoms.len()];
        rendered.push(render(atom, counter));
        counter += 1;
        expected += atom.code_lines;
    }
    if variant == Variant::Big {
        // Append 0-contribution comment atoms (still consuming counter values,
        // keeping `{n}` unique) until the file is comfortably past the 1024-byte
        // two-phase-read threshold.
        let mut len: usize = rendered.iter().map(|s| s.len() + 1).sum();
        while len < 1500 {
            let s = render(&spec.atoms[0], counter);
            counter += 1;
            len += s.len() + 1;
            rendered.push(s);
        }
    }
    let mut src = rendered.join("\n");
    src.push('\n');
    match variant {
        Variant::Plain | Variant::Big => {}
        Variant::Crlf => src = src.replace('\n', "\r\n"),
        Variant::NoTrailingNewline => {
            src.pop(); // drop the single trailing '\n' just appended
        }
        Variant::Bom => src.insert(0, '\u{feff}'),
    }
    (src, expected)
}

#[test]
fn generated_corpus_matches_oracle() {
    let dir = tempfile::tempdir().unwrap();
    for (lang_idx, spec) in SPECS.iter().enumerate() {
        for seed in 1..=14u32 {
            let (variant, tag) = match seed {
                1..=10 => (Variant::Plain, ""),
                11 => (Variant::Crlf, "_crlf"),
                12 => (Variant::NoTrailingNewline, "_notrail"),
                13 => (Variant::Bom, "_bom"),
                _ => (Variant::Big, "_big"),
            };
            let n_atoms = 8 + (seed as usize * 7 % 26);
            let (src, expected) = build(spec, seed, n_atoms, variant);
            let name = format!("gen_{lang_idx}_s{seed}{tag}.{}", spec.ext);
            let path = dir.path().join(&name);
            std::fs::write(&path, &src).unwrap();
            let c = count_file(&path).unwrap().unwrap();
            assert_eq!(
                c.language, spec.lang,
                "language mismatch in {name} (seed {seed}, variant {variant:?}, ext {})",
                spec.ext
            );
            assert_eq!(
                c.lines, expected,
                "oracle mismatch in {name} (seed {seed}, variant {variant:?}, ext {}, \
                 n_atoms {n_atoms})\nsource:\n{src}",
                spec.ext
            );
            if variant == Variant::Big {
                assert!(
                    src.len() > 1024,
                    "Big file {name} is only {} bytes",
                    src.len()
                );
            }
        }
    }
}

#[test]
fn generated_corpus_comment_only_is_zero() {
    let dir = tempfile::tempdir().unwrap();
    for (lang_idx, spec) in SPECS.iter().enumerate() {
        let comment = &spec.atoms[0]; // invariant: a 0-contribution comment atom
        let blank = spec
            .atoms
            .iter()
            .find(|a| a.tpl.is_empty())
            .expect("every language defines a blank atom");
        assert_eq!(
            comment.code_lines, 0,
            "atoms[0] must be a 0-contribution comment atom (ext {})",
            spec.ext
        );
        assert_eq!(
            blank.code_lines, 0,
            "blank atom must contribute 0 (ext {})",
            spec.ext
        );
        // 6 comment atoms + 3 blanks, order decided by the seeded PRNG (seed 7);
        // exactly 6/3 by construction, so every language exercises comment
        // exclusion deterministically.
        let mut state = 7u32.wrapping_mul(2654435761) | 1;
        let mut counter = 1u32;
        let mut expected = 0u64; // oracle: sum of contributions, all of them zero
        let mut rendered: Vec<String> = Vec::new();
        let mut comments_left = 6usize;
        let mut blanks_left = 3usize;
        while comments_left + blanks_left > 0 {
            let use_comment =
                comments_left > 0 && (blanks_left == 0 || next(&mut state).is_multiple_of(2));
            let atom = if use_comment { comment } else { blank };
            rendered.push(render(atom, counter));
            counter += 1;
            expected += atom.code_lines;
            if use_comment {
                comments_left -= 1;
            } else {
                blanks_left -= 1;
            }
        }
        let mut src = rendered.join("\n");
        src.push('\n');
        let name = format!("gen_{lang_idx}_commentonly.{}", spec.ext);
        let path = dir.path().join(&name);
        std::fs::write(&path, &src).unwrap();
        let c = count_file(&path).unwrap().unwrap();
        assert_eq!(c.language, spec.lang, "language mismatch in {name}");
        assert_eq!(expected, 0, "comment-only oracle must be 0 ({name})");
        assert_eq!(
            c.lines, 0,
            "comment-only file must count 0 ({name}):\n{src}"
        );
    }
}

use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
    Go,
    Gosu,
    C,
    Cpp,
    Java,
    Ruby,
    Bash,
}

impl Lang {
    pub fn from_path(path: &Path) -> Option<Lang> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "rs" => Some(Lang::Rust),
            "py" => Some(Lang::Python),
            "js" | "mjs" | "cjs" | "jsx" => Some(Lang::JavaScript),
            "ts" => Some(Lang::TypeScript),
            "tsx" => Some(Lang::Tsx),
            "go" => Some(Lang::Go),
            "gs" | "gsp" | "gsx" => Some(Lang::Gosu),
            "c" | "h" => Some(Lang::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some(Lang::Cpp),
            "java" => Some(Lang::Java),
            "rb" => Some(Lang::Ruby),
            "sh" | "bash" => Some(Lang::Bash),
            _ => None,
        }
    }

    pub fn ts_language(self) -> tree_sitter::Language {
        match self {
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
            Lang::Python => tree_sitter_python::LANGUAGE.into(),
            Lang::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::Go => tree_sitter_go::LANGUAGE.into(),
            Lang::Gosu => tree_sitter_gosu::LANGUAGE.into(),
            Lang::C => tree_sitter_c::LANGUAGE.into(),
            Lang::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Lang::Java => tree_sitter_java::LANGUAGE.into(),
            Lang::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            Lang::Bash => tree_sitter_bash::LANGUAGE.into(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript | Lang::Tsx => "typescript",
            Lang::Go => "go",
            Lang::Gosu => "gosu",
            Lang::C => "c",
            Lang::Cpp => "cpp",
            Lang::Java => "java",
            Lang::Ruby => "ruby",
            Lang::Bash => "shell",
        }
    }
}

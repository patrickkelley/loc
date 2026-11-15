# AGENTS.md

## What this is

`loc` — Rust CLI that counts non-blank, non-comment lines of code in a git repo using tree-sitter. The spec is `docs/PRD.md` and is authoritative for scope; it mandates strict TDD (failing test first) for every unit of work.

## Verification (CI gate — run all three in this order before declaring done)

```
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
```

Note the flags: clippy needs `--all-targets` (lints test code too) and test needs `--all-features`, per `.github/workflows/ci.yml`. Single test: `cargo test --test languages python_comments` (file + name substring).

Integration tests are split by concern: `tests/git.rs` (repo discovery), `tests/counter.rs` (blank/comment/string/binary/fallback), `tests/languages.rs` (one fixture per language), `tests/cli.rs` (end-to-end via `assert_cmd`).

## Gotchas

- Integration and counter tests spawn real `git` and write temp dirs (`tempfile`); git must be on PATH. `tests/common/mod.rs` holds the shared `git init` fixture used by `tests/git.rs` and `tests/cli.rs` — reuse it, don't re-create one.
- `tests/cli.rs` runs the built binary with `current_dir` pointed at a temp repo, so tests count the fixture repo, not this repo.
- Comment exclusion relies on tree-sitter node kinds whose name contains `comment` (`src/counter.rs`). A grammar whose comment node is named differently (e.g. `line_comment`) would silently count comments as code — always add a fixture test for a new language.
- Adding a language touches: `Cargo.toml` dep, `Lang` variant, extension map, `ts_language()`, `name()` in `src/langs.rs`, plus a `lang_case!` test. The extension set must match the PRD's R3 table.
- Binary detection: a UTF-16 BOM (`FF FE`/`FE FF`), or a NUL byte in the first 1024 bytes when the first KB does not decode as valid UTF-8 (a text file embedding a NUL, e.g. in a string literal, is counted as text). Skipped files return `None`, not an error. Unknown extensions report as language `other` with non-blank-line counts — that is fallback behavior, not a bug.

## Release

Pushing a `v*` tag triggers `.github/workflows/release.yml`, which builds release tarballs for 4 targets (linux/windows/macos×2). There is no crates.io publish.

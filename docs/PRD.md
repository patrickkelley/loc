# PRD: `loc` — Lines of Code CLI

## Problem statement
Developers need a quick, accurate count of the lines of code in a local git
repository. Naive `wc -l` style counts overcount because they include blank
lines and comments. `loc` counts only meaningful code lines, using real
parsers (tree-sitter) so that comment markers inside strings are not mistaken
for comments.

## Goals
- Accurate non-blank, non-comment line counts for common languages.
- Correct handling of strings: `//` or `#` inside a string literal is code, not a comment.
- Only files tracked by git are counted (respects `.gitignore`, excludes untracked junk).
- Fast, small CLI with clear per-language and total output.

## Non-goals
- Counting files outside git repos.
- Token-level metrics (cyclomatic complexity, symbols, etc.).
- Supporting every language tree-sitter has a grammar for (extensible set only).

## Requirements

### R1 — Repo discovery
- Run from the current working directory; locate the repo root by walking up
  using `git rev-parse --show-toplevel`.
- List files with `git ls-files -z` (tracked files only).
- Untracked and `.gitignore`d files are excluded.
- Running outside a git repo is a clean error, not a crash.

### R2 — Counting
- A counted line is any line that contains code.
- Excluded: blank/whitespace-only lines, comment lines (line and block).
- Included: string literal lines containing comment-like markers, multi-line
  strings/docstrings.
- Language detection by file extension.

### R3 — Supported languages (initial set)
| Language    | Extensions                                   |
|-------------|----------------------------------------------|
| Rust        | `.rs`                                        |
| Python      | `.py`                                        |
| JavaScript  | `.js`, `.mjs`, `.cjs`, `.jsx`                |
| TypeScript  | `.ts`, `.tsx`                                |
| Go          | `.go`                                        |
| C           | `.c`, `.h`                                   |
| C++         | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx` |
| Java        | `.java`                                      |
| Gosu        | `.gs`, `.gsp`, `.gsx`, `.gst`                |
| Ruby        | `.rb`                                        |
| Shell       | `.sh`, `.bash`                               |

### R4 — Fallbacks
- Unknown extension → counted as non-blank lines (comments are *not*
  excluded), reported under `other`.
- Binary file → skipped entirely, not an error. A file is binary when:
  - it starts with a UTF-16 BOM (`FF FE` / `FE FF`), or
  - it contains a NUL byte in the first KB **and** the first KB does not
    decode as valid UTF-8. If the first KB decodes as valid UTF-8, the file
    is text and the NUL is treated as a code character — true binaries
    (`.class`, images, archives) still get skipped because their first KB
    contains invalid UTF-8 beyond the NUL.

### R5 — CLI
- `loc` — count the repo found from cwd.
- `loc --files` — additionally print a per-file breakdown.
- `loc --help`, `loc --version`.

### R6 — Output
- Table: `language, files, lines`, sorted, plus a total row.

## Implementation workflow — Strict TDD
Every unit of work follows red-green-refactor:
1. **Red**: write a failing test first (assertion failure or compile error).
2. **Green**: write the minimum production code to pass.
3. **Refactor**: clean up with tests staying green.

- `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` must be
  green before moving to the next test.
- No speculative abstractions; internals emerge via refactoring.
- Minimal unavoidable scaffold before the first test is acceptable and noted.

## Work breakdown
0. Write this PRD (`docs/PRD.md`).
1. Scaffold crate + dependencies; first failing test for repo discovery.
2. `git.rs`: toplevel discovery, tracked-file listing, non-repo error
   (temp git repo fixtures).
3. `counter.rs`: blank/comment/string/binary/fallback cases per language.
4. `langs.rs`: extension → grammar mapping.
5. CLI integration tests (`assert_cmd`): e2e table output, `--files`,
   `--help`, `--version`.
6. Refactor pass + verification: full test/clippy/fmt run and a manual run on
   a real repo (sanity vs `tokei` if available).

## Dependencies
- Runtime: `clap` (derive), `tree-sitter`, grammar crates: `tree-sitter-rust`,
  `tree-sitter-python`, `tree-sitter-javascript`, `tree-sitter-typescript`,
  `tree-sitter-go`, `tree-sitter-c`, `tree-sitter-cpp`, `tree-sitter-java`,
  `tree-sitter-gosu`, `tree-sitter-ruby`, `tree-sitter-bash`.
- Dev: `assert_cmd`, `predicates`, `tempfile`.

## Module structure
```
Cargo.toml
docs/PRD.md
src/
  main.rs      — arg parsing + wiring (thin)
  git.rs       — rev-parse toplevel, ls-files listing
  counter.rs   — tree-sitter walk / fallback counting
  langs.rs     — extension → language map
tests/         — integration tests (temp git repo fixtures)
```

## Acceptance criteria
- All tests green; clippy and fmt clean.
- Counts verified against hand-checked fixtures: strings with comment
  markers, block comments, docstrings, binaries, ignored/untracked files,
  non-repo directories.
- Fixtures for the R4 binary rule: a text file with an embedded NUL in a
  string literal (counted as text), a UTF-16 BOM file (skipped), and a real
  binary blob (skipped).
- Manual run on a real repo produces sane counts.

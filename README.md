# loc

A fast, comment-aware lines-of-code CLI for local Git repositories, powered by [tree-sitter](https://tree-sitter.github.io/).

Unlike `wc -l` or naive regex counters, `loc` parses tracked source files with tree-sitter grammars so comment markers inside string literals and multi-line strings/docstrings are counted as code, while blank lines and comments are excluded.

## Features

- **Git-aware**: Finds the repository root from the current directory (`git rev-parse --show-toplevel`) and counts only tracked files (`git ls-files -z`), automatically excluding untracked and `.gitignore`d files.
- **Comment-aware**: Strips line and block comments using tree-sitter syntax trees while preserving comment-like markers inside strings and docstrings.
- **Safe fallbacks**: Unknown file extensions fall back to non-blank line counts under `other`; binary files (NUL byte in the first 1024 bytes) are skipped.

## Installation

### From source

```sh
cargo install --path .
```

Or build locally:

```sh
cargo build --release
```

### Prebuilt binaries

Release tarballs are published on GitHub Releases for tagged releases (`v*`) covering:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Usage

Run `loc` from anywhere inside a Git repository:

```sh
loc            # per-language summary table + total
loc --files    # include per-file line breakdown
loc --help     # show usage
loc --version  # show version
```

Example output:

```text
Language     Files  Lines
other            7    789
rust             9    478
Total           16   1267
```

## Supported Languages

| Language   | Extensions                                   |
|------------|----------------------------------------------|
| Rust       | `.rs`                                        |
| Python     | `.py`                                        |
| JavaScript | `.js`, `.mjs`, `.cjs`, `.jsx`                |
| TypeScript | `.ts`, `.tsx`                                |
| Go         | `.go`                                        |
| C          | `.c`, `.h`                                   |
| C++        | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx` |
| Java       | `.java`                                      |
| Gosu       | `.gs`, `.gsp`, `.gsx`, `.gst`                |
| Ruby       | `.rb`                                        |
| Shell      | `.sh`, `.bash`                               |

All other non-binary tracked files are counted as non-blank lines under `other`.

## Development

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-features
```

See `docs/PRD.md` for requirements and scope.

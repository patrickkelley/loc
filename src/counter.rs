use crate::langs::Lang;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Count {
    pub language: String,
    pub lines: u64,
}

pub fn count_file(path: &Path) -> anyhow::Result<Option<Count>> {
    let bytes = std::fs::read(path)?;
    if bytes.get(..1024).unwrap_or(&bytes[..]).contains(&0) {
        return Ok(None);
    }
    let src = String::from_utf8_lossy(&bytes).into_owned();
    let (language, lines) = match Lang::from_path(path) {
        Some(lang) => {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang.ts_language())?;
            let tree = parser.parse(&src, None).expect("grammar should parse");
            let mut comments = Vec::new();
            collect_comment_ranges(tree.root_node(), &mut comments);
            (lang.name().to_string(), count_lines(&src, &comments))
        }
        None => ("other".to_string(), fallback_lines(&src)),
    };
    Ok(Some(Count { language, lines }))
}

fn fallback_lines(src: &str) -> u64 {
    src.lines().filter(|l| !l.trim().is_empty()).count() as u64
}

fn count_lines(src: &str, comment_ranges: &[(usize, usize)]) -> u64 {
    let mut total = 0;
    let mut start = 0;
    for line in src.split('\n') {
        let end = start + line.len();
        let mut has_code = false;
        for (i, b) in src.as_bytes()[start..end].iter().enumerate() {
            if b.is_ascii_whitespace() {
                continue;
            }
            let pos = start + i;
            let in_comment = comment_ranges.iter().any(|&(cs, ce)| pos >= cs && pos < ce);
            if !in_comment {
                has_code = true;
                break;
            }
        }
        if has_code {
            total += 1;
        }
        start = end + 1;
    }
    total
}

fn collect_comment_ranges(node: tree_sitter::Node, out: &mut Vec<(usize, usize)>) {
    if node.kind().contains("comment") {
        out.push((node.start_byte(), node.end_byte()));
        return;
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            collect_comment_ranges(child, out);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

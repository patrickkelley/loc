use crate::langs::Lang;
use std::io::Read;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Count {
    pub language: String,
    pub lines: u64,
}

pub fn count_file(path: &Path) -> anyhow::Result<Option<Count>> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) if m.is_file() => m,
        Ok(_) => return Ok(None),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut bytes = Vec::with_capacity(meta.len().min(1024) as usize);
    (&mut file).take(1024).read_to_end(&mut bytes)?;
    if bytes.contains(&0) {
        return Ok(None);
    }
    if bytes.len() == 1024 {
        bytes.reserve(meta.len().saturating_sub(1024) as usize);
        file.read_to_end(&mut bytes)?;
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
    src.lines()
        .filter(|l| {
            !l.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
                .is_empty()
        })
        .count() as u64
}

fn count_lines(src: &str, comment_ranges: &[(usize, usize)]) -> u64 {
    let mut total = 0;
    let mut start = 0;
    let mut c_idx = 0;
    for line in src.split('\n') {
        let mut has_code = false;
        for (i, ch) in line.char_indices() {
            if ch.is_whitespace() || ch == '\u{feff}' {
                continue;
            }
            let pos = start + i;
            while c_idx < comment_ranges.len() && comment_ranges[c_idx].1 <= pos {
                c_idx += 1;
            }
            let in_comment = c_idx < comment_ranges.len() && pos >= comment_ranges[c_idx].0;
            if !in_comment {
                has_code = true;
                break;
            }
        }
        if has_code {
            total += 1;
        }
        start += line.len() + 1;
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

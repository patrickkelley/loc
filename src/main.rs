use clap::Parser;
use loc::{counter, git};
use std::collections::BTreeMap;

#[derive(Parser)]
#[command(
    name = "loc",
    version,
    about = "Count lines of code in a git repository"
)]
struct Args {
    /// Print a per-file breakdown
    #[arg(long)]
    files: bool,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let root = git::find_toplevel(&cwd)?;
    let files = git::tracked_files(&root)?;

    let mut langs: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    let mut per_file: Vec<(String, u64)> = Vec::new();
    for path in &files {
        if let Some(c) = counter::count_file(path)? {
            let entry = langs.entry(c.language).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += c.lines;
            if args.files {
                let rel = path.strip_prefix(&root)?.to_string_lossy().into_owned();
                per_file.push((rel, c.lines));
            }
        }
    }

    let mut total_files = 0;
    let mut total_lines = 0;
    println!("{:<12} {:>5} {:>6}", "Language", "Files", "Lines");
    for (name, (files, lines)) in &langs {
        println!("{:<12} {:>5} {:>6}", name, files, lines);
        total_files += files;
        total_lines += lines;
    }
    println!("{:<12} {:>5} {:>6}", "Total", total_files, total_lines);

    if args.files {
        per_file.sort_by(|a, b| a.0.cmp(&b.0));
        let path_width = per_file
            .iter()
            .map(|(rel, _)| rel.chars().count())
            .max()
            .unwrap_or(0)
            .max(39);
        let num_width = per_file
            .iter()
            .map(|(_, lines)| lines.checked_ilog10().unwrap_or(0) as usize + 1)
            .max()
            .unwrap_or(0)
            .max(6);
        println!();
        println!("Files:");
        for (rel, lines) in &per_file {
            println!("  {:<path_width$} {:>num_width$}", rel, lines);
        }
    }
    Ok(())
}

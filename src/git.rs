use std::path::PathBuf;

pub fn find_toplevel(path: &std::path::Path) -> anyhow::Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args([
            "-C",
            path.to_str().expect("utf8 path"),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("not a git repository: {}", path.display());
    }
    let s = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(s))
}

pub fn tracked_files(root: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    let output = std::process::Command::new("git")
        .args(["-C", root.to_str().expect("utf8 path"), "ls-files", "-z"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("git ls-files failed: {}", root.display());
    }
    Ok(String::from_utf8(output.stdout)?
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|rel| root.join(rel))
        .collect())
}

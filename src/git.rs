use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_toplevel(path: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("not a git repository: {}", path.display());
    }
    let mut raw = output.stdout.as_slice();
    if let Some(stripped) = raw.strip_suffix(b"\n") {
        raw = stripped;
    }
    if let Some(stripped) = raw.strip_suffix(b"\r") {
        raw = stripped;
    }
    bytes_to_path(raw)
}

pub fn tracked_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", "-s"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("git ls-files failed: {}", root.display());
    }
    let mut paths = Vec::new();
    for entry in output.stdout.split(|&b| b == 0) {
        if entry.is_empty() {
            continue;
        }
        let Some(tab) = entry.iter().position(|&b| b == b'\t') else {
            continue;
        };
        let mode = entry[..tab].split(|&b| b == b' ').next();
        if !matches!(mode, Some(b"100644" | b"100755")) {
            continue;
        }
        let rel = bytes_to_path(&entry[tab + 1..])?;
        let full = root.join(rel);
        match std::fs::symlink_metadata(&full) {
            Ok(m) if m.is_file() => paths.push(full),
            Ok(_) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) => {}
            Err(e) => return Err(e.into()),
        }
    }
    paths.dedup();
    Ok(paths)
}

#[cfg(unix)]
fn bytes_to_path(bytes: &[u8]) -> anyhow::Result<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    Ok(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
fn bytes_to_path(bytes: &[u8]) -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(std::str::from_utf8(bytes)?))
}

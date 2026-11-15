use std::path::Path;
use std::process::Command;

/// Create a git repo at `dir` and `git add` all files inside it.
pub fn init_repo(dir: &Path) {
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git should be installed");
        assert!(status.success(), "git {args:?} failed");
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["add", "-A"]);
}

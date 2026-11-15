mod common;
use common::init_repo;

#[test]
fn finds_toplevel_from_subdirectory() {
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    let sub = dir.path().join("a/b");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("f.txt"), "x").unwrap();

    let found = loc::git::find_toplevel(&sub);
    assert_eq!(found.unwrap(), dir.path());
}

#[test]
fn lists_tracked_files_only() {
    let dir = tempfile::tempdir().unwrap();
    init_repo(dir.path());
    std::fs::write(dir.path().join("tracked.rs"), "x").unwrap();
    let ignored = dir.path().join("ignored.txt");
    std::fs::write(&ignored, "x").unwrap();
    std::fs::write(dir.path().join(".gitignore"), "ignored.txt\n").unwrap();
    init_repo(dir.path()); // re-add to commit/tracked state
    std::fs::write(dir.path().join("untracked.txt"), "x").unwrap();

    let files = loc::git::tracked_files(dir.path()).unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| {
            p.strip_prefix(dir.path())
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert!(names.contains(&".gitignore".to_string()));
    assert!(names.contains(&"tracked.rs".to_string()));
    assert!(!names.contains(&"untracked.txt".to_string()));
    assert!(!names.contains(&"ignored.txt".to_string()));
}

#[test]
fn errors_outside_repo() {
    let dir = tempfile::tempdir().unwrap();
    // find_toplevel must not depend on cwd; pass the non-repo dir directly
    let result = loc::git::find_toplevel(dir.path());
    assert!(result.is_err());
}

#[test]
fn skips_deleted_files_and_submodules_and_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("real.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.path().join("deleted.rs"), "fn gone() {}\n").unwrap();
    let replaced_dir = dir.path().join("replaced");
    std::fs::create_dir(&replaced_dir).unwrap();
    std::fs::write(replaced_dir.join("nested.rs"), "fn nested() {}\n").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        std::fs::create_dir(dir.path().join("adir")).unwrap();
        symlink("real.rs", dir.path().join("link_file.rs")).unwrap();
        symlink("missing.rs", dir.path().join("link_broken.rs")).unwrap();
        symlink("adir", dir.path().join("link_dir")).unwrap();
    }

    init_repo(dir.path());
    std::fs::remove_file(dir.path().join("deleted.rs")).unwrap();
    std::fs::remove_dir_all(&replaced_dir).unwrap();
    std::fs::write(&replaced_dir, "not a directory anymore\n").unwrap();

    // Register a gitlink (submodule, mode 160000) backed by a real directory on disk.
    let sub_dir = dir.path().join("submod");
    std::fs::create_dir(&sub_dir).unwrap();
    let status = std::process::Command::new("git")
        .args([
            "update-index",
            "--add",
            "--cacheinfo",
            "160000,1111111111111111111111111111111111111111,submod",
        ])
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(status.success());

    let files = loc::git::tracked_files(dir.path()).unwrap();
    assert_eq!(files, vec![dir.path().join("real.rs")]);
}

#[cfg(unix)]
#[test]
fn propagates_permission_denied_on_unreadable_subdir() {
    use std::os::unix::fs::PermissionsExt;

    struct RestorePerms(std::path::PathBuf);
    impl Drop for RestorePerms {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
        }
    }

    let dir = tempfile::tempdir().unwrap();
    let locked = dir.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::write(locked.join("secret.rs"), "fn main() {}\n").unwrap();
    init_repo(dir.path());

    let _guard = RestorePerms(locked.clone());
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

    // Skip if running with elevated privileges (e.g. root in a container) where chmod 000 does not block stat.
    if std::fs::symlink_metadata(locked.join("secret.rs")).is_ok() {
        return;
    }

    let res = loc::git::tracked_files(dir.path());
    assert!(
        res.is_err(),
        "expected permission error from tracked_files, got: {res:?}"
    );
}

#[cfg(unix)]
#[test]
fn handles_non_utf8_repo_path() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let base = tempfile::tempdir().unwrap();
    let canonical_base = base.path().canonicalize().unwrap();
    let non_utf8 = canonical_base.join(OsStr::from_bytes(b"repo_\xff"));
    if std::fs::create_dir(&non_utf8).is_err() {
        // Filesystems enforcing UTF-8 filenames (e.g. macOS APFS, ZFS utf8only) reject raw 0xff bytes.
        std::fs::create_dir(canonical_base.join("utf8_probe")).unwrap();
        return;
    }
    std::fs::write(non_utf8.join("main.rs"), "fn main() {}\n").unwrap();
    init_repo(&non_utf8);

    let root = loc::git::find_toplevel(&non_utf8).unwrap();
    assert_eq!(root, non_utf8);
    let files = loc::git::tracked_files(&root).unwrap();
    assert_eq!(files, vec![non_utf8.join("main.rs")]);
}

#[test]
fn tracked_files_errors_outside_repo() {
    let dir = tempfile::tempdir().unwrap();
    // No git repo here: `git ls-files` must fail and be surfaced as an error.
    let res = loc::git::tracked_files(dir.path());
    assert!(
        res.is_err(),
        "expected ls-files failure outside a repo, got: {res:?}"
    );
}

#[test]
fn tracked_file_replaced_by_directory_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("flipped.rs"), "fn main() {}\n").unwrap();
    init_repo(dir.path());

    // After commit, replace the tracked file with a directory of the same name.
    std::fs::remove_file(dir.path().join("flipped.rs")).unwrap();
    std::fs::create_dir(dir.path().join("flipped.rs")).unwrap();

    let files = loc::git::tracked_files(dir.path()).unwrap();
    assert_eq!(files, Vec::<std::path::PathBuf>::new());
}

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

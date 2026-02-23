use std::fs;
use crate::common::run_rsb_with_env;
use tempfile::TempDir;

/// Helper: create a tags test project with given .md files and optional .tags file.
fn setup_tags_project(md_files: &[(&str, &str)], tags_file: Option<&str>) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let p = temp_dir.path();

    let config = "[processor]\nenabled = [\"tags\"]\n";
    fs::write(p.join("rsb.toml"), config).unwrap();

    for (name, content) in md_files {
        if let Some(parent) = std::path::Path::new(name).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(p.join(parent)).unwrap();
            }
        }
        fs::write(p.join(name), content).unwrap();
    }

    if let Some(tags_content) = tags_file {
        fs::write(p.join(".tags"), tags_content).unwrap();
    }

    temp_dir
}

#[test]
fn tags_basic_build_and_query() {
    let temp_dir = setup_tags_project(
        &[
            ("course1.md", "---\nlevel: beginner\ntags:\n  - python\n  - docker\n---\n# Course 1\n"),
            ("course2.md", "---\nlevel: advanced\ntags:\n  - rust\n  - docker\n---\n# Course 2\n"),
        ],
        None,
    );
    let p = temp_dir.path();

    // Build should succeed and create the tags database
    let output = run_rsb_with_env(p, &["build"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "build failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(p.join("out/tags/tags.db").exists(), "tags database should be created");

    // `rsb tags list` should show all tags sorted
    let list_output = run_rsb_with_env(p, &["tags", "list"], &[("NO_COLOR", "1")]);
    assert!(list_output.status.success());
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    let tags: Vec<&str> = stdout.lines().collect();
    assert!(tags.contains(&"docker"));
    assert!(tags.contains(&"python"));
    assert!(tags.contains(&"rust"));
    assert!(tags.contains(&"level=beginner"));
    assert!(tags.contains(&"level=advanced"));

    // `rsb tags files docker` should return both files
    let files_output = run_rsb_with_env(p, &["tags", "files", "docker"], &[("NO_COLOR", "1")]);
    assert!(files_output.status.success());
    let files_stdout = String::from_utf8_lossy(&files_output.stdout);
    assert!(files_stdout.contains("course1.md"));
    assert!(files_stdout.contains("course2.md"));

    // `rsb tags files docker rust` (AND) should return only course2
    let and_output = run_rsb_with_env(p, &["tags", "files", "docker", "rust"], &[("NO_COLOR", "1")]);
    assert!(and_output.status.success());
    let and_stdout = String::from_utf8_lossy(&and_output.stdout);
    assert!(!and_stdout.contains("course1.md"));
    assert!(and_stdout.contains("course2.md"));

    // `rsb tags files --or python rust` (OR) should return both files
    let or_output = run_rsb_with_env(p, &["tags", "files", "--or", "python", "rust"], &[("NO_COLOR", "1")]);
    assert!(or_output.status.success());
    let or_stdout = String::from_utf8_lossy(&or_output.stdout);
    assert!(or_stdout.contains("course1.md"));
    assert!(or_stdout.contains("course2.md"));
}

#[test]
fn tags_validation_rejects_unknown_tags() {
    let temp_dir = setup_tags_project(
        &[
            ("course.md", "---\nlevel: beginner\ntags:\n  - python\n  - dockker\n---\n# Course\n"),
        ],
        Some("python\ndocker\nlevel=beginner\n"),
    );
    let p = temp_dir.path();

    // Build should fail because "dockker" is not in .tags
    let output = run_rsb_with_env(p, &["build"], &[("NO_COLOR", "1")]);
    assert!(!output.status.success(), "build should fail with unknown tag");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("dockker"), "error should mention the unknown tag: {}", stderr);
    // Should suggest "docker" as a typo correction
    assert!(stderr.contains("docker"), "error should suggest 'docker': {}", stderr);
}

#[test]
fn tags_validation_allows_wildcard_patterns() {
    let temp_dir = setup_tags_project(
        &[
            ("course.md", "---\nlevel: beginner\nduration_days: 5\ntags:\n  - python\n---\n# Course\n"),
        ],
        // Wildcard pattern for duration_days=*
        Some("python\nlevel=beginner\nduration_days=*\n"),
    );
    let p = temp_dir.path();

    // Build should succeed because duration_days=5 matches duration_days=*
    let output = run_rsb_with_env(p, &["build"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(),
        "build should succeed with wildcard pattern: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn tags_for_file_path_matching() {
    let temp_dir = setup_tags_project(
        &[
            ("sub/foo.md", "---\ntags:\n  - alpha\n---\n# Foo\n"),
            ("sub/barfoo.md", "---\ntags:\n  - beta\n---\n# Barfoo\n"),
        ],
        None,
    );
    let p = temp_dir.path();

    let output = run_rsb_with_env(p, &["build"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "build failed: {}", String::from_utf8_lossy(&output.stderr));

    // Querying for "sub/foo.md" should return alpha, NOT beta
    let for_file = run_rsb_with_env(p, &["tags", "for-file", "sub/foo.md"], &[("NO_COLOR", "1")]);
    assert!(for_file.status.success());
    let stdout = String::from_utf8_lossy(&for_file.stdout);
    assert!(stdout.contains("alpha"), "should find tag 'alpha' for sub/foo.md: {}", stdout);
    assert!(!stdout.contains("beta"), "should NOT match barfoo.md's tag 'beta': {}", stdout);
}

#[test]
fn tags_init_and_unused() {
    let temp_dir = setup_tags_project(
        &[
            ("a.md", "---\ntags:\n  - used\n---\n"),
        ],
        None,
    );
    let p = temp_dir.path();

    // Build first to populate db
    let build = run_rsb_with_env(p, &["build"], &[("NO_COLOR", "1")]);
    assert!(build.status.success());

    // Init should create .tags
    let init = run_rsb_with_env(p, &["tags", "init"], &[("NO_COLOR", "1")]);
    assert!(init.status.success());
    assert!(p.join(".tags").exists(), ".tags file should be created");

    let tags_content = fs::read_to_string(p.join(".tags")).unwrap();
    assert!(tags_content.contains("used"));

    // Add an extra tag that doesn't exist in any file
    fs::write(p.join(".tags"), "used\nobsolete\n").unwrap();

    // `rsb tags unused` should report "obsolete" as unused
    let unused = run_rsb_with_env(p, &["tags", "unused"], &[("NO_COLOR", "1")]);
    assert!(unused.status.success());
    let unused_stdout = String::from_utf8_lossy(&unused.stdout);
    assert!(unused_stdout.contains("obsolete"), "should report 'obsolete' as unused: {}", unused_stdout);
    assert!(!unused_stdout.contains("used\n"), "should not report 'used' as unused");
}

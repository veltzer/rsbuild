use std::fs;
use tempfile::TempDir;
use crate::common::run_rsb;

#[test]
fn init_creates_project() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    let output = run_rsb(project_path, &["init"]);
    assert!(output.status.success(), "rsb init failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check files were created
    assert!(project_path.join("rsb.toml").exists(), "rsb.toml should be created");
    assert!(project_path.join(".rsbignore").exists(), ".rsbignore should be created");

    // Verify rsb.toml has content
    let toml_content = fs::read_to_string(project_path.join("rsb.toml")).unwrap();
    assert!(toml_content.contains("[build]"), "rsb.toml should contain [build] section");
    assert!(toml_content.contains("[processor]"), "rsb.toml should contain [processor] section");

    // Verify .rsbignore has content
    let rsbignore_content = fs::read_to_string(project_path.join(".rsbignore")).unwrap();
    assert!(rsbignore_content.contains(".gitignore syntax"), ".rsbignore should reference gitignore syntax");

    assert!(stdout.contains("Created"), "Output should mention Created");
}

#[test]
fn init_fails_if_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    // Create rsb.toml first
    fs::write(project_path.join("rsb.toml"), "# existing").unwrap();

    let output = run_rsb(project_path, &["init"]);
    assert!(!output.status.success(), "rsb init should fail if rsb.toml exists");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "Error should mention 'already exists': {}", stderr);
}

#[test]
fn init_ignores_existing_dirs() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    // Create templates.tera dir with a file
    fs::create_dir_all(project_path.join("templates.tera")).unwrap();
    fs::write(project_path.join("templates.tera/existing.txt"), "do not delete").unwrap();

    let output = run_rsb(project_path, &["init"]);
    assert!(output.status.success());

    // Existing file should still be there
    assert!(project_path.join("templates.tera/existing.txt").exists(),
        "Existing files in templates.tera/ should be preserved");
    let content = fs::read_to_string(project_path.join("templates.tera/existing.txt")).unwrap();
    assert_eq!(content, "do not delete");
}

#[test]
fn init_preserves_existing_rsbignore() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    // Create .rsbignore with custom content before init
    let custom_content = "# my custom ignore rules\n*.tmp\n";
    fs::write(project_path.join(".rsbignore"), custom_content).unwrap();

    let output = run_rsb(project_path, &["init"]);
    assert!(output.status.success(), "rsb init failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify .rsbignore was not overwritten
    let content = fs::read_to_string(project_path.join(".rsbignore")).unwrap();
    assert_eq!(content, custom_content, ".rsbignore should not be overwritten");
}

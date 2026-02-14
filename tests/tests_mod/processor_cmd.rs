use std::fs;
use tempfile::TempDir;
use crate::common::{setup_test_project, run_rsb_with_env};

#[test]
fn processors_list_shows_enabled() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    let output = run_rsb_with_env(project_path, &["processors", "list"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "processors list failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tera"), "Expected tera processor in list");
    assert!(stdout.contains("enabled"), "Expected 'enabled' status for tera");
}

#[test]
fn processors_list_shows_disabled() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    let output = run_rsb_with_env(project_path, &["processors", "list"], &[("NO_COLOR", "1")]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Only tera is enabled in setup_test_project, others should be disabled
    assert!(stdout.contains("disabled"), "Expected some disabled processors in list");
}

#[test]
fn processors_auto_detects_tera() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // Write a template file so the tera processor is detected
    fs::write(
        project_path.join("templates/test.txt.tera"),
        "hello"
    ).expect("Failed to write template");

    let output = run_rsb_with_env(project_path, &["processors", "auto"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "processors auto failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tera"), "Expected tera in auto-detect output");
    assert!(stdout.contains("detected"), "Expected 'detected' for tera processor");
}

#[test]
fn processors_files_shows_products() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // Write a template so there's at least one product
    fs::write(
        project_path.join("config/test.py"),
        "value = 42"
    ).expect("Failed to write config");
    fs::write(
        project_path.join("templates/output.txt.tera"),
        "{% set c = load_python(path='config/test.py') %}{{ c.value }}"
    ).expect("Failed to write template");

    let output = run_rsb_with_env(project_path, &["processors", "files"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "processors files failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[tera]"), "Expected [tera] header in output");
    assert!(stdout.contains("output.txt"), "Expected output file in listing");
}

#[test]
fn processors_files_no_files_message() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // No template files written, so no products
    let output = run_rsb_with_env(project_path, &["processors", "files"], &[("NO_COLOR", "1")]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No files discovered") || stdout.contains("(no files)"),
        "Expected empty message, got: {}", stdout);
}

#[test]
fn processors_files_unknown_processor_fails() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    let output = run_rsb_with_env(project_path, &["processors", "files", "nonexistent"], &[("NO_COLOR", "1")]);
    assert!(!output.status.success(), "Expected failure for unknown processor");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown processor"), "Expected 'Unknown processor' error, got: {}", stderr);
}

#[test]
fn processors_all_shows_descriptions() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    let output = run_rsb_with_env(project_path, &["processors", "all"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "processors all failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // processors all shows descriptions with " — " separator
    assert!(stdout.contains("tera"), "Expected tera processor");
}

#[test]
fn processors_files_json_output() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // Write a template so there's at least one product
    fs::write(
        project_path.join("config/test.py"),
        "value = 42"
    ).expect("Failed to write config");
    fs::write(
        project_path.join("templates/output.txt.tera"),
        "{% set c = load_python(path='config/test.py') %}{{ c.value }}"
    ).expect("Failed to write template");

    let output = run_rsb_with_env(project_path, &["--json", "processors", "files"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "processors files --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .expect("Expected valid JSON array");
    assert!(!entries.is_empty(), "Expected at least one entry");

    let entry = &entries[0];
    assert!(entry.get("processor").is_some(), "Entry should have 'processor' field");
    assert!(entry.get("processor_type").is_some(), "Entry should have 'processor_type' field");
    assert!(entry.get("inputs").is_some(), "Entry should have 'inputs' field");
    assert!(entry.get("outputs").is_some(), "Entry should have 'outputs' field");
    assert_eq!(entry["processor"], "tera");
    assert_eq!(entry["processor_type"], "generator");
}

#[test]
fn processors_files_json_empty() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // No template files written, so no products
    let output = run_rsb_with_env(project_path, &["--json", "processors", "files"], &[("NO_COLOR", "1")]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .expect("Expected valid JSON array");
    assert!(entries.is_empty(), "Expected empty JSON array, got: {}", stdout);
}

#[test]
fn processors_all_works_without_config() {
    // Run from a temp dir with no rsb.toml
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = run_rsb_with_env(temp_dir.path(), &["processors", "all"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "processors all should work without rsb.toml: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tera"), "Expected tera processor in output");
    assert!(stdout.contains("ruff"), "Expected ruff processor in output");
    assert!(stdout.contains("shellcheck"), "Expected shellcheck processor in output");
}

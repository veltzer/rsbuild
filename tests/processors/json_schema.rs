use std::fs;
use tempfile::TempDir;
use crate::common::run_rsb_with_env;

#[test]
fn json_schema_valid() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsb.toml"),
        "[processor]\nenabled = [\"json_schema\"]\n",
    )
    .unwrap();

    fs::write(
        project_path.join("schema.json"),
        r#"{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "age": { "type": "integer" }
  },
  "propertyOrdering": ["name", "age"]
}"#,
    )
    .unwrap();

    let output = run_rsb_with_env(project_path, &["build", "-v"], &[("NO_COLOR", "1")]);
    assert!(
        output.status.success(),
        "Build should succeed with valid schema: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Processing:"),
        "Should process json_schema: {}",
        stdout
    );
}

#[test]
fn json_schema_mismatch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsb.toml"),
        "[processor]\nenabled = [\"json_schema\"]\n",
    )
    .unwrap();

    fs::write(
        project_path.join("bad.json"),
        r#"{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "age": { "type": "integer" }
  },
  "propertyOrdering": ["name"]
}"#,
    )
    .unwrap();

    let output = run_rsb_with_env(project_path, &["build", "-v"], &[("NO_COLOR", "1")]);
    assert!(
        !output.status.success(),
        "Build should fail with mismatched propertyOrdering"
    );
}

#[test]
fn json_schema_incremental_skip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsb.toml"),
        "[processor]\nenabled = [\"json_schema\"]\n",
    )
    .unwrap();

    fs::write(
        project_path.join("schema.json"),
        r#"{
  "type": "object",
  "properties": {
    "name": { "type": "string" }
  },
  "propertyOrdering": ["name"]
}"#,
    )
    .unwrap();

    // First build
    let output1 = run_rsb_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(output1.status.success());

    // Second build should skip
    let output2 = run_rsb_with_env(project_path, &["build", "--verbose"], &[("NO_COLOR", "1")]);
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("[json_schema] Skipping (unchanged):"),
        "Second build should skip: {}",
        stdout2
    );
}

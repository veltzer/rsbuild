use std::fs;
use tempfile::TempDir;
use sha2::{Sha256, Digest};
use crate::common::run_rsconstruct_with_env;

const SCHEMA_URL: &str = "https://example.com/test_schema.json";

/// The schema content for tests. propertyOrdering is ["age", "name"] which
/// matches alphabetical order (serde_json::Value uses BTreeMap so keys are
/// always sorted alphabetically).
const SCHEMA: &str = r#"{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" }
    },
    "propertyOrdering": ["age", "name"],
    "required": ["name"]
}"#;

/// Pre-populate the webcache so the processor can find the schema without
/// network access. The webcache stores files under
/// `.rsconstruct/webcache/<first-2-chars-of-sha256>/<rest-of-sha256>`.
fn populate_webcache(project_path: &std::path::Path, url: &str, content: &str) {
    let hash = hex::encode(Sha256::digest(url.as_bytes()));
    let prefix = &hash[..2];
    let rest = &hash[2..];
    let cache_dir = project_path.join(".rsconstruct/webcache").join(prefix);
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(cache_dir.join(rest), content).unwrap();
}

#[test]
fn iyamlschema_valid_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    populate_webcache(project_path, SCHEMA_URL, SCHEMA);

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor.iyamlschema]\nscan_dirs = [\".\"]\n",
    ).unwrap();

    fs::write(
        project_path.join("data.yaml"),
        format!("$schema: \"{}\"\nname: Alice\nage: 30\n", SCHEMA_URL),
    ).unwrap();

    let output = run_rsconstruct_with_env(project_path, &["build", "-v"], &[("NO_COLOR", "1")]);
    assert!(
        output.status.success(),
        "Build should succeed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn iyamlschema_invalid_data_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    populate_webcache(project_path, SCHEMA_URL, SCHEMA);

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor.iyamlschema]\nscan_dirs = [\".\"]\n",
    ).unwrap();

    // "age" should be integer, not string
    fs::write(
        project_path.join("data.yaml"),
        format!("$schema: \"{}\"\nname: Alice\nage: not_a_number\n", SCHEMA_URL),
    ).unwrap();

    let output = run_rsconstruct_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(!output.status.success(), "Build should fail for invalid data");
}

#[test]
fn iyamlschema_wrong_ordering_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    // Use a schema where propertyOrdering does NOT match alphabetical order
    let schema_wrong_order = r#"{
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "propertyOrdering": ["name", "age"],
        "required": ["name"]
    }"#;
    let wrong_url = "https://example.com/wrong_order_schema.json";
    populate_webcache(project_path, wrong_url, schema_wrong_order);

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor.iyamlschema]\nscan_dirs = [\".\"]\n",
    ).unwrap();

    // Keys will be sorted alphabetically to ["age", "name"] but schema
    // expects ["name", "age"], so ordering check should fail.
    fs::write(
        project_path.join("data.yaml"),
        format!("$schema: \"{}\"\nname: Alice\nage: 30\n", wrong_url),
    ).unwrap();

    let output = run_rsconstruct_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(!output.status.success(), "Build should fail for wrong key order");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("property ordering"), "Error should mention property ordering: {}", stderr);
}

#[test]
fn iyamlschema_no_schema_field_fails() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor.iyamlschema]\nscan_dirs = [\".\"]\n",
    ).unwrap();

    fs::write(
        project_path.join("data.yaml"),
        "name: Alice\nage: 30\n",
    ).unwrap();

    let output = run_rsconstruct_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(!output.status.success(), "Build should fail when $schema is missing");
}

#[test]
fn iyamlschema_incremental_skip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    populate_webcache(project_path, SCHEMA_URL, SCHEMA);

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor.iyamlschema]\nscan_dirs = [\".\"]\n",
    ).unwrap();

    fs::write(
        project_path.join("data.yaml"),
        format!("$schema: \"{}\"\nname: Alice\nage: 30\n", SCHEMA_URL),
    ).unwrap();

    // First build
    let output1 = run_rsconstruct_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(
        output1.status.success(),
        "First build should succeed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output1.stdout),
        String::from_utf8_lossy(&output1.stderr),
    );

    // Second build should skip
    let output2 = run_rsconstruct_with_env(project_path, &["build", "--verbose"], &[("NO_COLOR", "1")]);
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("[iyamlschema] Skipping (unchanged):"),
        "Second build should skip: {}", stdout2,
    );
}

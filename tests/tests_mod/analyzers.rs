use std::fs;
use tempfile::TempDir;
use crate::common::run_rsconstruct_with_env;

/// `enabled = false` on an analyzer stanza must keep it out of the active set —
/// `analyzers used` is the public surface for this and should omit disabled analyzers.
#[test]
fn analyzer_disabled_via_enabled_false() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        r#"[processor.markdown2html]
src_dirs = ["."]

[analyzer.markdown]
enabled = false
"#,
    )
    .unwrap();

    fs::write(project_path.join("doc.md"), "# hi\n").unwrap();

    let output = run_rsconstruct_with_env(
        project_path,
        &["analyzers", "used"],
        &[("NO_COLOR", "1")],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("markdown"),
        "Disabled analyzer should not appear in `analyzers used`: {}",
        stdout
    );
}

/// `enabled = true` (the default) keeps the analyzer active — sanity check that
/// the toggle isn't stuck off.
#[test]
fn analyzer_enabled_true_is_active() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        r#"[processor.markdown2html]
src_dirs = ["."]

[analyzer.markdown]
enabled = true
"#,
    )
    .unwrap();

    fs::write(project_path.join("doc.md"), "# hi\n").unwrap();

    let output = run_rsconstruct_with_env(
        project_path,
        &["analyzers", "used"],
        &[("NO_COLOR", "1")],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("markdown"),
        "Enabled analyzer should appear in `analyzers used`: {}",
        stdout
    );
}

/// Unknown analyzer type must produce a schema error at config-load time,
/// before anything else runs. `toml check` should surface the error.
#[test]
fn analyzer_unknown_type_is_config_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        r#"[processor.markdown2html]
src_dirs = ["."]

[analyzer.not_a_real_analyzer]
"#,
    )
    .unwrap();

    let output = run_rsconstruct_with_env(
        project_path,
        &["toml", "check"],
        &[("NO_COLOR", "1")],
    );
    assert!(!output.status.success(), "Config with unknown analyzer must fail validation");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("not_a_real_analyzer") && combined.contains("unknown analyzer"),
        "Error should name the unknown analyzer: {}", combined
    );
}

/// Unknown field in a known analyzer must produce a schema error listing the
/// valid fields, so the user can spot the typo.
#[test]
fn analyzer_unknown_field_is_config_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        r#"[processor.markdown2html]
src_dirs = ["."]

[analyzer.markdown]
enabeld = false
"#,
    )
    .unwrap();

    let output = run_rsconstruct_with_env(
        project_path,
        &["toml", "check"],
        &[("NO_COLOR", "1")],
    );
    assert!(!output.status.success(), "Config with unknown analyzer field must fail validation");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("enabeld") && combined.contains("unknown field"),
        "Error should name the typo field: {}", combined
    );
    assert!(
        combined.contains("enabled"),
        "Error should list valid fields to help fix the typo: {}", combined
    );
}

/// Omitting `enabled` entirely must default to true (backward-compatible with
/// existing rsconstruct.toml files that predate the field).
#[test]
fn analyzer_enabled_defaults_to_true() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        r#"[processor.markdown2html]
src_dirs = ["."]

[analyzer.markdown]
"#,
    )
    .unwrap();

    fs::write(project_path.join("doc.md"), "# hi\n").unwrap();

    let output = run_rsconstruct_with_env(
        project_path,
        &["analyzers", "used"],
        &[("NO_COLOR", "1")],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("markdown"),
        "Analyzer with no `enabled` field should default to active: {}",
        stdout
    );
}

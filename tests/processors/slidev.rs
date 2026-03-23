use std::fs;
use tempfile::TempDir;
use crate::common::{run_rsconstruct_with_env, tool_available};

#[test]
fn slidev_no_project_discovered() {
    if !tool_available("slidev") {
        eprintln!("slidev not found, skipping test");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    // Enable only slidev (no .md files)
    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor]\nenabled = [\"slidev\"]\n",
    )
    .unwrap();

    let output = run_rsconstruct_with_env(project_path, &["build", "-v"], &[("NO_COLOR", "1")]);
    assert!(
        output.status.success(),
        "Build should succeed with no .md files: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

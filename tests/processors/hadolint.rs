use std::fs;
use tempfile::TempDir;
use crate::common::{run_rsconstruct_with_env, tool_available};

#[test]
fn hadolint_valid_dockerfile() {
    if !tool_available("hadolint") {
        eprintln!("hadolint not found, skipping test");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor]\nenabled = [\"hadolint\"]\n",
    )
    .unwrap();

    fs::write(
        project_path.join("Dockerfile"),
        "FROM ubuntu:22.04\nRUN apt-get update\n",
    )
    .unwrap();

    let output = run_rsconstruct_with_env(project_path, &["build", "-v"], &[("NO_COLOR", "1")]);
    assert!(
        output.status.success(),
        "Build should succeed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn hadolint_incremental_skip() {
    if !tool_available("hadolint") {
        eprintln!("hadolint not found, skipping test");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor]\nenabled = [\"hadolint\"]\n",
    )
    .unwrap();

    fs::write(
        project_path.join("Dockerfile"),
        "FROM ubuntu:22.04\nRUN apt-get update\n",
    )
    .unwrap();

    let output1 = run_rsconstruct_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(output1.status.success());

    let output2 = run_rsconstruct_with_env(project_path, &["build", "--verbose"], &[("NO_COLOR", "1")]);
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("[hadolint] Skipping (unchanged):"),
        "Second build should skip: {}",
        stdout2
    );
}

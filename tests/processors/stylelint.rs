use std::fs;
use tempfile::TempDir;
use crate::common::{run_rsconstruct_with_env, tool_available};

#[test]
fn stylelint_valid_css() {
    if !tool_available("stylelint") {
        eprintln!("stylelint not found, skipping test");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor]\nenabled = [\"stylelint\"]\n",
    )
    .unwrap();

    fs::write(
        project_path.join(".stylelintrc.json"),
        "{\"rules\": {}}\n",
    )
    .unwrap();

    fs::write(
        project_path.join("test.css"),
        "body {\n  color: red;\n}\n",
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
fn stylelint_incremental_skip() {
    if !tool_available("stylelint") {
        eprintln!("stylelint not found, skipping test");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("rsconstruct.toml"),
        "[processor]\nenabled = [\"stylelint\"]\n",
    )
    .unwrap();

    fs::write(project_path.join(".stylelintrc.json"), "{\"rules\": {}}\n").unwrap();
    fs::write(project_path.join("test.css"), "body {\n  color: red;\n}\n").unwrap();

    let output1 = run_rsconstruct_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(output1.status.success());

    let output2 = run_rsconstruct_with_env(project_path, &["build", "--verbose"], &[("NO_COLOR", "1")]);
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("[stylelint] Skipping (unchanged):"),
        "Second build should skip: {}",
        stdout2
    );
}

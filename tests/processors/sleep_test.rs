use std::fs;
use crate::common::{setup_test_project, run_rsb, run_rsb_with_env};

#[test]
fn sleep_processor() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // Create sleep directory and a sleep file with a short duration
    fs::create_dir_all(project_path.join("sleep")).unwrap();
    fs::write(project_path.join("sleep/quick.sleep"), "0.1").unwrap();

    // Enable only sleep processor (disable template and lint to avoid needing their dirs)
    fs::write(
        project_path.join("rsb.toml"),
        "[processor]\nenabled = [\"sleep\"]\n"
    ).unwrap();

    // Build
    let output = run_rsb_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(), "rsb build failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[sleep] Processing:"));

    // Verify stub file was created
    let stub_path = project_path.join("out/sleep/quick.done");
    assert!(stub_path.exists(), "Sleep stub file was not created");
    let stub_content = fs::read_to_string(&stub_path).unwrap();
    assert!(stub_content.contains("slept for 0.1 seconds"));

    // Second build should skip (incremental)
    let output2 = run_rsb_with_env(project_path, &["build", "--verbose"], &[("NO_COLOR", "1")]);
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("[sleep] Skipping (unchanged):"));

    // Clean should remove stub files
    let clean_output = run_rsb(project_path, &["clean"]);
    assert!(clean_output.status.success());
    assert!(!stub_path.exists(), "Sleep stub should be removed after clean");
}

#[test]
fn sleep_extra_inputs_valid() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // Create sleep directory and file
    fs::create_dir_all(project_path.join("sleep")).unwrap();
    fs::write(project_path.join("sleep/test.sleep"), "0.01").unwrap();

    // Create an extra input file
    fs::write(project_path.join("extra.txt"), "extra data").unwrap();

    // Configure sleep processor with extra_inputs
    fs::write(
        project_path.join("rsb.toml"),
        "[processor]\nenabled = [\"sleep\"]\n\n[processor.sleep]\nextra_inputs = [\"extra.txt\"]\n"
    ).unwrap();

    let output = run_rsb_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(output.status.success(),
        "Build should succeed with valid sleep extra_inputs: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    assert!(project_path.join("out/sleep/test.done").exists(),
        "Sleep stub should be created");
}

#[test]
fn sleep_extra_inputs_nonexistent_fails() {
    let temp_dir = setup_test_project();
    let project_path = temp_dir.path();

    // Create sleep directory and file
    fs::create_dir_all(project_path.join("sleep")).unwrap();
    fs::write(project_path.join("sleep/test.sleep"), "0.01").unwrap();

    // Configure sleep processor with nonexistent extra_input
    fs::write(
        project_path.join("rsb.toml"),
        "[processor]\nenabled = [\"sleep\"]\n\n[processor.sleep]\nextra_inputs = [\"does_not_exist.txt\"]\n"
    ).unwrap();

    let output = run_rsb_with_env(project_path, &["build"], &[("NO_COLOR", "1")]);
    assert!(!output.status.success(),
        "Build should fail with nonexistent sleep extra_input: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("extra_inputs file not found") || stderr.contains("does_not_exist.txt"),
        "Error should mention missing extra_inputs file: {}", stderr);
}

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Emit standard vergen variables (commit hash, branch, etc.)
    // NOTE: BuildBuilder is intentionally excluded — it emits VERGEN_BUILD_TIMESTAMP
    // which changes every second, breaking incremental builds.
    let cargo = vergen_gix::CargoBuilder::all_cargo()?;
    let gix = vergen_gix::GixBuilder::all_git()?;
    let rustc = vergen_gix::RustcBuilder::all_rustc()?;
    vergen_gix::Emitter::default()
        .add_instructions(&cargo)?
        .add_instructions(&gix)?
        .add_instructions(&rustc)?
        .emit()?;

    // Emit git describe (tag + distance + short hash, e.g. v0.5.0-3-gabcdef1)
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output();
    let describe = match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_owned()
        }
        _ => String::from("unknown"),
    };
    println!("cargo:rustc-env=RSB_GIT_DESCRIBE={describe}");

    // NOTE: vergen already emits proper rerun-if-changed directives for
    // .git/HEAD and .git/refs/heads/<branch>. Do NOT add manual directives
    // for directories like .git/refs/heads/ — cargo treats directories as
    // always-changed, which breaks incremental builds.

    Ok(())
}

# Crates.io Publishing

Notes on publishing rsconstruct to [crates.io](https://crates.io/crates/rsconstruct).

## Version Limits

There is no limit on how many versions can be published to crates.io. You can publish as many releases as needed without worrying about quota or cleanup.

## Pruning Old Releases

Crates.io does not support deleting published versions. Once a version is uploaded, it exists permanently.

The only removal mechanism is **yanking** (`cargo yank --version 0.1.0`), which:

- Prevents new projects from adding a dependency on the yanked version
- Does **not** break existing projects that already depend on it (they continue to download it via their lockfile)
- Does **not** delete the crate data from the registry

Yanking should only be used for versions with security vulnerabilities or serious bugs, not for general housekeeping.

## Publishing a New Version

1. Update the version in `Cargo.toml`
2. Run `cargo publish --dry-run` to verify
3. Run `cargo publish` to upload

# Suggestions

Ideas for future improvements, organized by category.

## Missing Test Coverage

### No ruff/pylint processor tests
- `tests/processors/` has tests for cc, sleep, spellcheck, and template, but not for ruff or pylint.
- Add integration tests for both Python linting processors.

### No make processor tests
- `tests/processors/` has no tests for the make processor.
- Add integration tests covering Makefile discovery and execution.

## New Processors

### Linting / Checking (stub-based)

#### yamllint
- Lint YAML files (`.yml`, `.yaml`) using `yamllint`.
- Catches syntax errors and style violations.
- Config: `linter` (default `"yamllint"`), `args`, `extra_inputs`, `scan`.

#### jsonlint
- Validate JSON files (`.json`) for syntax errors.
- Could use `python3 -m json.tool` or a dedicated tool like `jsonlint`.
- Config: `linter`, `args`, `extra_inputs`, `scan`.

#### toml-lint
- Validate TOML files (`.toml`) for syntax errors.
- Could use `taplo check` or a built-in Rust parser.
- Config: `linter` (default `"taplo"`), `args`, `extra_inputs`, `scan`.

#### markdownlint
- Lint Markdown files (`.md`) for structural issues (complements spellcheck which only checks spelling).
- Uses `mdl` or `markdownlint-cli`.
- Config: `linter` (default `"mdl"`), `args`, `extra_inputs`, `scan`.

#### mypy
- Python type checking using `mypy`.
- Batch-capable like ruff/pylint.
- Config: `args`, `extra_inputs`, `scan`.

#### black-check
- Python formatting verification using `black --check`.
- Verifies files are formatted without modifying them.
- Config: `args`, `extra_inputs`, `scan`.

### Compilation / Generation

#### rust_single_file
- Compile single-file Rust programs (`.rs`) to executables, like cc_single_file but for Rust.
- Useful for exercise/example repositories.
- Config: `rustc` (default `"rustc"`), `flags`, `output_suffix`, `extra_inputs`, `scan`.

#### sass
- Compile `.scss`/`.sass` files to `.css`.
- Single-file transformation using `sass` or `dart-sass`.
- Config: `compiler` (default `"sass"`), `args`, `extra_inputs`, `scan`.

#### protobuf
- Compile `.proto` files to generated code using `protoc`.
- Config: `protoc` (default `"protoc"`), `args`, `language` (default `"cpp"`), `extra_inputs`, `scan`.

#### pandoc
- Convert Markdown (`.md`) to other formats (PDF, HTML, EPUB) using `pandoc`.
- Single-file transformation.
- Config: `output_format` (default `"html"`), `args`, `extra_inputs`, `scan`.

### Testing

#### pytest
- Run Python test files and produce pass/fail stubs.
- Each `test_*.py` file becomes a product.
- Config: `runner` (default `"pytest"`), `args`, `extra_inputs`, `scan` (default extensions `["test_*.py"]`).

#### doctest
- Run Python doctests and produce stubs.
- Each `.py` file with doctests produces a stub.
- Config: `args`, `extra_inputs`, `scan`.

## Build Execution

### Remote/distributed caching
- Share the `.rsb/` cache across machines via an HTTP API or cloud storage (S3, GCS).
- Highest-impact feature from modern build systems. Every CI run and every developer benefits from artifacts built by others.
- Bazel, Buck2, Pants, Nx, and Turborepo all support this.

### Sandboxed execution
- Run each processor in an isolated environment where it can only access its declared inputs.
- Prevents accidental undeclared dependencies (e.g., a linter reading a file that isn't listed as an input).
- Bazel and Buck2 both enforce this. On Linux, namespaces can provide lightweight sandboxing without container overhead.

### Content-addressable outputs (unchanged output pruning)
- Currently rsb hashes inputs to detect staleness. Hashing outputs too would allow skipping downstream rebuilds when an input changes but produces identical output (e.g., reformatting a comment in a C file that doesn't change the compiled binary).
- Bazel calls this "unchanged output pruning."

## Graph & Query

### Build graph query language
- Bazel has `bazel query`, `cquery`, and `aquery` for exploring the dependency graph.
- rsb could support queries like:
  - `rsb query deps out/template/foo.py` — what does this product depend on?
  - `rsb query rdeps src/main.c` — what products are affected if this file changes?
  - `rsb query processor:ruff` — list all ruff products
- Useful for debugging builds and for CI systems that want to build only affected targets.

### Affected analysis
- Given a set of changed files (e.g., from `git diff`), determine which products are affected and only build those.
- Nx and Pants both feature this prominently.
- Useful for large projects where a full build is expensive but most changes only affect a subset.

## Extensibility

### Plugin/custom processor system
- Currently processors are compiled into rsb. A plugin system would let users define custom processors without forking rsb.
- Options range from simple (shell-script-based processors defined in `rsb.toml`) to full (dynamic library loading or WASM plugins).
- Pants uses a Python-based plugin API. A lightweight version could be:
  ```toml
  [[processor.custom]]
  name = "eslint"
  command = "eslint {input}"
  extensions = [".js", ".ts"]
  stub_dir = "out/eslint"
  ```

### Rule composition / aspects
- Bazel's "aspects" let you attach cross-cutting behavior to all targets of a certain type (e.g., "add coverage analysis to every C++ compile").
- rsb could support something similar — e.g., automatically lint everything that gets compiled.

## Developer Experience

### Structured build logs
- Machine-readable output (JSON lines) for CI integration.
- Includes product name, processor, duration, status, error output.
- Makes it easy to build dashboards or feed into observability systems.

### Build profiling / tracing
- Beyond `--timings`, generate a Chrome trace format or flamegraph SVG showing exactly what ran when, including parallel lanes.
- Bazel generates `--profile` output viewable in Chrome's `chrome://tracing`.
- Invaluable for diagnosing slow builds.

### Actionable error messages
- When a product fails, show context: which processor, which input file, the exact command that was run.
- Include suggestions (e.g., "shellcheck not found — install with `apt install shellcheck`").

### Progress indicator
- For parallel builds, show a status line like `[3/17] Building... (2 running)` instead of just streaming output.
- Ninja and Buck2 both do this well.

## Caching & Performance

### Deferred materialization
- Don't write cached outputs to disk until they're actually needed by a downstream product or the final build result.
- For large graphs with deep caching, this avoids writing files that are never used.
- Buck2 does this aggressively.

### Garbage collection policy
- Currently `rsb cache trim` removes unreferenced objects.
- Add time-based or size-based policies: "keep cache under 1GB" or "evict entries older than 30 days."
- Useful for CI environments with limited disk.

### Shared cache across branches
- When switching git branches, products built on another branch should be restorable from cache if their inputs match.
- This already works implicitly if the input hash matches, but it could be surfaced in `rsb status` ("restorable from branch X").

## Reproducibility

### ~~Tool version locking~~ *(Done)*
- Each processor declares the tools it needs via `required_tools()` and how to query their version via `tool_version_commands()`.
- `rsb tools lock` queries each enabled processor's tools for their version, resolves the full binary path, and writes `.tools.versions` (JSON) to the project root. This file should be committed to version control.
- Lock file format includes schema version, timestamp, and per-tool entries with resolved path, version output, and the arguments used to obtain the version.
- On every `rsb build`, rsb reads `.tools.versions` and checks that installed tool versions match. Mismatch is a hard error by default; `--ignore-tool-versions` overrides this.
- If `.tools.versions` does not exist, `rsb build` warns and suggests running `rsb tools lock`.
- Tool versions are included in the cache key hash for each processor, so upgrading a tool and re-locking automatically invalidates cached outputs.
- Only tools for enabled processors are included in the lock file. Adding a new processor to `enabled` requires re-locking.
- Version comparison uses raw output strings (not parsed semver) to handle the wide variety of version output formats across tools.
- The lock file stores the resolved binary path so switching between system and local installs is detected.
- `rsb tools lock --check` verifies without writing. Bare `rsb tools lock` writes/updates the lock file.
- Inspired by Bazel's explicit toolchain management.

### Determinism verification
- A `rsb build --verify` mode that builds each product twice and compares outputs.
- If they differ, the build is non-deterministic.
- Bazel has `--experimental_check_output_files` for similar purposes.

## Security

### Shell command execution from source file comments
- `src/processors/cc.rs` — `EXTRA_*_SHELL` directives execute arbitrary shell commands parsed from source file comments.
- Document the security implications clearly.

# Suggestions

Ideas for future improvements, organized by category.
Completed items have been moved to [suggestions-done.md](suggestions-done.md).

Grades:
- **Urgency**: `high` (users need this), `medium` (nice to have), `low` (speculative/future)
- **Complexity**: `low` (hours), `medium` (days), `high` (weeks+)

## Missing Test Coverage

### No ruff/pylint processor tests
- `tests/processors/` has tests for cc, sleep, spellcheck, and template, but not for ruff or pylint.
- Add integration tests for both Python linting processors.
- **Urgency**: high | **Complexity**: low

### No make processor tests
- `tests/processors/` has no tests for the make processor.
- Add integration tests covering Makefile discovery and execution.
- **Urgency**: high | **Complexity**: low

## New Processors

### Linting / Checking (stub-based)

#### yamllint
- Lint YAML files (`.yml`, `.yaml`) using `yamllint`.
- Catches syntax errors and style violations.
- Config: `linter` (default `"yamllint"`), `args`, `extra_inputs`, `scan`.
- **Urgency**: medium | **Complexity**: low

#### jsonlint
- Validate JSON files (`.json`) for syntax errors.
- Could use `python3 -m json.tool` or a dedicated tool like `jsonlint`.
- Config: `linter`, `args`, `extra_inputs`, `scan`.
- **Urgency**: medium | **Complexity**: low

#### toml-lint
- Validate TOML files (`.toml`) for syntax errors.
- Could use `taplo check` or a built-in Rust parser.
- Config: `linter` (default `"taplo"`), `args`, `extra_inputs`, `scan`.
- **Urgency**: low | **Complexity**: low

#### markdownlint
- Lint Markdown files (`.md`) for structural issues (complements spellcheck which only checks spelling).
- Uses `mdl` or `markdownlint-cli`.
- Config: `linter` (default `"mdl"`), `args`, `extra_inputs`, `scan`.
- **Urgency**: low | **Complexity**: low

#### black-check
- Python formatting verification using `black --check`.
- Verifies files are formatted without modifying them.
- Config: `args`, `extra_inputs`, `scan`.
- **Urgency**: low | **Complexity**: low

### Compilation / Generation

#### rust_single_file
- Compile single-file Rust programs (`.rs`) to executables, like cc_single_file but for Rust.
- Useful for exercise/example repositories.
- Config: `rustc` (default `"rustc"`), `flags`, `output_suffix`, `extra_inputs`, `scan`.
- **Urgency**: medium | **Complexity**: medium

#### sass
- Compile `.scss`/`.sass` files to `.css`.
- Single-file transformation using `sass` or `dart-sass`.
- Config: `compiler` (default `"sass"`), `args`, `extra_inputs`, `scan`.
- **Urgency**: low | **Complexity**: low

#### protobuf
- Compile `.proto` files to generated code using `protoc`.
- Config: `protoc` (default `"protoc"`), `args`, `language` (default `"cpp"`), `extra_inputs`, `scan`.
- **Urgency**: low | **Complexity**: medium

#### pandoc
- Convert Markdown (`.md`) to other formats (PDF, HTML, EPUB) using `pandoc`.
- Single-file transformation.
- Config: `output_format` (default `"html"`), `args`, `extra_inputs`, `scan`.
- **Urgency**: low | **Complexity**: low

### Testing

#### pytest
- Run Python test files and produce pass/fail stubs.
- Each `test_*.py` file becomes a product.
- Config: `runner` (default `"pytest"`), `args`, `extra_inputs`, `scan` (default extensions `["test_*.py"]`).
- **Urgency**: medium | **Complexity**: medium

#### doctest
- Run Python doctests and produce stubs.
- Each `.py` file with doctests produces a stub.
- Config: `args`, `extra_inputs`, `scan`.
- **Urgency**: low | **Complexity**: medium

## Build Execution

### Distributed builds
- Run builds across multiple machines, similar to distcc or icecream for C/C++.
- A coordinator node distributes work to worker nodes, each running rsb in worker mode.
- Workers execute products and return outputs to the coordinator, which caches them locally.
- Challenges: network overhead for small products, identical tool versions across workers, local filesystem access.
- **Urgency**: low | **Complexity**: high

### Sandboxed execution
- Run each processor in an isolated environment where it can only access its declared inputs.
- Prevents accidental undeclared dependencies.
- On Linux, namespaces can provide lightweight sandboxing.
- **Urgency**: low | **Complexity**: high

### Content-addressable outputs (unchanged output pruning)
- Hash outputs too to skip downstream rebuilds when an input changes but produces identical output.
- Bazel calls this "unchanged output pruning."
- **Urgency**: medium | **Complexity**: medium

### Persistent daemon mode
- Keep rsb running as a background daemon to avoid startup overhead.
- Benefits: instant file index via inotify, warm Lua VMs, connection pooling, faster incremental builds.
- Daemon listens on Unix socket (`.rsb/daemon.sock`).
- `rsb watch` becomes a client that triggers rebuilds on file events.
- **Urgency**: low | **Complexity**: high

### Build profiles
- Named configuration sets for different build scenarios (ci, dev, release).
- Profiles inherit from base configuration and override specific values.
- Usage: `rsb build --profile=ci`
- **Urgency**: medium | **Complexity**: medium

### Conditional processors
- Enable or disable processors based on conditions (environment variables, file existence, git branch, custom commands).
- Multiple conditions can be combined with `all`/`any` logic.
- **Urgency**: low | **Complexity**: medium

### Target aliases
- Define named groups of processors for easy invocation.
- Usage: `rsb build @lint`, `rsb build @test`
- Special aliases: `@all`, `@changed`, `@failed`
- File-based targeting: `rsb build src/main.c`
- **Urgency**: medium | **Complexity**: medium

## Graph & Query

### Build graph query language
- Support queries like `rsb query deps out/foo`, `rsb query rdeps src/main.c`, `rsb query processor:ruff`.
- Useful for debugging builds and CI systems that want to build only affected targets.
- **Urgency**: low | **Complexity**: medium

### Affected analysis
- Given changed files (from `git diff`), determine which products are affected and only build those.
- Useful for large projects where a full build is expensive.
- **Urgency**: medium | **Complexity**: medium

## Extensibility

### Plugin registry
- A central repository of community-contributed Lua plugins.
- Install with `rsb plugin install eslint`.
- Registry could be a GitHub repository with a JSON index.
- Version pinning in `rsb.toml`.
- **Urgency**: low | **Complexity**: high

### Project templates
- Initialize new projects with pre-configured processors and directory structure.
- `rsb init --template=python`, `rsb init --template=cpp`, etc.
- Custom templates from local directories or URLs.
- **Urgency**: low | **Complexity**: medium

### Rule composition / aspects
- Attach cross-cutting behavior to all targets of a certain type (e.g., "add coverage analysis to every C++ compile").
- **Urgency**: low | **Complexity**: high

## Developer Experience

### Build profiling / tracing
- Generate Chrome trace format or flamegraph SVG showing what ran when, including parallel lanes.
- Usage: `rsb build --trace=build.json`
- **Urgency**: medium | **Complexity**: medium

### Build notifications
- Desktop notifications when builds complete, especially for long builds.
- Platform-specific: `notify-send` (Linux), `osascript` (macOS).
- Config: `notify = true`, `notify_on_success = false`.
- **Urgency**: low | **Complexity**: low

### Actionable error messages
- When a product fails, include suggestions (e.g., "shellcheck not found — install with `apt install shellcheck`").
- **Urgency**: medium | **Complexity**: low

### `rsb build <target>` — Build specific targets
- Build only specific targets by name or pattern:
  `rsb build src/main.c`, `rsb build out/cc_single_file/`, `rsb build "*.py"`
- **Urgency**: medium | **Complexity**: medium

### Parallel dependency analysis
- The cpp analyzer scans files sequentially, which can be slow for large codebases.
- Parallelize header scanning using rayon or tokio.
- **Urgency**: low | **Complexity**: medium

### IDE / LSP integration
- Language Server Protocol server for IDE integration.
- Features: diagnostics, code actions, hover info, file decorations.
- Plugins for VS Code, Neovim, Emacs.
- **Urgency**: low | **Complexity**: high

### Build log capture
- Save stdout/stderr from each product execution to a log file.
- Config: `log_dir = ".rsb/logs"`, `log_retention = 10`.
- `rsb log ruff:main.py` to view logs.
- **Urgency**: low | **Complexity**: medium

### Build timing history
- Store timing data to `.rsb/timings.json` after each build.
- `rsb timings` shows slowest products, trends, time per processor.
- **Urgency**: low | **Complexity**: medium

### Remote cache authentication
- Support authenticated remote caches: S3 (AWS credentials), HTTP (bearer tokens), GCS.
- Variable substitution from environment for secrets.
- **Urgency**: medium | **Complexity**: medium

### `rsb lint` — Run only checkers
- Convenience command to run only checker processors.
- Equivalent to `rsb build -p ruff,pylint,...` but shorter.
- **Urgency**: low | **Complexity**: low

### `--quiet` flag
- Suppress all output except errors.
- Useful for CI scripts that only care about exit code.
- **Urgency**: medium | **Complexity**: low

### Watch mode keyboard commands
- During `rsb watch`, support `r` (rebuild), `c` (clean), `q` (quit), `Enter` (rebuild now), `s` (status).
- Only activate when stdin is a TTY.
- **Urgency**: low | **Complexity**: medium

## Caching & Performance

### Lazy file hashing (mtime-based)
- Only re-hash files whose mtime has changed since the last build.
- Store `(path, mtime, checksum)` tuples in cache database.
- Config: `mtime_cache = true`.
- Risk: mtime can be unreliable. `--force` bypasses this.
- **Urgency**: medium | **Complexity**: medium

### Compressed cache objects
- Compress cached objects with zstd to reduce disk usage and remote transfer times.
- Config: `compression = "zstd"`, `compression_level = 3`.
- Typical savings: 50-80% for text files.
- **Urgency**: low | **Complexity**: medium

### Deferred materialization
- Don't write cached outputs to disk until they're actually needed by a downstream product.
- **Urgency**: low | **Complexity**: high

### Garbage collection policy
- Time-based or size-based cache policies: "keep cache under 1GB" or "evict entries older than 30 days."
- Config: `max_size = "1GB"`, `max_age = "30d"`, `gc_policy = "lru"`.
- `rsb cache gc` for manual garbage collection.
- **Urgency**: low | **Complexity**: medium

### Shared cache across branches
- Surface in `rsb status` when products are restorable from another branch.
- Already works implicitly via input hash matching.
- **Urgency**: low | **Complexity**: low

## Reproducibility

### Hermetic builds
- Control all inputs beyond tool versions: isolate env vars, control timestamps, sandbox network, pin system libraries.
- Config: `hermetic = true`, `allowed_env = ["HOME", "PATH"]`.
- Verification: `rsb build --verify` builds twice and compares outputs.
- **Urgency**: low | **Complexity**: high

### Determinism verification
- `rsb build --verify` mode that builds each product twice and compares outputs.
- **Urgency**: low | **Complexity**: medium

## Security

### Shell command execution from source file comments
- `EXTRA_*_SHELL` directives execute arbitrary shell commands parsed from source file comments.
- Document the security implications clearly.
- **Urgency**: medium | **Complexity**: low

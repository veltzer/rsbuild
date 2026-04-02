# Completed Suggestions

Items from `suggestions.md` that have been implemented.

## Completed Features

- **Remote caching** — See [Remote Caching](remote-caching.md). Share build artifacts across machines via S3, HTTP, or filesystem.
- **Lua plugin system** — See [Lua Plugins](plugins.md). Define custom processors in Lua without forking rsconstruct.
- **Tool version locking** — `rsconstruct tools lock` locks and verifies external tool versions. Tool versions are included in cache keys.
- **JSON output mode** — `--json` flag for machine-readable JSON Lines output (build_start, product_start, product_complete, build_summary events).
- **Native C/C++ include scanner** — Default `include_scanner = "native"` uses regex-based scanning. Falls back to `include_scanner = "compiler"` (gcc -MM).
- **`--processors` flag** — `rsconstruct build -p tera,ruff` and `rsconstruct watch -p tera` filter which processors run.
- **Colored diff on config changes** — When processor config changes trigger rebuilds, rsconstruct shows what changed with colored diff output.
- **Batch processing** — ruff, pylint, shellcheck, zspell, mypy, and rumdl all support batch execution via `execute_batch()`.
- **Progress bar** — Uses `indicatif` crate. Progress bar sized to actual work (excludes instant skips), hidden in verbose/JSON mode.
- **Emit `ProductStart` JSON events** — Emitted before each product starts executing, pairs with `ProductComplete` for per-product timing.
- **mypy processor** — Python type checking with mypy. Batch-capable. Auto-detects `mypy.ini` as extra input.
- **Explain commands** — `--explain` flag shows skip/restore/rebuild reasons for each product during build.

## Completed Code Consolidation

- **Collapsed `checker_config!` macro variants** — Merged `@basic`, `@with_auto_inputs`, and `@with_linter` into two internal variants (`@no_linter` and `@with_linter`).
- **Added `batch` field to all manually-defined processor configs** — All processor configs now support `batch = false` to disable batching per-project.
- **Replaced trivial checker files with `simple_checker!` macro** — 25 trivial checkers reduced from ~35 lines each to 3-5 lines (~800 lines eliminated).
- **Unified `lint_files`/`check_files` naming** — All checkers now use `check_files` consistently.
- **Moved `should_process` guard into macro** — Added `guard: scan_root` built-in to `impl_checker!`, removed boilerplate `should_process()` from 7 processors.
- **Simplified `KnownFields`** — Scan config fields auto-appended by validation layer via `SCAN_CONFIG_FIELDS` constant; `KnownFields` impls only list their own fields.
- **Extracted `WordManager` for spellcheck/aspell** — Shared word-file management (loading, collecting, flushing, execute/batch patterns) in `word_manager.rs`.

## Completed New Processors

- **mypy** — Python type checking using `mypy`. Batch-capable. Config: `checker`, `args`, `extra_inputs`, `scan`.
- **yamllint** — Lint YAML files using `yamllint`. `src/processors/checkers/yamllint.rs`.
- **jsonlint** — Validate JSON files for syntax errors. `src/processors/checkers/jsonlint.rs`.
- **taplo (toml-lint)** — Validate TOML files using `taplo`. `src/processors/checkers/taplo.rs`.
- **markdownlint** — Lint Markdown files for structural issues. Uses `mdl` or `markdownlint-cli`.
- **pandoc** — Convert Markdown to other formats (PDF, HTML, EPUB). Generator processor.
- **jinja2** — Render Jinja2 templates (`.j2`) via Python jinja2 library. `src/processors/generators/jinja2.rs`.
- **black** — Python formatting verification using `black --check`. `src/processors/checkers/black.rs`.
- **rust_single_file** — Compile single-file Rust programs to executables. `src/processors/generators/rust_single_file.rs`.
- **sass** — Compile SCSS/SASS files to CSS. `src/processors/generators/sass.rs`.
- **protobuf** — Compile `.proto` files to generated code using `protoc`. `src/processors/generators/protobuf.rs`.
- **pytest** — Run Python test files with pytest. `src/processors/checkers/pytest.rs`.
- **doctest** — Run Python doctests via `python3 -m doctest`. `src/processors/checkers/doctest.rs`.

## Completed Test Coverage

- **Ruff/pylint processor tests** — `tests/processors/ruff.rs` and `tests/processors/pylint.rs` with integration tests.
- **Make processor tests** — `tests/processors/make.rs` with Makefile discovery and execution tests.
- **All generator processor tests** — Integration tests for all 14 previously untested generators: a2x, drawio, gem, libreoffice, markdown, marp, mermaid, npm, pandoc, pdflatex, pdfunite, pip, sphinx.
- **All checker processor tests** — Integration tests for all 5 previously untested checkers: ascii, aspell, markdownlint, mdbook, mdl.

## Completed Caching & Performance

- **Lazy file hashing (mtime-based)** — `mtime_check` config (default `true`), `fast_checksum()` with MTIME_TABLE. Stores `(path, mtime, checksum)` tuples. Disable with `--no-mtime`.
- **Compressed cache objects** — Optional zstd compression for `.rsconstruct/objects/`. Config: `compression = true` in `[cache]`. Incompatible with hardlink restore (must use `restore_method = "copy"`). Checksums computed on original content for stable cache keys.

## Completed Developer Experience

- **`--quiet` flag** — `-q`/`--quiet` suppresses all output except errors. Useful for CI scripts that only care about exit code.
- **Flaky product detection / retry** — `--retry=N` retries failed products up to N times. Reports FLAKY (passed on retry) vs FAILED status in build summary.
- **Actionable error messages** — `rsconstruct tools check` shows install hints for missing tools (e.g., "install with: pip install ruff").
- **Build profiling / tracing** — `--trace=file.json` generates Chrome trace format output viewable in `chrome://tracing` or Perfetto UI.
- **`rsconstruct build <target>`** — Build specific targets by name or pattern via `--target` glob patterns and `-d/--dir` flags.
- **`rsconstruct why <file>` / Explain rebuilds** — `--explain` flag shows why each product is skipped, restored, or rebuilt.
- **`rsconstruct doctor`** — Diagnose build environment: checks config, tools, and versions. Full implementation in `src/builder/doctor.rs`.
- **`rsconstruct sloc`** — Source lines of code statistics with COCOMO effort/cost estimation. `src/builder/sloc.rs`.

## Completed Quick Wins

- **Batch processing for more processors** — All checker processors that support multiple file arguments now use batching.
- **Progress bar for long builds** — Implemented with `indicatif`, shows `[elapsed] [bar] pos/len message`.
- **`--processors` flag for build and watch** — Filter processors with `-p` flag.
- **Emit `ProductStart` JSON events** — Wired up and emitted before execution.
- **Colored diff on config changes** — Shows colored JSON diff when processor config changes.

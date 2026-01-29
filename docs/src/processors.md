# Processors

RSB uses **processors** to discover and build products. Each processor scans for source files matching its conventions and produces output files.

Enable processors in `rsb.toml`:

```toml
[processor]
enabled = ["template", "pylint", "cc", "cpplint"]
```

Use `rsb processor list` to see available processors and their status.

## Template processor

The template processor renders [Tera](https://keats.github.io/tera/) templates into output files.

**Convention:** `templates/{X}.tera` produces `{X}` in the project root.

### Configuration

```toml
[processor.template]
strict = true           # Fail on undefined variables (default: true)
extensions = [".tera"]  # File extensions to process
trim_blocks = false     # Remove newline after block tags
```

### Loading Python config

Templates can load variables from Python files using the built-in `load_python()` function:

```jinja2
{% set config = load_python(path="config/settings.py") %}
[app]
name = "{{ config.project_name }}"
version = "{{ config.version }}"
```

## Pylint processor

The pylint processor runs a Python linter on Python source files and produces stub output files to track which files have been linted.

**Convention:** Python files are linted and stubs are written to `out/pylint/`.

### Configuration

```toml
[processor.pylint]
linter = "ruff"   # Python linter command (default: ruff)
args = []          # Extra arguments passed to the linter
```

## Cpplint processor

The cpplint processor runs a C/C++ static analysis tool on source files.

**Convention:** C/C++ source files are analyzed and stubs are written to `out/cpplint/`.

### Configuration

```toml
[processor.cpplint]
checker = "cppcheck"  # Static analysis tool (default: cppcheck)
args = ["--error-exitcode=1", "--enable=warning,style,performance,portability"]
```

To use a suppressions file, add to args:

```toml
args = [
    "--error-exitcode=1",
    "--enable=warning,style,performance,portability",
    "--suppressions-list=.cppcheck-suppressions"
]
```

## C/C++ compiler processor

The cc processor compiles C and C++ source files into executables. Due to its complexity (per-file flags, header tracking, command ordering), it has its own dedicated page: [C/C++ Processor Details](cc-details.md).

## Sleep processor

The sleep processor is used for testing parallel execution. It reads `.sleep` files containing a duration and waits for that amount of time, producing stubs in `out/sleep/`.

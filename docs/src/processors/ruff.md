# Ruff Processor

## Purpose

Lints Python source files using [ruff](https://docs.astral.sh/ruff/).

## How It Works

Discovers `.py` files in the project (excluding common non-source directories),
runs `ruff check` on each file, and creates a stub file on success.
A non-zero exit code from ruff fails the product.

This processor supports batch mode, allowing multiple files to be checked in a
single ruff invocation for better performance.

## Source Files

- Input: `**/*.py`
- Output: `out/ruff/{flat_name}.ruff`

## Configuration

```toml
[processor.ruff]
linter = "ruff"                            # The ruff command to run
args = []                                  # Additional arguments to pass to ruff
extra_inputs = ["pyproject.toml"]          # Additional files that trigger rebuilds when changed
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `linter` | string | `"ruff"` | The ruff executable to run |
| `args` | string[] | `[]` | Extra arguments passed to ruff |
| `extra_inputs` | string[] | `[]` | Extra files whose changes trigger rebuilds |

# Pyrefly Processor

## Purpose

Type-checks Python source files using [pyrefly](https://pyrefly.org/).

## How It Works

Discovers `.py` files in the project (excluding common non-source directories),
runs `pyrefly check` on each file, and records success in the cache.
A non-zero exit code from pyrefly fails the product.

This processor supports batch mode, allowing multiple files to be checked in a
single pyrefly invocation for better performance.

## Source Files

- Input: `**/*.py`
- Output: none (checker)

## Configuration

```toml
[processor.pyrefly]
checker = "pyrefly"                          # The pyrefly command to run
args = []                                    # Additional arguments to pass to pyrefly
extra_inputs = []                            # Additional files that trigger rebuilds when changed
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `checker` | string | `"pyrefly"` | The pyrefly executable to run |
| `args` | string[] | `[]` | Extra arguments passed to pyrefly |
| `extra_inputs` | string[] | `[]` | Extra files whose changes trigger rebuilds |

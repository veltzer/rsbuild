# Shellcheck Processor

## Purpose

Lints shell scripts using [shellcheck](https://www.shellcheck.net/).

## How It Works

Discovers `.sh` and `.bash` files in the project (excluding common build tool
directories), runs `shellcheck` on each file, and records success in the cache.
A non-zero exit code from shellcheck fails the product.

This processor supports batch mode, allowing multiple files to be checked in a
single shellcheck invocation for better performance.

## Source Files

- Input: `**/*.sh`, `**/*.bash`
- Output: none (checker)

## Configuration

```toml
[processor.shellcheck]
checker = "shellcheck"                       # The shellcheck command to run
args = []                                    # Additional arguments to pass to shellcheck
extra_inputs = []                            # Additional files that trigger rebuilds when changed
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `checker` | string | `"shellcheck"` | The shellcheck executable to run |
| `args` | string[] | `[]` | Extra arguments passed to shellcheck |
| `extra_inputs` | string[] | `[]` | Extra files whose changes trigger rebuilds |

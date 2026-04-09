# Mako Processor

## Purpose

Renders Mako template files into output files using the Python Mako template library.

## How It Works

Files matching configured extensions in `templates.mako/` are rendered via `python3` using
the `mako` Python library. Output is written with the extension stripped and the
`templates.mako/` prefix removed:

```
templates.mako/app.config.mako  →  app.config
templates.mako/sub/readme.txt.mako  →  sub/readme.txt
```

Templates use the [Mako](https://www.makotemplates.org/) templating engine. A
`TemplateLookup` is configured with the project root as the lookup directory, so
templates can include or inherit from other templates using relative paths.

## Source Files

- Input: `templates.mako/**/*{src_extensions}`
- Output: project root, mirroring the template path (minus `templates.mako/` prefix) with the extension removed

## Configuration

```toml
[processor.mako]
src_extensions = [".mako"]                    # File extensions to process (default: [".mako"])
dep_inputs = ["config/settings.py"]     # Additional files that trigger rebuilds when changed
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `src_extensions` | string[] | `[".mako"]` | File extensions to discover |
| `dep_inputs` | string[] | `[]` | Extra files whose changes trigger rebuilds |

## Batch support

Each input file is processed individually, producing its own output file.

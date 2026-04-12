# tera

Scans Tera template files for `{% include %}`, `{% import %}`, and `{% extends %}` directives and adds referenced template files as dependencies.

**Native**: Yes.

**Auto-detects**: Projects with `.tera` files.

## Features

- Extracts paths from `{% include "path" %}`, `{% import "path" %}`, and `{% extends "path" %}`
- Handles both double- and single-quoted paths
- Resolves paths relative to the source file's directory and the project root

This ensures that when an included template changes, any template that includes it is rebuilt.

## Configuration

```toml
[analyzer.tera]
# currently no tunables
```

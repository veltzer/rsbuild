# python

Scans Python source files for `import` and `from ... import` statements and adds dependencies on local Python modules.

**Native**: Yes.

**Auto-detects**: Projects with `.py` files.

## Features

- Resolves imports to local files (ignores stdlib / external packages)
- Supports both `import foo` and `from foo import bar` syntax
- Searches relative to the source file and project root

## Configuration

```toml
[analyzer.python]
# currently no tunables
```

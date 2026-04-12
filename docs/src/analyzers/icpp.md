# icpp

Native (no-subprocess) C/C++ dependency analyzer. Scans `#include` directives by parsing source files directly in Rust, without invoking `gcc` or `pkg-config`.

**Native**: Yes.

**Auto-detects**: Projects with `.c`, `.cc`, `.cpp`, `.cxx`, `.h`, `.hh`, `.hpp`, or `.hxx` files.

## When to use

- You want faster analysis without the overhead of launching `gcc` per file.
- You don't need compiler-driven include path discovery.
- You're happy to enumerate include paths explicitly in `rsconstruct.toml`.

Prefer [cpp](cpp.md) if you need compiler-discovered system include paths or pkg-config integration.

## Configuration

```toml
[analyzer.icpp]
include_paths    = ["include", "src"]
src_exclude_dirs = ["/kernel/", "/vendor/"]
```

## See also

- [cpp](cpp.md) — compiler-aware (external) C/C++ dependency analyzer

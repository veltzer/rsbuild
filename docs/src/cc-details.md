# C/C++ Processor Details

The cc processor compiles C (`.c`) and C++ (`.cc`) source files into executables under `out/cc/`, mirroring the source directory structure.

**Example:** `src/a/b.c` produces `out/cc/a/b.elf`

Header dependencies are automatically tracked via `gcc -MMD`, so changes to included headers trigger recompilation.

## Configuration

```toml
[processor.cc]
cc = "gcc"              # C compiler (default: gcc)
cxx = "g++"             # C++ compiler (default: g++)
cflags = ["-Wall"]      # C compiler flags
cxxflags = ["-Wall"]    # C++ compiler flags
ldflags = []            # Linker flags
include_paths = ["src/include"]  # Additional -I paths (passed as-is)
source_dir = "src"      # Source directory (default: src)
output_suffix = ".elf"  # Suffix for output executables (default: .elf)
```

## Per-file flags

Per-file compile and link flags can be set via special comments in source files. This allows individual files to require specific libraries or compiler options without affecting the entire project.

### Flag directives

```c
// EXTRA_COMPILE_FLAGS_BEFORE=-pthread
// EXTRA_COMPILE_FLAGS_AFTER=-O2 -DNDEBUG
// EXTRA_LINK_FLAGS_BEFORE=-L/usr/local/lib
// EXTRA_LINK_FLAGS_AFTER=-lX11
```

### Command directives

Execute a command and use its stdout as flags (no shell):

```c
// EXTRA_COMPILE_CMD=pkg-config --cflags gtk+-3.0
// EXTRA_LINK_CMD=pkg-config --libs gtk+-3.0
```

### Shell directives

Execute via `sh -c` (full shell syntax):

```c
// EXTRA_COMPILE_SHELL=echo -DLEVEL2_CACHE_LINESIZE=$(getconf LEVEL2_CACHE_LINESIZE)
// EXTRA_LINK_SHELL=echo -L$(brew --prefix openssl)/lib
```

### Directive summary

| Directive | Execution | Use case |
|---|---|---|
| `EXTRA_COMPILE_FLAGS_BEFORE` | Literal flags | Flags before default cflags |
| `EXTRA_COMPILE_FLAGS_AFTER` | Literal flags | Flags after default cflags |
| `EXTRA_LINK_FLAGS_BEFORE` | Literal flags | Flags before default ldflags |
| `EXTRA_LINK_FLAGS_AFTER` | Literal flags | Flags after default ldflags |
| `EXTRA_COMPILE_CMD` | Subprocess (no shell) | Dynamic compile flags via command |
| `EXTRA_LINK_CMD` | Subprocess (no shell) | Dynamic link flags via command |
| `EXTRA_COMPILE_SHELL` | `sh -c` (full shell) | Dynamic compile flags needing shell features |
| `EXTRA_LINK_SHELL` | `sh -c` (full shell) | Dynamic link flags needing shell features |

## Supported comment styles

Directives can appear in any of these comment styles:

**C++ style:**
```c
// EXTRA_LINK_FLAGS_AFTER=-lX11
```

**C block comment (single line):**
```c
/* EXTRA_LINK_FLAGS_AFTER=-lX11 */
```

**C block comment (multi-line, star-prefixed):**
```c
/*
 * EXTRA_LINK_FLAGS_AFTER=-lX11
 */
```

## Command line ordering

The compiler command is constructed in this order:

```
compiler -MMD -MF deps -I... [compile_before] [cflags/cxxflags] [compile_after] -o output source [link_before] [ldflags] [link_after]
```

Link flags come **after** the source file so the linker can resolve symbols correctly.

The positional fields map to directives as follows:

| Position | Source |
|---|---|
| `compile_before` | `EXTRA_COMPILE_FLAGS_BEFORE` + `EXTRA_COMPILE_CMD` + `EXTRA_COMPILE_SHELL` |
| `cflags/cxxflags` | `[processor.cc]` config `cflags` or `cxxflags` |
| `compile_after` | `EXTRA_COMPILE_FLAGS_AFTER` |
| `link_before` | `EXTRA_LINK_FLAGS_BEFORE` + `EXTRA_LINK_CMD` + `EXTRA_LINK_SHELL` |
| `ldflags` | `[processor.cc]` config `ldflags` |
| `link_after` | `EXTRA_LINK_FLAGS_AFTER` |

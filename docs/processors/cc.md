# CC Processor

## Purpose

Compiles C (`.c`) and C++ (`.cc`) source files into executables.

## How It Works

Source files under the configured source directory are compiled into executables
under `out/cc/`, mirroring the directory structure:

```
src/main.c       →  out/cc/main.elf
src/a/b.c        →  out/cc/a/b.elf
src/app.cc       →  out/cc/app.elf
```

Header dependencies are tracked automatically via compiler-generated `.d` files
(`-MMD -MF`). When a header changes, all source files that include it are rebuilt.

## Source Files

- Input: `{source_dir}/**/*.c`, `{source_dir}/**/*.cc`
- Output: `out/cc/{relative_path}{output_suffix}`

## Per-File Flags

Per-file compile and link flags can be set via comments in source files:

```c
// EXTRA_COMPILE_FLAGS_BEFORE=-pthread
// EXTRA_COMPILE_FLAGS_AFTER=-O2 -DNDEBUG
// EXTRA_LINK_FLAGS_BEFORE=-L/usr/local/lib
// EXTRA_LINK_FLAGS_AFTER=-lX11
// EXTRA_COMPILE_CMD=pkg-config --cflags gtk+-3.0
// EXTRA_LINK_CMD=pkg-config --libs gtk+-3.0
// EXTRA_COMPILE_SHELL=echo -DLEVEL2_CACHE_LINESIZE=$(getconf LEVEL2_CACHE_LINESIZE)
// EXTRA_LINK_SHELL=echo -L$(brew --prefix openssl)/lib
```

Supported comment styles: `//`, `/* ... */` (single-line), and `*`-prefixed block
comment continuation lines:

```c
/*
 * EXTRA_LINK_FLAGS_AFTER=-lX11
 */
```

- `EXTRA_*_FLAGS_*` — literal flags (with backtick expansion for command substitution)
- `EXTRA_*_CMD` — executed as a subprocess (no shell); stdout becomes flags
- `EXTRA_*_SHELL` — executed via `sh -c` (full shell syntax); stdout becomes flags

## Command Line Ordering

```
compiler -MMD -MF deps -I... [compile_before] [cflags/cxxflags] [compile_after] -o output source [link_before] [ldflags] [link_after]
```

Link flags come after the source file so the linker can resolve symbols correctly.

## Verbosity Levels (`--processor-verbose N`)

| Level | Output |
|-------|--------|
| 0 (default) | Target basename: `main.elf` |
| 1 | Target path + compiler commands: `out/cc/main.elf` |
| 2 | Adds source path: `out/cc/main.elf <- src/main.c` |
| 3 | Adds all inputs: `out/cc/main.elf <- src/main.c, src/utils.h` |

## Configuration

```toml
[processor.cc]
cc = "gcc"                # C compiler (default: "gcc")
cxx = "g++"               # C++ compiler (default: "g++")
cflags = []               # C compiler flags
cxxflags = []             # C++ compiler flags
ldflags = []              # Linker flags
include_paths = []        # Additional -I paths (relative to project root)
source_dir = "src"        # Source directory (default: "src")
output_suffix = ".elf"    # Suffix for output executables (default: ".elf")
extra_inputs = []         # Additional files that trigger rebuilds when changed
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `cc` | string | `"gcc"` | C compiler command |
| `cxx` | string | `"g++"` | C++ compiler command |
| `cflags` | string[] | `[]` | Flags passed to the C compiler |
| `cxxflags` | string[] | `[]` | Flags passed to the C++ compiler |
| `ldflags` | string[] | `[]` | Flags passed to the linker |
| `include_paths` | string[] | `[]` | Additional `-I` include paths |
| `source_dir` | string | `"src"` | Directory to scan for source files |
| `output_suffix` | string | `".elf"` | Suffix appended to output executables |
| `extra_inputs` | string[] | `[]` | Extra files whose changes trigger rebuilds |

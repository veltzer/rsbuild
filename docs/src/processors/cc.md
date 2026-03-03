# CC Project Processor

## Purpose

Builds full C/C++ projects with multiple targets (libraries and executables)
defined in a `cc.yaml` manifest file. Unlike the [CC Single File](cc_single_file.md)
processor which compiles each source file into a standalone executable, this
processor supports multi-file targets with dependency linking.

## How It Works

The processor scans for `cc.yaml` files. Each manifest defines libraries
and programs to build. All compiler and linker commands run with the working
directory set to the `cc.yaml` file's directory, so all paths in the manifest
(sources, include directories, output directory) are relative to that location —
just like Make operates from the Makefile's directory.

Source files are compiled to object files, then linked into the final targets:

```
cc.yaml defines:
  library "mymath" (static) from src/math.c, src/utils.c
  program "main" from src/main.c, links mymath

Build produces:
  out/cc/obj/mymath/math.o
  out/cc/obj/mymath/utils.o
  out/cc/lib/libmymath.a
  out/cc/obj/main/main.o
  out/cc/bin/main
```

## cc.yaml Format

All paths in the manifest are relative to the `cc.yaml` file's location.

```yaml
# Global settings (all optional)
cc: gcc               # C compiler (default: gcc)
cxx: g++              # C++ compiler (default: g++)
cflags: [-Wall]       # Global C flags
cxxflags: [-Wall]     # Global C++ flags
ldflags: []           # Global linker flags
include_dirs: [include]  # Global -I paths
output_dir: out/cc    # Build output directory (default: out/cc)

# Library definitions
libraries:
  - name: mymath
    lib_type: shared   # shared (.so) | static (.a) | both
    sources: [src/math.c, src/utils.c]
    include_dirs: [include]  # Additional -I for this library
    cflags: []               # Additional C flags
    cxxflags: []             # Additional C++ flags
    ldflags: [-lm]           # Linker flags for shared lib

  - name: myhelper
    lib_type: static
    sources: [src/helper.c]

# Program definitions
programs:
  - name: main
    sources: [src/main.c]
    link: [mymath, myhelper]  # Libraries defined above to link against
    ldflags: [-lpthread]      # Additional linker flags

  - name: tool
    sources: [src/tool.cc]    # .cc -> uses C++ compiler
    link: [mymath]
```

## Library Types

| Type | Output | Description |
|------|--------|-------------|
| `shared` | `lib/lib<name>.so` | Shared library (default). Sources compiled with `-fPIC`. |
| `static` | `lib/lib<name>.a` | Static library via `ar rcs`. |
| `both` | Both `.so` and `.a` | Builds both shared and static variants. |

## Language Detection

The compiler is chosen per source file based on extension:

| Extensions | Compiler |
|-----------|----------|
| `.c` | C compiler (`cc` field) |
| `.cc`, `.cpp`, `.cxx`, `.C` | C++ compiler (`cxx` field) |

Global `cflags` are used for C files and `cxxflags` for C++ files.

## Output Layout

```
<output_dir>/
  obj/<target_name>/    # Object files per target
    file.o
  lib/                  # Libraries
    lib<name>.a
    lib<name>.so
  bin/                  # Executables
    <program_name>
```

## Build Modes

### Compile + Link (default)

Each source is compiled to a `.o` file, then targets are linked from objects.
This provides incremental rebuilds — only changed sources are recompiled.

### Single Invocation

When `single_invocation = true` in `rsb.toml`, programs are built by passing
all sources directly to the compiler in one command. Libraries still use
compile+link since `ar` requires object files.

## Configuration

```toml
[processor.cc]
enabled = true            # Enable/disable (default: true)
cc = "gcc"                # Default C compiler (default: "gcc")
cxx = "g++"               # Default C++ compiler (default: "g++")
cflags = []               # Additional global C flags
cxxflags = []             # Additional global C++ flags
ldflags = []              # Additional global linker flags
include_dirs = []         # Additional global -I paths
output_dir = "out/cc"     # Output directory (default: "out/cc")
single_invocation = false # Use single-invocation mode (default: false)
extra_inputs = []         # Extra files that trigger rebuilds
cache_output_dir = true   # Cache entire output directory (default: true)
```

Note: The `cc.yaml` manifest settings override the `rsb.toml` defaults for
compiler, flags, and output directory.

### Configuration Reference

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `true` | Enable/disable the processor |
| `cc` | string | `"gcc"` | Default C compiler |
| `cxx` | string | `"g++"` | Default C++ compiler |
| `cflags` | string[] | `[]` | Global C compiler flags |
| `cxxflags` | string[] | `[]` | Global C++ compiler flags |
| `ldflags` | string[] | `[]` | Global linker flags |
| `include_dirs` | string[] | `[]` | Global include directories |
| `output_dir` | string | `"out/cc"` | Build output directory |
| `single_invocation` | bool | `false` | Build programs in single compiler invocation |
| `extra_inputs` | string[] | `[]` | Extra files that trigger rebuilds when changed |
| `cache_output_dir` | bool | `true` | Cache the entire output directory |
| `scan_dir` | string | `""` | Directory to scan for cc.yaml files |
| `extensions` | string[] | `["cc.yaml"]` | File patterns to scan for |

## Example

Given this project layout:

```
myproject/
  cc.yaml
  include/
    math.h
  src/
    math.c
    main.c
  rsb.toml
```

With `cc.yaml`:

```yaml
include_dirs: [include]

libraries:
  - name: math
    lib_type: static
    sources: [src/math.c]

programs:
  - name: main
    sources: [src/main.c]
    link: [math]
```

Running `rsb build` produces:

```
out/cc/obj/math/math.o
out/cc/lib/libmath.a
out/cc/obj/main/main.o
out/cc/bin/main
```

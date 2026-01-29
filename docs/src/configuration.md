# Configuration

RSB is configured via an `rsb.toml` file in the project root.

## Full reference

```toml
[build]
parallel = 1  # Number of parallel jobs (1 = sequential, 0 = auto-detect CPU cores)

[processor]
enabled = ["template", "pylint", "cc", "cpplint"]

[cache]
restore_method = "hardlink"  # or "copy" (hardlink is faster, copy works across filesystems)

[processor.cc]
cc = "gcc"              # C compiler (default: gcc)
cxx = "g++"             # C++ compiler (default: g++)
cflags = ["-Wall"]      # C compiler flags
cxxflags = ["-Wall"]    # C++ compiler flags
ldflags = []            # Linker flags
include_paths = ["src/include"]  # Additional -I paths (passed as-is)
source_dir = "src"      # Source directory (default: src)
output_suffix = ".elf"  # Suffix for output executables (default: .elf)

[processor.template]
strict = true           # Fail on undefined variables (default: true)
extensions = [".tera"]  # File extensions to process
trim_blocks = false     # Remove newline after block tags

[processor.pylint]
linter = "ruff"         # Python linter to use
args = []               # Extra arguments passed to the linter

[processor.cpplint]
checker = "cppcheck"  # C/C++ static checker (default: cppcheck)
args = ["--error-exitcode=1", "--enable=warning,style,performance,portability"]
# To use a suppressions file: add "--suppressions-list=.cppcheck-suppressions" to args

[graph]
viewer = "google-chrome"  # Command to open graph files (default: platform-specific)

[completions]
shells = ["bash"]
```

## Section details

### `[build]`

| Key | Type | Default | Description |
|---|---|---|---|
| `parallel` | integer | `1` | Number of parallel jobs. `1` = sequential, `0` = auto-detect CPU cores. |

### `[processor]`

| Key | Type | Default | Description |
|---|---|---|---|
| `enabled` | array of strings | all | List of processors to enable. Available: `template`, `pylint`, `cc`, `cpplint`, `sleep`. |

### `[cache]`

| Key | Type | Default | Description |
|---|---|---|---|
| `restore_method` | string | `"hardlink"` | How to restore cached outputs. `"hardlink"` is faster; `"copy"` works across filesystems. |

### `[processor.cc]`

See [C/C++ Processor Details](cc-details.md) for full documentation.

| Key | Type | Default | Description |
|---|---|---|---|
| `cc` | string | `"gcc"` | C compiler |
| `cxx` | string | `"g++"` | C++ compiler |
| `cflags` | array | `[]` | C compiler flags |
| `cxxflags` | array | `[]` | C++ compiler flags |
| `ldflags` | array | `[]` | Linker flags |
| `include_paths` | array | `[]` | Additional `-I` paths (passed as-is) |
| `source_dir` | string | `"src"` | Source directory |
| `output_suffix` | string | `".elf"` | Suffix for output executables |

### `[processor.template]`

| Key | Type | Default | Description |
|---|---|---|---|
| `strict` | bool | `true` | Fail on undefined variables |
| `extensions` | array | `[".tera"]` | File extensions to process |
| `trim_blocks` | bool | `false` | Remove newline after block tags |

### `[processor.pylint]`

| Key | Type | Default | Description |
|---|---|---|---|
| `linter` | string | `"ruff"` | Python linter command |
| `args` | array | `[]` | Extra arguments |

### `[processor.cpplint]`

| Key | Type | Default | Description |
|---|---|---|---|
| `checker` | string | `"cppcheck"` | C/C++ static analysis tool |
| `args` | array | see above | Arguments passed to the checker |

### `[graph]`

| Key | Type | Default | Description |
|---|---|---|---|
| `viewer` | string | platform-specific | Command to open graph files |

### `[completions]`

| Key | Type | Default | Description |
|---|---|---|---|
| `shells` | array | `["bash"]` | Shells to generate completions for |

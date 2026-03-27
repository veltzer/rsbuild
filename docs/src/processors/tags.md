# Tags Processor

## Purpose

Extracts YAML frontmatter tags from markdown files into a searchable database.

## How It Works

Scans `.md` files for YAML frontmatter blocks (delimited by `---`), parses tag
metadata, and builds a [redb](https://github.com/cberner/redb) database. The
database enables querying files by tags via `rsconstruct tags` subcommands.

Optionally validates tags against a `.tags` allowlist file.

### Tag Indexing

Two kinds of frontmatter fields are indexed:

- **List fields** — each item becomes a bare tag.
  ```yaml
  tags:
    - docker
    - python
  ```
  Produces tags: `docker`, `python`.

- **Scalar fields** — indexed as `key:value` (colon separator).
  ```yaml
  level: beginner
  difficulty: 3
  published: true
  url: https://example.com/path
  ```
  Produces tags: `level:beginner`, `difficulty:3`, `published:true`,
  `url:https://example.com/path`.

Both inline YAML lists (`tags: [a, b, c]`) and multi-line lists are supported.

### The `.tags` Allowlist

When a `.tags` file exists in the project root, the build validates every
indexed tag against it. Unknown tags cause a build error with typo suggestions
(Levenshtein distance). Wildcard patterns are supported:

```
# .tags
docker
python
level:beginner
level:advanced
difficulty:*
```

The pattern `difficulty:*` matches any tag starting with `difficulty:`.

## Source Files

- Input: `**/*.md`
- Output: `out/tags/tags.db`

## Configuration

```toml
[processor.tags]
output = "out/tags/tags.db"            # Output database path
tags_file = ".tags"                    # Allowlist file for tag validation
tags_file_strict = false               # When true, missing .tags file is an error
extra_inputs = []                      # Additional files that trigger rebuilds when changed
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `output` | string | `"out/tags/tags.db"` | Path to the tags database file |
| `tags_file` | string | `".tags"` | Path to the tag allowlist file |
| `tags_file_strict` | bool | `false` | Fail if the `.tags` file is missing |
| `extra_inputs` | string[] | `[]` | Extra files whose changes trigger rebuilds |

## Subcommands

All subcommands require a prior `rsconstruct build` to populate the database.
All support `--json` for machine-readable output.

### Querying

| Command | Description |
|---------|-------------|
| `rsconstruct tags list` | List all unique tags (sorted) |
| `rsconstruct tags files TAG [TAG...]` | List files matching all given tags (AND) |
| `rsconstruct tags files --or TAG [TAG...]` | List files matching any given tag (OR) |
| `rsconstruct tags grep TEXT` | Search for tags containing a substring |
| `rsconstruct tags grep -i TEXT` | Case-insensitive tag search |
| `rsconstruct tags for-file PATH` | List all tags for a specific file (supports suffix matching) |
| `rsconstruct tags frontmatter PATH` | Show raw parsed frontmatter for a file |
| `rsconstruct tags count` | Show each tag with its file count, sorted by frequency |
| `rsconstruct tags tree` | Show tags grouped by key (e.g. `level=` group) vs bare tags |
| `rsconstruct tags stats` | Show database statistics (file count, unique tags, associations) |

### `.tags` File Management

| Command | Description |
|---------|-------------|
| `rsconstruct tags init` | Generate a `.tags` file from all currently indexed tags |
| `rsconstruct tags sync` | Add missing tags to `.tags` (preserves existing entries) |
| `rsconstruct tags sync --prune` | Sync and remove unused tags from `.tags` |
| `rsconstruct tags add TAG` | Add a single tag to `.tags` |
| `rsconstruct tags remove TAG` | Remove a single tag from `.tags` |
| `rsconstruct tags unused` | List tags in `.tags` that no file uses |
| `rsconstruct tags unused --strict` | Same, but exit with error if any unused tags exist (for CI) |
| `rsconstruct tags validate` | Validate indexed tags against `.tags` without rebuilding |

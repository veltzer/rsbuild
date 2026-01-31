# Suggestions

Ideas for future improvements, organized by category.

## Error Handling

### ~~Mutex unwraps in executor.rs~~ *(Done)*
- Switched to `parking_lot::Mutex` which doesn't poison. Eliminated 48 `.lock().unwrap()` calls and 5 `.into_inner().map_err(...)` chains.

## Missing Test Coverage

### ~~Limited parallel execution tests~~ *(Done)*
- Five tests now exercise parallel builds: `-j` flag, keep-going in parallel, all-products parallel, parallel timings, and parallel caching after failure.

### No ruff/pylint processor tests
- `tests/processors/` has tests for cc, sleep, spellcheck, and template, but not for ruff or pylint.
- Add integration tests for both Python linting processors.

### No make processor tests
- `tests/processors/` has no tests for the make processor.
- Add integration tests covering Makefile discovery and execution.

## Security

### Shell command execution from source file comments
- `src/processors/cc.rs` — `EXTRA_*_SHELL` directives execute arbitrary shell commands parsed from source file comments.
- Document the security implications clearly.

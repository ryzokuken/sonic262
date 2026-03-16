# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is sonic262?

sonic262 is a Rust-based test runner for [Test262](https://github.com/tc39/test262), the official ECMAScript conformance test suite. It parses Test262 test files (extracting YAML frontmatter for metadata like `includes`, `flags`, and `features`), assembles the required harness files, and executes each test in a Node.js `vm` sandbox via a temporary file.

## Build and Run

```sh
cargo build                # compile
cargo run -- --root-path /path/to/test262   # run against a test262 checkout
cargo run -- --test-path ./some/test.js --include-path ./harness  # run a single test
cargo test                 # unit tests
cargo bench                # criterion benchmarks (requires Node.js)
cargo fmt --all -- --check # format check
cargo clippy -- -D warnings # lint
```

**Runtime dependency:** Node.js must be installed — tests are executed by spawning `node` on a temp file that uses `vm.runInContext`.

## Architecture

The codebase is small (~160 lines of Rust + a JS container):

- **`src/main.rs`** — CLI entry point using `clap`. Accepts `--root-path`, `--test-path`, and `--include-path`. If only `--root-path` is given, it derives test and harness paths from the Test262 directory layout (`<root>/test` and `<root>/harness`).
- **`src/lib.rs`** — Core logic, all in one module:
  - `extract_frontmatter` — parses `/*--- ... ---*/` YAML blocks from test files
  - `generate_includes` — reads and concatenates harness files (`assert.js`, `sta.js`, plus any declared in frontmatter)
  - `run_code` — JSON-escapes assembled code, injects it into `container.js`, writes to a tempfile, and runs `node`
  - `process_file` — orchestrates: read test → parse frontmatter → assemble includes → execute
  - `run_test` — public entry point; handles single file or recursive directory walk via `walkdir`
- **`src/container.js`** — Node.js wrapper that creates a `vm` context with `print`, `console`, `setTimeout`, `require`, then evaluates the interpolated test code (`${code}` placeholder)
- **`benches/`** — Criterion benchmarks with fixture test files and a copy of the Test262 harness

## Test262 Concepts

Each Test262 file has YAML frontmatter between `/*---` and `---*/` markers. Key fields:
- `includes` — additional harness files to prepend (e.g., `compareArray.js`, `propertyHelper.js`)
- `flags` — execution modifiers (e.g., `onlyStrict`, `async`) — parsed but not yet used
- `features` — required JS features — parsed but not yet used

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is sonic262?

sonic262 is a fast, parallel Rust-based test runner for [Test262](https://github.com/tc39/test262), the official ECMAScript conformance test suite. It replaces [test262-harness](https://github.com/nicolo-ribaudo/tc39-test262-harness) with true OS-level parallelism via Rayon, harness caching, and correct handling of all Test262 execution modes (strict, non-strict, module, raw, async, negative).

## Build and Run

```sh
cargo build                                    # compile
cargo run -- /path/to/test262/test/            # run against a test262 checkout
cargo run -- tests/fixtures/pass.js            # run a single test
cargo run -- /path/to/tests/ --threads 8       # limit parallelism
cargo test                                     # all tests (unit + integration)
cargo fmt --all -- --check                     # format check
cargo clippy --all-targets -- -D warnings      # lint
```

**Runtime dependency:** Node.js must be installed — tests are executed by spawning `node` on a temp file that uses `vm.createContext`.

## CLI Interface

```
sonic262 [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...          Test files or directories (globs supported)

Options:
      --test262-dir <DIR>       Root test262 directory (auto-detected)
      --includes-dir <DIR>      Harness includes directory (auto-detected)
      --host-path <PATH>        Path to host executable [default: node]
      --host-args <ARGS>        Comma-separated args to pass to the host
  -t, --threads <N>             Max parallel threads [default: all cores]
      --timeout <MS>            Per-test timeout in milliseconds [default: 10000]
  -r, --reporter <FORMAT>       Output format: simple, json [default: simple]
      --reporter-keys <KEYS>    Comma-separated keys for JSON reporter
      --features-include <F>    Only run tests with these features
      --features-exclude <F>    Skip tests with these features
      --error-for-failures      Exit with non-zero code if any test fails
```

## Architecture

Pipeline: Discover -> Parse -> Assemble -> Run -> Validate -> Report

Six modules connected by typed data structures:

- **`src/types.rs`** — Pipeline types: `TestMetadata`, `TestCase`, `Scenario`, `TestRun`, `RawResult`, `TestResult`, `RunSummary`
- **`src/parser.rs`** — YAML frontmatter extraction, file discovery (recursive walk, glob, `_FIXTURE` skipping), feature filtering, harness auto-detection
- **`src/assembler.rs`** — Scenario expansion (strict/non-strict/module/raw), harness caching via `HarnessCache`, code assembly with container scripts
- **`src/runner.rs`** — Process spawning with `wait-timeout` for per-test timeouts, stdout/stderr capture
- **`src/validator.rs`** — Pass/fail determination for normal, async, and negative tests
- **`src/reporter.rs`** — `SimpleReporter` (TTY-aware, colored) and `JsonReporter` (structured output)
- **`src/main.rs`** — CLI (clap v4 derive), Rayon thread pool, crossbeam channel for result streaming
- **`src/container_script.js`** — Node.js `vm.createContext` sandbox for script tests
- **`src/container_module.js`** — Node.js `vm.SourceTextModule` sandbox for module tests

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` v4 | CLI argument parsing (derive) |
| `walkdir` | Recursive file discovery |
| `yaml-rust2` | YAML frontmatter parsing |
| `tempfile` | Temp files for assembled test code |
| `colored` | Terminal colors |
| `json` | JSON output for reporter |
| `rayon` | Work-stealing parallel thread pool |
| `crossbeam-channel` | Thread-safe result channel |
| `wait-timeout` | Process timeout with kill |
| `glob` | Glob pattern matching for test paths |

## Test262 Concepts

Each Test262 file has YAML frontmatter between `/*---` and `---*/` markers. Key fields:
- `includes` — additional harness files to prepend (e.g., `compareArray.js`, `propertyHelper.js`)
- `flags` — execution modifiers: `onlyStrict`, `noStrict`, `module`, `raw`, `async`, `generated`, `CanBlockIsFalse`, `CanBlockIsTrue`, `non-deterministic`
- `features` — required JS features (used for filtering)
- `negative` — expected error: `{ phase: parse|resolution|runtime, type: ErrorName }`

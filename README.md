# sonic262

A fast, parallel [Test262](https://github.com/tc39/test262) runner built in Rust. **3.5x faster** than [test262-harness](https://github.com/nicolo-ribaudo/tc39-test262-harness) on multi-core machines.

sonic262 runs the official ECMAScript conformance test suite with true OS-level parallelism via [Rayon](https://github.com/rayon-rs/rayon), harness caching, and correct handling of all Test262 execution modes: strict, non-strict, module, raw, async, and negative tests.

## Benchmarks

Measured against test262-harness on `test/language/expressions/` (~11K test files, ~21K scenarios) using Node.js v22 on a 22-core machine:

| Configuration | Threads | sonic262 | test262-harness | Speedup |
|---|---|---|---|---|
| `expressions/object` (~1,170 tests) | 1 | 30.9s | 46.0s | 1.49x |
| `expressions/` (~11K tests) | 22 | 35.1s | 125.4s | 3.57x |

Single-threaded, sonic262 is ~1.5x faster due to lower per-test overhead (direct `node` spawning vs. the `eshost` abstraction layer). Multi-threaded, Rayon's work-stealing thread pool scales efficiently across all cores, widening the gap to 3.5x.

Results will vary by hardware. Run the benchmarks yourself — see [Contributing](#contributing).

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v18+)

### Install

```sh
cargo install --path .
```

### Run

```sh
# Clone test262 (shallow for speed)
git clone --depth 1 https://github.com/nicolo-ribaudo/test262.git

# Run the full suite
sonic262 test262/test/

# Run a subset
sonic262 test262/test/language/expressions/

# Run a single file
sonic262 test262/test/language/expressions/addition/S11.6.1_A1.js

# Limit parallelism
sonic262 --threads 4 test262/test/

# JSON output
sonic262 -r json test262/test/built-ins/Array/
```

sonic262 auto-detects the test262 root and harness directory by walking up from the test path. Override with `--test262-dir` and `--includes-dir` if needed.

## Usage

```
sonic262 [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Test files or directories (globs supported)

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

### Filtering by Feature

Test262 tests declare required JavaScript features in their frontmatter. Use `--features-include` and `--features-exclude` to target specific areas:

```sh
# Only run tests that use optional chaining
sonic262 --features-include optional-chaining test262/test/

# Skip tests requiring Intl (useful for engines without ICU)
sonic262 --features-exclude Intl test262/test/
```

### JSON Output

The JSON reporter outputs one object per test result. Use `--reporter-keys` to select specific fields:

```sh
sonic262 -r json --reporter-keys file,scenario,pass test262/test/built-ins/Array/
```

### Exit Codes

By default, sonic262 exits 0 regardless of test results. Use `--error-for-failures` to exit with a non-zero code when any test fails — useful in CI pipelines.

## How It Works

sonic262 processes tests through a six-stage pipeline:

```
Discover -> Parse -> Assemble -> Run -> Validate -> Report
```

1. **Discover** — Recursively walks directories, expands globs, skips `_FIXTURE` files, applies feature filters
2. **Parse** — Extracts YAML frontmatter (`/*--- ... ---*/`) from each test file for metadata: flags, includes, features, negative expectations
3. **Assemble** — Expands each test into scenarios (strict/non-strict/module/raw), prepends harness files from a shared read-through cache, wraps code in a Node.js `vm.createContext` or `vm.SourceTextModule` container
4. **Run** — Spawns `node` processes in parallel via Rayon, with per-test timeouts and kill-on-expiry
5. **Validate** — Determines pass/fail by checking exit codes, stdout for async completion markers, and stderr for expected error types and phases
6. **Report** — Streams results as they complete: colored TTY output (simple) or structured JSON

Each stage communicates through typed Rust data structures (`TestCase` -> `TestRun` -> `RawResult` -> `TestResult`), making the pipeline easy to test and extend.

## Project History

sonic262 started as a proof of concept developed by [Ujjwal Sharma](https://github.com/ryzokuken) with the help of Claude (Opus). The v1 rewrite — the current codebase — was built on top of that foundation, expanding it into a fully modular, parallel runner with complete Test262 execution mode support.

## Contributing

```sh
cargo build                                # compile
cargo test                                 # all tests (unit + integration)
cargo fmt --all -- --check                 # format check
cargo clippy --all-targets -- -D warnings  # lint
```

To reproduce the benchmarks:

```sh
# Clone test262
git clone --depth 1 https://github.com/nicolo-ribaudo/test262.git /tmp/test262

# Install test262-harness
mkdir /tmp/test262-bench && cd /tmp/test262-bench
npm init -y && npm install test262-harness

# Build sonic262 in release mode
cargo build --release

# Run sonic262
time ./target/release/sonic262 \
  --test262-dir /tmp/test262 \
  --includes-dir /tmp/test262/harness \
  /tmp/test262/test/language/expressions/

# Run test262-harness
time ./node_modules/.bin/test262-harness \
  --host-type node --host-path "$(which node)" \
  --test262-dir /tmp/test262 \
  --threads "$(nproc)" \
  "/tmp/test262/test/language/expressions/**/*.js"
```

## License

[MIT](LICENSE)

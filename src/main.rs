use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Parser;
use crossbeam_channel::unbounded;
use rayon::prelude::*;

use sonic262::assembler::{assemble, HarnessCache};
use sonic262::parser::{detect_harness_dir, discover_tests};
use sonic262::reporter::{JsonReporter, Reporter, SimpleReporter};
use sonic262::runner::{run_single, RunConfig};
use sonic262::types::*;
use sonic262::validator::validate;

#[derive(Parser)]
#[command(name = "sonic262", version, about = "Fast Test262 test runner")]
struct Cli {
    /// Test files or directories
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Root test262 directory (auto-detected if omitted)
    #[arg(long)]
    test262_dir: Option<PathBuf>,

    /// Harness includes directory (auto-detected if omitted)
    #[arg(long)]
    includes_dir: Option<PathBuf>,

    /// Path to host executable
    #[arg(long, default_value = "node")]
    host_path: String,

    /// Additional args to pass to the host (comma-separated)
    #[arg(long)]
    host_args: Option<String>,

    /// Max parallel threads (default: all cores)
    #[arg(short, long)]
    threads: Option<usize>,

    /// Per-test timeout in milliseconds
    #[arg(long, default_value = "10000")]
    timeout: u64,

    /// Output format: simple, json
    #[arg(short, long, default_value = "simple")]
    reporter: String,

    /// Comma-separated keys for JSON reporter
    #[arg(long)]
    reporter_keys: Option<String>,

    /// Only run tests with these features (comma-separated)
    #[arg(long)]
    features_include: Option<String>,

    /// Skip tests with these features (comma-separated)
    #[arg(long)]
    features_exclude: Option<String>,

    /// Exit with non-zero code if any test fails
    #[arg(long)]
    error_for_failures: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Some(threads) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
    }

    let features_include: Option<Vec<String>> = cli
        .features_include
        .as_ref()
        .map(|s| s.split(',').map(|f| f.trim().to_string()).collect());
    let features_exclude: Option<Vec<String>> = cli
        .features_exclude
        .as_ref()
        .map(|s| s.split(',').map(|f| f.trim().to_string()).collect());

    let tests = match discover_tests(
        &cli.paths,
        features_include.as_deref(),
        features_exclude.as_deref(),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error discovering tests: {e}");
            std::process::exit(1);
        }
    };

    if tests.is_empty() {
        eprintln!("No tests found.");
        std::process::exit(1);
    }

    let harness_dir = cli
        .includes_dir
        .or_else(|| cli.test262_dir.as_ref().map(|d| d.join("harness")))
        .or_else(|| detect_harness_dir(&tests[0].path))
        .unwrap_or_else(|| {
            eprintln!("Could not detect harness directory. Use --includes-dir.");
            std::process::exit(1);
        });

    let cache = HarnessCache::with_harness_dir(harness_dir);

    let mut all_runs: Vec<TestRun> = Vec::new();
    for test in tests {
        let test = Arc::new(test);
        match assemble(&test, &cache) {
            Ok(runs) => all_runs.extend(runs),
            Err(e) => {
                eprintln!("Error assembling {}: {e}", test.relative_path);
            }
        }
    }

    let run_config = RunConfig {
        host_path: cli.host_path.clone(),
        host_args: cli
            .host_args
            .as_ref()
            .map(|s| s.split(',').map(|a| a.trim().to_string()).collect())
            .unwrap_or_default(),
        timeout_ms: cli.timeout,
    };

    let stdout = std::io::stdout();
    let is_tty = stdout.is_terminal();
    let mut reporter: Box<dyn Reporter> = match cli.reporter.as_str() {
        "json" => {
            let keys = cli
                .reporter_keys
                .map(|s| s.split(',').map(|k| k.trim().to_string()).collect());
            Box::new(JsonReporter::new(stdout.lock(), keys))
        }
        _ => Box::new(SimpleReporter::new(stdout.lock(), is_tty)),
    };

    let (tx, rx) = unbounded();
    let start = Instant::now();

    {
        let tx = tx;
        all_runs.into_par_iter().for_each_with(tx, |tx, run| {
            let result = match run_single(&run.assembled_code, &run_config) {
                Ok(raw) => validate(&run.test, run.scenario, &raw),
                Err(e) => TestResult {
                    test: Arc::clone(&run.test),
                    scenario: run.scenario,
                    pass: false,
                    message: Some(format!("Runner error: {e}")),
                },
            };
            tx.send(result).ok();
        });
    }

    let mut summary = RunSummary {
        total: 0,
        passed: 0,
        failed: 0,
        duration: Duration::ZERO,
    };

    for result in rx {
        summary.total += 1;
        if result.pass {
            summary.passed += 1;
        } else {
            summary.failed += 1;
        }
        reporter.test_result(&result);
    }

    summary.duration = start.elapsed();
    reporter.finish(&summary);

    if cli.error_for_failures && summary.failed > 0 {
        std::process::exit(1);
    }
}

use std::path::PathBuf;
use std::sync::Arc;

use sonic262::assembler::{assemble, HarnessCache};
use sonic262::types::*;

fn harness() -> HarnessCache {
    HarnessCache::with_harness_dir(PathBuf::from("benches/fixtures/harness"))
}

fn make_test(flags: &[TestFlag]) -> Arc<TestCase> {
    Arc::new(TestCase {
        path: PathBuf::from("test.js"),
        relative_path: "test.js".to_string(),
        contents: "assert(true);".to_string(),
        metadata: TestMetadata {
            flags: flags.iter().cloned().collect(),
            ..TestMetadata::default()
        },
    })
}

#[test]
fn no_flags_produces_two_scenarios() {
    let cache = harness();
    let test = make_test(&[]);
    let runs = assemble(&test, &cache).unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].scenario, Scenario::Default);
    assert_eq!(runs[1].scenario, Scenario::Strict);
}

#[test]
fn only_strict_produces_one_scenario() {
    let cache = harness();
    let test = make_test(&[TestFlag::OnlyStrict]);
    let runs = assemble(&test, &cache).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].scenario, Scenario::Strict);
}

#[test]
fn no_strict_produces_one_scenario() {
    let cache = harness();
    let test = make_test(&[TestFlag::NoStrict]);
    let runs = assemble(&test, &cache).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].scenario, Scenario::Default);
}

#[test]
fn module_produces_module_scenario() {
    let cache = harness();
    let test = make_test(&[TestFlag::Module]);
    let runs = assemble(&test, &cache).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].scenario, Scenario::Module);
}

#[test]
fn raw_produces_unmodified_code() {
    let cache = harness();
    let test = make_test(&[TestFlag::Raw]);
    let runs = assemble(&test, &cache).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].scenario, Scenario::Default);
    assert!(!runs[0].assembled_code.contains("assert._isSameValue"));
}

#[test]
fn strict_scenario_prepends_use_strict() {
    let cache = harness();
    let test = make_test(&[TestFlag::OnlyStrict]);
    let runs = assemble(&test, &cache).unwrap();
    // The "use strict" is inside the JSON-stringified code within the container
    assert!(runs[0].assembled_code.contains("use strict"));
}

#[test]
fn async_flag_includes_done_print_handle() {
    let cache = harness();
    let test = make_test(&[TestFlag::Async]);
    let runs = assemble(&test, &cache).unwrap();
    assert!(runs[0].assembled_code.contains("$DONE"));
}

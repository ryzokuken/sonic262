use std::path::PathBuf;

use sonic262::parser::{detect_harness_dir, discover_tests};

#[test]
fn discovers_single_js_file() {
    let tests = discover_tests(&[PathBuf::from("benches/fixtures/single.js")], None, None).unwrap();
    assert_eq!(tests.len(), 1);
    assert!(tests[0].relative_path.ends_with("single.js"));
}

#[test]
fn discovers_directory_recursively() {
    let tests = discover_tests(&[PathBuf::from("benches/fixtures/multiple")], None, None).unwrap();
    assert!(tests.len() >= 5);
}

#[test]
fn skips_fixture_files() {
    let tests = discover_tests(&[PathBuf::from("benches/fixtures/multiple")], None, None).unwrap();
    for test in &tests {
        assert!(
            !test.relative_path.contains("_FIXTURE"),
            "Should skip _FIXTURE files, found: {}",
            test.relative_path
        );
    }
}

#[test]
fn filters_by_feature_include() {
    let tests = discover_tests(
        &[PathBuf::from("benches/fixtures/multiple")],
        Some(&["BigInt".to_string()]),
        None,
    )
    .unwrap();
    assert!(tests.is_empty());
}

#[test]
fn feature_exclude_filters_out_matching() {
    let all = discover_tests(&[PathBuf::from("benches/fixtures/multiple")], None, None).unwrap();
    let filtered = discover_tests(
        &[PathBuf::from("benches/fixtures/multiple")],
        None,
        Some(&["BigInt".to_string()]),
    )
    .unwrap();
    assert_eq!(all.len(), filtered.len());
}

#[test]
fn detects_harness_from_test_path() {
    let harness = detect_harness_dir(&PathBuf::from("benches/fixtures/multiple/15.1.1.2-0.js"));
    assert!(harness.is_some());
    let harness = harness.unwrap();
    assert!(harness.join("assert.js").exists());
}

#[test]
fn returns_none_when_no_harness_found() {
    let harness = detect_harness_dir(&PathBuf::from("/tmp/nonexistent/test.js"));
    assert!(harness.is_none());
}

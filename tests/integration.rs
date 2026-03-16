use assert_cmd::Command;

fn sonic262() -> Command {
    Command::cargo_bin("sonic262").unwrap()
}

#[test]
fn pass_test_exits_zero() {
    sonic262()
        .args([
            "--includes-dir",
            "tests/fixtures/harness",
            "tests/fixtures/pass.js",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("PASS"));
}

#[test]
fn fail_test_with_error_for_failures_exits_nonzero() {
    sonic262()
        .args([
            "--includes-dir",
            "tests/fixtures/harness",
            "--error-for-failures",
            "tests/fixtures/fail.js",
        ])
        .assert()
        .failure()
        .stdout(predicates::str::contains("FAIL"));
}

#[test]
fn negative_parse_test_passes() {
    sonic262()
        .args([
            "--includes-dir",
            "tests/fixtures/harness",
            "tests/fixtures/negative_parse.js",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("PASS"));
}

#[test]
fn negative_runtime_test_passes() {
    sonic262()
        .args([
            "--includes-dir",
            "tests/fixtures/harness",
            "tests/fixtures/negative_runtime.js",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("PASS"));
}

#[test]
fn json_reporter_outputs_valid_json() {
    let output = sonic262()
        .args([
            "--includes-dir",
            "tests/fixtures/harness",
            "--reporter",
            "json",
            "tests/fixtures/pass.js",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed = json::parse(&stdout);
    assert!(parsed.is_ok(), "Output should be valid JSON: {stdout}");
}

#[test]
fn raw_test_runs_without_harness() {
    sonic262()
        .args([
            "--includes-dir",
            "tests/fixtures/harness",
            "tests/fixtures/raw.js",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("PASS"));
}

#[test]
fn summary_shows_total_counts() {
    sonic262()
        .args([
            "--includes-dir",
            "tests/fixtures/harness",
            "tests/fixtures/",
        ])
        .assert()
        .stdout(predicates::str::contains("Ran"))
        .stdout(predicates::str::contains("passed"))
        .stdout(predicates::str::contains("failed"));
}

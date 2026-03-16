use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use sonic262::reporter::{JsonReporter, Reporter, SimpleReporter};
use sonic262::types::*;

fn make_pass(path: &str, scenario: Scenario) -> TestResult {
    TestResult {
        test: Arc::new(TestCase {
            path: PathBuf::from(path),
            relative_path: path.to_string(),
            contents: String::new(),
            metadata: TestMetadata::default(),
        }),
        scenario,
        pass: true,
        message: None,
    }
}

fn make_fail(path: &str, scenario: Scenario, msg: &str) -> TestResult {
    TestResult {
        test: Arc::new(TestCase {
            path: PathBuf::from(path),
            relative_path: path.to_string(),
            contents: String::new(),
            metadata: TestMetadata::default(),
        }),
        scenario,
        pass: false,
        message: Some(msg.to_string()),
    }
}

#[test]
fn simple_reporter_counts() {
    let mut buf = Vec::new();
    {
        let mut reporter = SimpleReporter::new(&mut buf, false);
        reporter.test_result(&make_pass("a.js", Scenario::Default));
        reporter.test_result(&make_fail("b.js", Scenario::Strict, "oops"));
        reporter.finish(&RunSummary {
            total: 2,
            passed: 1,
            failed: 1,
            duration: Duration::from_secs(1),
        });
    }
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("1 passed"));
    assert!(output.contains("1 failed"));
}

#[test]
fn json_reporter_produces_valid_json() {
    let mut buf = Vec::new();
    {
        let mut reporter = JsonReporter::new(&mut buf, None);
        reporter.test_result(&make_pass("a.js", Scenario::Default));
        reporter.test_result(&make_fail("b.js", Scenario::Strict, "bad"));
        reporter.finish(&RunSummary {
            total: 2,
            passed: 1,
            failed: 1,
            duration: Duration::from_secs(1),
        });
    }
    let output = String::from_utf8(buf).unwrap();
    let parsed = json::parse(&output).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0]["pass"], true);
    assert_eq!(parsed[1]["pass"], false);
}

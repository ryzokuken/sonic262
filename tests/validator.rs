use std::path::PathBuf;
use std::sync::Arc;

use sonic262::types::*;
use sonic262::validator::validate;

fn make_result(
    flags: &[TestFlag],
    negative: Option<NegativeExpectation>,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
    timed_out: bool,
) -> (Arc<TestCase>, Scenario, RawResult) {
    let test = Arc::new(TestCase {
        path: PathBuf::from("test.js"),
        relative_path: "test.js".to_string(),
        contents: String::new(),
        metadata: TestMetadata {
            flags: flags.iter().cloned().collect(),
            negative,
            ..TestMetadata::default()
        },
    });
    let raw = RawResult {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        exit_code,
        timed_out,
    };
    (test, Scenario::Default, raw)
}

#[test]
fn normal_test_pass() {
    let (test, scenario, raw) = make_result(&[], None, Some(0), "", "", false);
    let result = validate(&test, scenario, &raw);
    assert!(result.pass);
}

#[test]
fn normal_test_fail_nonzero_exit() {
    let (test, scenario, raw) = make_result(&[], None, Some(1), "", "TypeError: bad\n", false);
    let result = validate(&test, scenario, &raw);
    assert!(!result.pass);
    assert!(result.message.as_ref().unwrap().contains("TypeError"));
}

#[test]
fn normal_test_pass_with_stderr_warnings() {
    let (test, scenario, raw) =
        make_result(&[], None, Some(0), "", "DeprecationWarning: blah\n", false);
    let result = validate(&test, scenario, &raw);
    assert!(result.pass);
}

#[test]
fn timeout_is_failure() {
    let (test, scenario, raw) = make_result(&[], None, None, "", "", true);
    let result = validate(&test, scenario, &raw);
    assert!(!result.pass);
    assert_eq!(result.message.as_deref(), Some("Test timed out"));
}

#[test]
fn async_test_pass() {
    let (test, scenario, raw) = make_result(
        &[TestFlag::Async],
        None,
        Some(0),
        "Test262:AsyncTestComplete\n",
        "",
        false,
    );
    let result = validate(&test, scenario, &raw);
    assert!(result.pass);
}

#[test]
fn async_test_failure() {
    let (test, scenario, raw) = make_result(
        &[TestFlag::Async],
        None,
        Some(0),
        "Test262:AsyncTestFailure:TypeError: bad value\n",
        "",
        false,
    );
    let result = validate(&test, scenario, &raw);
    assert!(!result.pass);
    assert!(result.message.as_ref().unwrap().contains("bad value"));
}

#[test]
fn async_test_no_completion() {
    let (test, scenario, raw) = make_result(&[TestFlag::Async], None, Some(0), "", "", false);
    let result = validate(&test, scenario, &raw);
    assert!(!result.pass);
    assert!(result
        .message
        .as_ref()
        .unwrap()
        .contains("did not run to completion"));
}

#[test]
fn negative_runtime_pass() {
    let neg = NegativeExpectation {
        phase: NegativePhase::Runtime,
        error_type: "TypeError".to_string(),
    };
    let (test, scenario, raw) = make_result(
        &[],
        Some(neg),
        Some(1),
        "",
        "TypeError: cannot read property\n",
        false,
    );
    let result = validate(&test, scenario, &raw);
    assert!(result.pass);
}

#[test]
fn negative_runtime_wrong_error() {
    let neg = NegativeExpectation {
        phase: NegativePhase::Runtime,
        error_type: "TypeError".to_string(),
    };
    let (test, scenario, raw) =
        make_result(&[], Some(neg), Some(1), "", "RangeError: invalid\n", false);
    let result = validate(&test, scenario, &raw);
    assert!(!result.pass);
}

#[test]
fn negative_runtime_no_error() {
    let neg = NegativeExpectation {
        phase: NegativePhase::Runtime,
        error_type: "TypeError".to_string(),
    };
    let (test, scenario, raw) = make_result(&[], Some(neg), Some(0), "", "", false);
    let result = validate(&test, scenario, &raw);
    assert!(!result.pass);
}

#[test]
fn negative_parse_pass() {
    let neg = NegativeExpectation {
        phase: NegativePhase::Parse,
        error_type: "SyntaxError".to_string(),
    };
    let (test, scenario, raw) = make_result(
        &[],
        Some(neg),
        Some(1),
        "",
        "SyntaxError: Unexpected token\n",
        false,
    );
    let result = validate(&test, scenario, &raw);
    assert!(result.pass);
}

#[test]
fn negative_parse_fail_donotevaluate_called() {
    let neg = NegativeExpectation {
        phase: NegativePhase::Parse,
        error_type: "SyntaxError".to_string(),
    };
    let (test, scenario, raw) = make_result(
        &[],
        Some(neg),
        Some(1),
        "",
        "Test262Error: $DONOTEVALUATE\n",
        false,
    );
    let result = validate(&test, scenario, &raw);
    assert!(!result.pass);
}

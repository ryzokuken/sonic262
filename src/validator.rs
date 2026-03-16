use std::sync::Arc;

use crate::types::*;

pub fn validate(test: &Arc<TestCase>, scenario: Scenario, raw: &RawResult) -> TestResult {
    if raw.timed_out {
        return TestResult {
            test: Arc::clone(test),
            scenario,
            pass: false,
            message: Some("Test timed out".to_string()),
        };
    }

    let is_async = test.metadata.flags.contains(&TestFlag::Async);

    match &test.metadata.negative {
        Some(negative) => validate_negative(test, scenario, raw, negative),
        None if is_async => validate_async(test, scenario, raw),
        None => validate_normal(test, scenario, raw),
    }
}

fn validate_normal(test: &Arc<TestCase>, scenario: Scenario, raw: &RawResult) -> TestResult {
    if raw.exit_code == Some(0) {
        TestResult {
            test: Arc::clone(test),
            scenario,
            pass: true,
            message: None,
        }
    } else {
        let message =
            extract_error_message(&raw.stderr).unwrap_or_else(|| "Non-zero exit code".to_string());
        TestResult {
            test: Arc::clone(test),
            scenario,
            pass: false,
            message: Some(message),
        }
    }
}

fn validate_async(test: &Arc<TestCase>, scenario: Scenario, raw: &RawResult) -> TestResult {
    for line in raw.stdout.lines() {
        if line.contains("Test262:AsyncTestComplete") {
            return TestResult {
                test: Arc::clone(test),
                scenario,
                pass: true,
                message: None,
            };
        }
        if let Some(rest) = line.strip_prefix("Test262:AsyncTestFailure:") {
            return TestResult {
                test: Arc::clone(test),
                scenario,
                pass: false,
                message: Some(rest.to_string()),
            };
        }
    }

    TestResult {
        test: Arc::clone(test),
        scenario,
        pass: false,
        message: Some("Test did not run to completion".to_string()),
    }
}

fn validate_negative(
    test: &Arc<TestCase>,
    scenario: Scenario,
    raw: &RawResult,
    negative: &NegativeExpectation,
) -> TestResult {
    if raw.exit_code == Some(0) {
        return TestResult {
            test: Arc::clone(test),
            scenario,
            pass: false,
            message: Some(format!(
                "Expected {} error of type {}, but test completed successfully",
                phase_name(&negative.phase),
                negative.error_type
            )),
        };
    }

    if matches!(
        negative.phase,
        NegativePhase::Parse | NegativePhase::Resolution
    ) && raw.stderr.contains("$DONOTEVALUATE")
    {
        return TestResult {
            test: Arc::clone(test),
            scenario,
            pass: false,
            message: Some(format!(
                "Expected {} error but code was evaluated ($DONOTEVALUATE called)",
                phase_name(&negative.phase)
            )),
        };
    }

    let error_name = extract_error_name(&raw.stderr);
    match error_name {
        Some(name) if name == negative.error_type => TestResult {
            test: Arc::clone(test),
            scenario,
            pass: true,
            message: None,
        },
        Some(name) => TestResult {
            test: Arc::clone(test),
            scenario,
            pass: false,
            message: Some(format!(
                "Expected {} error of type {}, got {}",
                phase_name(&negative.phase),
                negative.error_type,
                name
            )),
        },
        None => TestResult {
            test: Arc::clone(test),
            scenario,
            pass: false,
            message: Some(format!(
                "Expected error of type {}, but could not extract error name from stderr",
                negative.error_type
            )),
        },
    }
}

fn extract_error_name(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(colon_pos) = trimmed.find(':') {
            let candidate = &trimmed[..colon_pos];
            if candidate
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                && candidate.ends_with("Error")
            {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

fn extract_error_message(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(colon_pos) = trimmed.find(':') {
            let candidate = &trimmed[..colon_pos];
            if candidate.ends_with("Error") {
                return Some(trimmed.to_string());
            }
        }
    }
    if !stderr.trim().is_empty() {
        Some(stderr.lines().last().unwrap_or("").trim().to_string())
    } else {
        None
    }
}

fn phase_name(phase: &NegativePhase) -> &'static str {
    match phase {
        NegativePhase::Parse => "parse",
        NegativePhase::Resolution => "resolution",
        NegativePhase::Runtime => "runtime",
    }
}

use std::io::Write;

use colored::Colorize;

use crate::types::*;

pub trait Reporter {
    fn test_result(&mut self, result: &TestResult);
    fn finish(&mut self, summary: &RunSummary);
}

pub struct SimpleReporter<W: Write> {
    writer: W,
    is_tty: bool,
    last_was_pass: bool,
}

impl<W: Write> SimpleReporter<W> {
    pub fn new(writer: W, is_tty: bool) -> Self {
        Self {
            writer,
            is_tty,
            last_was_pass: false,
        }
    }
}

impl<W: Write> Reporter for SimpleReporter<W> {
    fn test_result(&mut self, result: &TestResult) {
        if result.pass {
            if self.last_was_pass && self.is_tty {
                write!(self.writer, "\r\x1b[K").ok();
            } else if self.last_was_pass {
                writeln!(self.writer).ok();
            }
            self.last_was_pass = true;
            write!(
                self.writer,
                "{} {}",
                "PASS".green(),
                result.test.relative_path
            )
            .ok();
        } else {
            if self.last_was_pass {
                writeln!(self.writer).ok();
            }
            self.last_was_pass = false;
            writeln!(
                self.writer,
                "{} {} ({})",
                "FAIL".red(),
                result.test.relative_path,
                result.scenario
            )
            .ok();
            if let Some(msg) = &result.message {
                writeln!(self.writer, "  {msg}").ok();
            }
        }
    }

    fn finish(&mut self, summary: &RunSummary) {
        if self.last_was_pass {
            writeln!(self.writer).ok();
        }
        writeln!(self.writer).ok();
        writeln!(
            self.writer,
            "Ran {} tests | {} passed | {} failed | {:.1}s",
            summary.total,
            summary.passed,
            summary.failed,
            summary.duration.as_secs_f64()
        )
        .ok();
    }
}

pub struct JsonReporter<W: Write> {
    writer: W,
    keys: Option<Vec<String>>,
    results: Vec<json::JsonValue>,
}

impl<W: Write> JsonReporter<W> {
    pub fn new(writer: W, keys: Option<Vec<String>>) -> Self {
        Self {
            writer,
            keys,
            results: Vec::new(),
        }
    }
}

impl<W: Write> Reporter for JsonReporter<W> {
    fn test_result(&mut self, result: &TestResult) {
        let mut obj = json::object! {
            "file" => result.test.relative_path.as_str(),
            "scenario" => result.scenario.to_string(),
            "pass" => result.pass,
        };
        if let Some(msg) = &result.message {
            obj["message"] = msg.as_str().into();
        }
        if !result.test.metadata.features.is_empty() {
            obj["features"] = result.test.metadata.features.clone().into();
        }

        if let Some(keys) = &self.keys {
            let mut filtered = json::JsonValue::new_object();
            for key in keys {
                if obj.has_key(key) {
                    filtered[key.as_str()] = obj[key.as_str()].clone();
                }
            }
            self.results.push(filtered);
        } else {
            self.results.push(obj);
        }
    }

    fn finish(&mut self, _summary: &RunSummary) {
        let arr = json::JsonValue::Array(std::mem::take(&mut self.results));
        writeln!(self.writer, "{}", json::stringify_pretty(arr, 2)).ok();
    }
}

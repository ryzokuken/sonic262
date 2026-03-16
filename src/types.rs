use std::collections::HashSet;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestFlag {
    OnlyStrict,
    NoStrict,
    Module,
    Raw,
    Async,
    Generated,
    CanBlockIsFalse,
    CanBlockIsTrue,
    NonDeterministic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegativePhase {
    Parse,
    Resolution,
    Runtime,
}

#[derive(Debug, Clone)]
pub struct NegativeExpectation {
    pub phase: NegativePhase,
    pub error_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct TestMetadata {
    pub description: Option<String>,
    pub flags: HashSet<TestFlag>,
    pub features: Vec<String>,
    pub includes: Vec<String>,
    pub negative: Option<NegativeExpectation>,
    pub locale: Vec<String>,
}

#[derive(Debug)]
pub struct TestCase {
    pub path: PathBuf,
    pub relative_path: String,
    pub contents: String,
    pub metadata: TestMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    Default,
    Strict,
    Module,
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scenario::Default => write!(f, "default"),
            Scenario::Strict => write!(f, "strict"),
            Scenario::Module => write!(f, "module"),
        }
    }
}

pub struct TestRun {
    pub test: Arc<TestCase>,
    pub scenario: Scenario,
    pub assembled_code: String,
}

#[derive(Debug)]
pub struct RawResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

pub struct TestResult {
    pub test: Arc<TestCase>,
    pub scenario: Scenario,
    pub pass: bool,
    pub message: Option<String>,
}

pub struct RunSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: Duration,
}

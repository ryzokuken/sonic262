use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::parser::ParseError;
use crate::types::*;

pub struct HarnessCache {
    cache: RwLock<HashMap<String, String>>,
    harness_dir: Option<PathBuf>,
}

impl HarnessCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            harness_dir: None,
        }
    }

    pub fn with_harness_dir(dir: PathBuf) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            harness_dir: Some(dir),
        }
    }

    pub fn get(&self, name: &str) -> Result<String, ParseError> {
        {
            let cache = self.cache.read().unwrap();
            if let Some(contents) = cache.get(name) {
                return Ok(contents.clone());
            }
        }

        let dir = self.harness_dir.as_ref().ok_or_else(|| {
            ParseError(format!(
                "No harness directory configured, cannot load {name}"
            ))
        })?;
        let path = dir.join(name);
        let contents = std::fs::read_to_string(&path).map_err(|e| {
            ParseError(format!(
                "Failed to read harness file {}: {e}",
                path.display()
            ))
        })?;

        let mut cache = self.cache.write().unwrap();
        cache.insert(name.to_string(), contents.clone());
        Ok(contents)
    }
}

pub fn assemble(
    test: &Arc<TestCase>,
    cache: &HarnessCache,
) -> Result<Vec<TestRun>, ParseError> {
    let flags = &test.metadata.flags;

    if flags.contains(&TestFlag::Raw) {
        return Ok(vec![TestRun {
            test: Arc::clone(test),
            scenario: Scenario::Default,
            assembled_code: test.contents.clone(),
        }]);
    }

    if flags.contains(&TestFlag::Module) {
        let preamble = build_preamble(test, cache)?;
        let container = include_str!("container_module.js");
        let code = container
            .replace("${preamble}", &json::stringify(preamble))
            .replace("${testCode}", &json::stringify(test.contents.clone()))
            .replace(
                "${testDir}",
                &json::stringify(
                    test.path
                        .parent()
                        .unwrap_or(Path::new("."))
                        .to_string_lossy()
                        .to_string(),
                ),
            );
        return Ok(vec![TestRun {
            test: Arc::clone(test),
            scenario: Scenario::Module,
            assembled_code: code,
        }]);
    }

    let scenarios = if flags.contains(&TestFlag::OnlyStrict) {
        vec![Scenario::Strict]
    } else if flags.contains(&TestFlag::NoStrict) {
        vec![Scenario::Default]
    } else {
        vec![Scenario::Default, Scenario::Strict]
    };

    let mut runs = Vec::with_capacity(scenarios.len());
    for scenario in scenarios {
        let code = build_script_code(test, cache, scenario)?;
        runs.push(TestRun {
            test: Arc::clone(test),
            scenario,
            assembled_code: code,
        });
    }

    Ok(runs)
}

fn build_preamble(
    test: &TestCase,
    cache: &HarnessCache,
) -> Result<String, ParseError> {
    let mut preamble = String::new();

    preamble.push_str(&cache.get("assert.js")?);
    preamble.push('\n');
    preamble.push_str(&cache.get("sta.js")?);
    preamble.push('\n');

    if test.metadata.flags.contains(&TestFlag::Async) {
        preamble.push_str(&cache.get("doneprintHandle.js")?);
        preamble.push('\n');
    }

    for include in &test.metadata.includes {
        preamble.push_str(&cache.get(include)?);
        preamble.push('\n');
    }

    Ok(preamble)
}

fn build_script_code(
    test: &TestCase,
    cache: &HarnessCache,
    scenario: Scenario,
) -> Result<String, ParseError> {
    let mut code = String::new();

    if scenario == Scenario::Strict {
        code.push_str("\"use strict\";\n");
    }

    code.push_str(&cache.get("assert.js")?);
    code.push('\n');
    code.push_str(&cache.get("sta.js")?);
    code.push('\n');

    if test.metadata.flags.contains(&TestFlag::Async) {
        code.push_str(&cache.get("doneprintHandle.js")?);
        code.push('\n');
    }

    for include in &test.metadata.includes {
        code.push_str(&cache.get(include)?);
        code.push('\n');
    }

    code.push_str(&test.contents);

    let container = include_str!("container_script.js");
    let assembled = container.replace(
        "${code}",
        &json::stringify(code)
            .replace("\u{2028}", "\\u2028")
            .replace("\u{2029}", "\\u2029"),
    );

    Ok(assembled)
}

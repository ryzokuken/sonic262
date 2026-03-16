use std::collections::HashSet;
use std::path::PathBuf;

use yaml_rust2::{Yaml, YamlLoader};

use crate::types::{
    NegativeExpectation, NegativePhase, TestCase, TestFlag, TestMetadata,
};

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}

pub fn extract_frontmatter(contents: &str) -> Result<TestMetadata, ParseError> {
    let yaml_start = contents
        .find("/*---")
        .ok_or_else(|| ParseError("No frontmatter found".to_string()))?;
    let yaml_end = contents
        .find("---*/")
        .ok_or_else(|| ParseError("Unterminated frontmatter".to_string()))?;

    let raw = contents[yaml_start + 5..yaml_end]
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let raw = raw.trim_matches('\n');

    if raw.is_empty() {
        return Ok(TestMetadata::default());
    }

    let docs = YamlLoader::load_from_str(raw)
        .map_err(|e| ParseError(format!("YAML parse error: {e}")))?;

    let doc = docs
        .first()
        .ok_or_else(|| ParseError("Empty YAML document".to_string()))?;

    let hash = doc
        .as_hash()
        .ok_or_else(|| ParseError("Frontmatter is not a YAML mapping".to_string()))?;

    let description = hash
        .get(&Yaml::String("description".into()))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string());

    let flags = extract_flags(hash.get(&Yaml::String("flags".into())));
    let features = extract_string_list(hash.get(&Yaml::String("features".into())));
    let includes = extract_string_list(hash.get(&Yaml::String("includes".into())));
    let locale = extract_string_list(hash.get(&Yaml::String("locale".into())));
    let negative = extract_negative(hash.get(&Yaml::String("negative".into())))?;

    Ok(TestMetadata {
        description,
        flags,
        features,
        includes,
        negative,
        locale,
    })
}

pub fn discover_tests(
    paths: &[PathBuf],
    features_include: Option<&[String]>,
    features_exclude: Option<&[String]>,
) -> Result<Vec<TestCase>, ParseError> {
    let mut tests = Vec::new();

    let mut expanded_paths = Vec::new();
    for path in paths {
        let path_str = path.to_string_lossy();
        if path_str.contains('*') || path_str.contains('?') {
            match glob::glob(&path_str) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        expanded_paths.push(entry);
                    }
                }
                Err(e) => {
                    return Err(ParseError(format!(
                        "Invalid glob pattern '{}': {e}",
                        path_str
                    )));
                }
            }
        } else {
            expanded_paths.push(path.clone());
        }
    }

    for path in &expanded_paths {
        if path.is_file() {
            let base = path.parent().unwrap_or(path);
            if let Some(test) = load_test_file(path, base)? {
                tests.push(test);
            }
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file()
                    && entry_path.extension().is_some_and(|e| e == "js")
                    && !entry_path
                        .file_name()
                        .is_some_and(|n| n.to_string_lossy().contains("_FIXTURE"))
                {
                    if let Some(test) = load_test_file(entry_path, path)? {
                        tests.push(test);
                    }
                }
            }
        }
    }

    if let Some(include) = features_include {
        tests.retain(|t| {
            t.metadata
                .features
                .iter()
                .any(|f| include.iter().any(|inc| inc == f))
        });
    }
    if let Some(exclude) = features_exclude {
        tests.retain(|t| {
            !t.metadata
                .features
                .iter()
                .any(|f| exclude.iter().any(|exc| exc == f))
        });
    }

    Ok(tests)
}

pub fn detect_harness_dir(test_path: &PathBuf) -> Option<PathBuf> {
    let start = if test_path.is_file() {
        test_path.parent()?
    } else {
        test_path.as_path()
    };

    let mut current = std::fs::canonicalize(start).ok()?;
    loop {
        let candidate = current.join("harness");
        if candidate.join("assert.js").exists() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn load_test_file(
    file_path: &std::path::Path,
    base_path: &std::path::Path,
) -> Result<Option<TestCase>, ParseError> {
    let contents = std::fs::read_to_string(file_path)
        .map_err(|e| ParseError(format!("Failed to read {}: {e}", file_path.display())))?;

    let relative_path = file_path
        .strip_prefix(base_path)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    let metadata = match extract_frontmatter(&contents) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "Warning: failed to parse frontmatter for {}: {e}",
                file_path.display()
            );
            return Ok(None);
        }
    };

    Ok(Some(TestCase {
        path: file_path.to_path_buf(),
        relative_path,
        contents,
        metadata,
    }))
}

fn extract_string_list(yaml: Option<&Yaml>) -> Vec<String> {
    match yaml {
        Some(Yaml::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

fn extract_flags(yaml: Option<&Yaml>) -> HashSet<TestFlag> {
    let mut flags = HashSet::new();
    if let Some(Yaml::Array(arr)) = yaml {
        for item in arr {
            if let Some(s) = item.as_str() {
                if let Some(flag) = parse_flag(s) {
                    flags.insert(flag);
                }
            }
        }
    }
    flags
}

fn parse_flag(s: &str) -> Option<TestFlag> {
    match s {
        "onlyStrict" => Some(TestFlag::OnlyStrict),
        "noStrict" => Some(TestFlag::NoStrict),
        "module" => Some(TestFlag::Module),
        "raw" => Some(TestFlag::Raw),
        "async" => Some(TestFlag::Async),
        "generated" => Some(TestFlag::Generated),
        "CanBlockIsFalse" => Some(TestFlag::CanBlockIsFalse),
        "CanBlockIsTrue" => Some(TestFlag::CanBlockIsTrue),
        "non-deterministic" => Some(TestFlag::NonDeterministic),
        _ => None,
    }
}

fn extract_negative(yaml: Option<&Yaml>) -> Result<Option<NegativeExpectation>, ParseError> {
    let yaml = match yaml {
        Some(y) => y,
        None => return Ok(None),
    };

    let hash = yaml
        .as_hash()
        .ok_or_else(|| ParseError("negative must be a mapping".to_string()))?;

    let phase_str = hash
        .get(&Yaml::String("phase".into()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParseError("negative.phase is required".to_string()))?;

    let phase = match phase_str {
        "parse" => NegativePhase::Parse,
        "resolution" => NegativePhase::Resolution,
        "runtime" => NegativePhase::Runtime,
        other => return Err(ParseError(format!("Unknown negative phase: {other}"))),
    };

    let error_type = hash
        .get(&Yaml::String("type".into()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParseError("negative.type is required".to_string()))?
        .to_string();

    Ok(Some(NegativeExpectation { phase, error_type }))
}

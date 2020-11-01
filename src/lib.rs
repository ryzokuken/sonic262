use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;
use std::process::Output;

use colored::Colorize;
use yaml_rust::Yaml;

fn extract_strings(yaml: Option<&Yaml>) -> Option<Vec<&str>> {
    match yaml {
        Some(arr) => Some(
            arr.as_vec()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect(),
        ),
        None => None,
    }
}

fn extract_frontmatter(contents: &str) -> Option<Yaml> {
    if let Some(yaml_start) = contents.find("/*---") {
        let yaml_end = contents.find("---*/").unwrap();
        let text = contents
            .get(yaml_start + 5..yaml_end)
            .unwrap()
            .replace("\r\n", "\n")
            .replace("\r", "\n");
        let text = text.trim_matches('\n');
        let frontmatter = yaml_rust::YamlLoader::load_from_str(&text).unwrap();
        Some(frontmatter.first().unwrap().clone())
    } else {
        None
    }
}

fn generate_includes(includes: Vec<&str>, include_path: &PathBuf) -> String {
    let mut contents = String::new();
    for include in includes {
        let mut file = std::fs::File::open(include_path.join(include)).unwrap();
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).unwrap();
        contents.push_str(&file_contents);
        contents.push('\n');
    }
    contents
}

fn process_file(test_path: &PathBuf, include_path: &PathBuf) -> Result<Output, Error> {
    let mut test_file = std::fs::File::open(test_path).unwrap();
    let mut contents = String::new();
    test_file.read_to_string(&mut contents).unwrap();
    let frontmatter = extract_frontmatter(&contents)
        .expect("no frontmatter found for the file")
        .into_hash()
        .unwrap();
    // let _flags = extract_strings(h.get(&Yaml::String(String::from("flags"))));
    // let _features = extract_strings(h.get(&Yaml::String(String::from("features"))));
    let mut includes = extract_strings(frontmatter.get(&Yaml::String(String::from("includes"))))
        .unwrap_or_default();
    includes.push("assert.js");
    includes.push("sta.js");
    let mut include_contents = generate_includes(includes, include_path);
    include_contents.push_str(&contents);
    let mut final_file = tempfile::NamedTempFile::new().unwrap();
    final_file.write_all(include_contents.as_bytes()).unwrap();
    std::process::Command::new("node")
        .arg(final_file.path())
        .output()
}

fn print_test_status(output: Output, path: &str) {
    if output.status.success() {
        println!("{} {}", "PASS".to_string().green(), path);
    } else {
        println!("{} {}", "FAIL".to_string().red(), path);
        println!("{}", String::from_utf8(output.stderr).unwrap());
    }
}

pub fn run_test(test_path: PathBuf, include_path: PathBuf) -> Result<(), Error> {
    if test_path.is_file() {
        let res = process_file(&test_path, &include_path);
        if let Err(e) = res {
            return Err(e);
        }
        print_test_status(res.unwrap(), test_path.to_str().unwrap());
        Ok(())
    } else {
        for entry in walkdir::WalkDir::new(test_path.clone()) {
            let ent = entry.unwrap();
            if ent.metadata().unwrap().is_file() {
                let res = process_file(&ent.path().to_path_buf(), &include_path);
                if let Err(e) = res {
                    return Err(e);
                }
                print_test_status(
                    res.unwrap(),
                    ent.path()
                        .strip_prefix(test_path.clone())
                        .unwrap()
                        .to_str()
                        .unwrap(),
                );
            }
        }
        Ok(())
    }
}

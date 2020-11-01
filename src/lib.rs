use colored::Colorize;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::process::ExitStatus;
use tempfile::NamedTempFile;
use walkdir::WalkDir;
use std::path::PathBuf;
use serde_yaml::Value::Sequence;

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub total_files: usize,
    pub run: u16,
    pub passed: u16,
    pub failed: u16,
}

pub fn generate_and_run(
    file_to_test: &PathBuf,
    files_to_add: Vec<PathBuf>,
    diagnostics: &mut Diagnostics,
) {
    match generate_final_file_to_test(file_to_test, files_to_add) {
        Ok(file_to_run) => match run_file_in_node(file_to_run) {
            Ok(exit_status) => {
                if exit_status.success() {
                    diagnostics.run += 1;
                    diagnostics.passed += 1;
                    println!("{} {:?}", "Great Success".green(), file_to_test);
                } else {
                    diagnostics.run += 1;
                    diagnostics.failed += 1;
                    eprintln!("{} {:?}", "FAIL".red(), file_to_test);
                }
            }
            Err(e) => eprintln!("Failed to execute the file_to_test | Error: {:?}", e),
        },
        Err(e) => eprintln!(
            "Couldn't generate file: {:?} to test | Err: {}",
            file_to_test, e
        ),
    }
}

// TODO walk file in parallel using jwalk
fn walk(root_path: PathBuf) -> walkdir::Result<Vec<PathBuf>> {
    let mut final_paths: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(root_path) {
        // FIXME possible unecessary clone
        let entry_clone = entry?.clone();
        if entry_clone.file_type().is_file() {
            final_paths.push(entry_clone.into_path());
        } else {
        }
    }
    Ok(final_paths)
}

pub fn extract_frontmatter(file_to_test: &PathBuf) -> Option<String> {
    // FIXME remove unwrap
    // TODO Read asynchronously
    let file_contents = fs::read_to_string(file_to_test).unwrap();
    let yaml_start = file_contents.find("/*---");
    if let Some(start) = yaml_start {
        let yaml_end = file_contents.find("---*/");
        if let Some(end) = yaml_end {
            // TODO remove unwrap here
            Some(file_contents.get(start + 5..end).unwrap().to_string())
        } else {
            eprintln!("This file has an invalid frontmatter");
            None
        }
    } else {
        eprintln!("frontmatter not found in file: {:?}", file_to_test);
        None
    }
}

pub fn get_serde_value(frontmatter_str: &str) -> serde_yaml::Result<serde_yaml::Value> {
    serde_yaml::from_str(frontmatter_str)
}

pub fn get_include_paths(
    includes_value: &serde_yaml::Value,
    include_path_root: &PathBuf,
) -> serde_yaml::Result<Vec<PathBuf>> {
    let mut includes: Vec<String> = serde_yaml::from_value(includes_value.clone())?;
    let must_include = &mut vec!["assert.js".to_string(), "sta.js".to_string()];
    includes.append(must_include);
    let mut include_paths: Vec<PathBuf> = vec![];
    for include in includes {
        include_paths.push(include_path_root.join(include));
    }
    Ok(include_paths)
}

fn generate_final_file_to_test(
    file_to_test: &PathBuf,
    files_to_add: Vec<PathBuf>,
) -> std::io::Result<NamedTempFile> {
    // TODO do this asynchronously
    let mut contents = String::new();
    for file in files_to_add {
        let file_contents = fs::read_to_string(file)?;
        contents.push_str(&file_contents);
    }
    let file_to_test_contents = fs::read_to_string(file_to_test)?;
    contents.push_str(&file_to_test_contents);
    let mut file = tempfile::Builder::new().suffix(".js").tempfile()?;
    writeln!(file, "{}", contents)?;
    Ok(file)
}

pub fn run_file_in_node(file: NamedTempFile) -> std::io::Result<ExitStatus> {
    // TODO .status() waits for the command to execute
    // FIXME currently shows all the errors if node is not able to run the file
    Command::new("node").arg(file.path()).status()
}

pub fn run(test_path: PathBuf, include_path: PathBuf, mut diagnostics: Diagnostics) {
    let files_to_test = walk(test_path).unwrap();
    for file in files_to_test {
        let frontmatter = extract_frontmatter(&file);
        match frontmatter {
            Some(f) => match get_serde_value(&f) {
                Ok(frontmatter_value) => match frontmatter_value.get("includes") {
                    Some(includes_value) => {
                        match get_include_paths(includes_value, &include_path) {
                            Ok(include_files) => {
                                generate_and_run(&file, include_files, &mut diagnostics)
                            }
                            Err(e) => eprintln!("Not able to find {:?} | Err: {}", file, e),
                        }
                    }
                    None => match get_include_paths(&Sequence([].to_vec()), &include_path) {
                        Ok(include_files) => {
                            generate_and_run(&file, include_files, &mut diagnostics)
                        }
                        Err(e) => eprintln!("Not able to find {:?} | Err: {}", file, e),
                    },
                },
                Err(e) => eprintln!("Could not get serde value from frontmatter | Err: {}", e),
            },
            None => generate_and_run(&file, vec![], &mut diagnostics),
        }
    }
    println!(
        "TOTAL: {}, FAILED: {}, PASSED: {}",
        diagnostics.run.to_string().yellow(),
        diagnostics.failed.to_string().red(),
        diagnostics.passed.to_string().green()
    );
}
